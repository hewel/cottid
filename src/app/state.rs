use std::collections::HashMap;
use std::path::PathBuf;

use crate::app::scheduler::{RefreshDirtyFlags, RefreshScheduler, RefreshTrigger};
use crate::app::{ActionMessage, ActionTarget, RefreshInvalidation};
use crate::aria2::client::{BatchRefreshRequest, ConnectionTest};
use crate::aria2::domain::{
    DownloadDetail, DownloadFile, DownloadItem, DownloadSnapshot, DownloadStatus, Gid, GlobalStats,
};
use crate::aria2::errors::ClientError;
use crate::aria2::notifications::Aria2Notification;
use crate::config::{
    AuthStorage, ConfigLoad, PersistedConfig, RpcAuthDraft, Settings, SettingsDraft,
    SystemTokenStore, ThemePreference, default_config_path, load_config,
    save_config_with_token_store,
};
use crate::ui::overlay::{PopoverId, PopoverState};
use crate::util::format::{
    format_bytes, format_count, format_eta, format_eta_duration, format_progress, format_speed,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Offline,
    Testing,
    Connected,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadFilter {
    All,
    Active,
    Waiting,
    Paused,
    Complete,
    Error,
}

impl DownloadFilter {
    pub const ALL: [Self; 6] = [
        Self::All,
        Self::Active,
        Self::Waiting,
        Self::Paused,
        Self::Complete,
        Self::Error,
    ];
    pub const VISIBLE: [Self; 2] = [Self::Active, Self::Complete];

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Waiting => "Waiting",
            Self::Paused => "Paused",
            Self::Complete => "Complete",
            Self::Error => "Error",
        }
    }

    pub fn config_value(self) -> &'static str {
        match self.visible_sidebar_filter() {
            Self::Active => "active",
            Self::Complete => "complete",
            Self::All | Self::Waiting | Self::Paused | Self::Error => unreachable!(),
        }
    }

    pub fn from_config_value(value: &str) -> Self {
        match value {
            "active" => Self::Active,
            "complete" => Self::Complete,
            "error" => Self::Complete,
            "all" | "waiting" | "paused" => Self::Active,
            _ => Self::Active,
        }
    }

    pub fn visible_sidebar_filter(self) -> Self {
        match self {
            Self::Complete | Self::Error => Self::Complete,
            Self::All | Self::Active | Self::Waiting | Self::Paused => Self::Active,
        }
    }

    fn matches(self, item: &DownloadItem) -> bool {
        match self {
            Self::All => true,
            Self::Active => matches!(
                item.status(),
                DownloadStatus::Waiting | DownloadStatus::Paused | DownloadStatus::Active
            ),
            Self::Waiting => matches!(item.status(), DownloadStatus::Waiting),
            Self::Paused => matches!(item.status(), DownloadStatus::Paused),
            Self::Complete => {
                matches!(
                    item.status(),
                    DownloadStatus::Error | DownloadStatus::Complete
                )
            }
            Self::Error => matches!(item.status(), DownloadStatus::Error),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshState {
    NeverRefreshed,
    Refreshing,
    Fresh,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackTone {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFeedback {
    tone: FeedbackTone,
    message: String,
}

impl FormFeedback {
    pub fn info(message: impl Into<String>) -> Self {
        Self::new(FeedbackTone::Info, message)
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(FeedbackTone::Success, message)
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(FeedbackTone::Warning, message)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(FeedbackTone::Error, message)
    }

    fn new(tone: FeedbackTone, message: impl Into<String>) -> Self {
        Self {
            tone,
            message: message.into(),
        }
    }

    pub fn tone(&self) -> FeedbackTone {
        self.tone
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadRowView {
    name: String,
    gid: String,
    gid_value: Gid,
    file_icon: FileIcon,
    metadata: String,
    progress: String,
    progress_per_mille: u16,
    download_speed: String,
    upload_speed: String,
    eta: String,
    can_pause: bool,
    can_unpause: bool,
    can_remove: bool,
    pending: bool,
    error: Option<String>,
    selected: bool,
}

impl DownloadRowView {
    pub fn name(&self) -> &str {
        &self.name
    }

    #[cfg(test)]
    pub fn gid(&self) -> &str {
        &self.gid
    }

    pub fn gid_value(&self) -> Gid {
        self.gid_value.clone()
    }

    pub fn file_icon(&self) -> FileIcon {
        self.file_icon
    }

    pub fn metadata(&self) -> &str {
        &self.metadata
    }

    pub fn progress(&self) -> &str {
        &self.progress
    }

    pub fn progress_ratio(&self) -> f32 {
        f32::from(self.progress_per_mille) / 1000.0
    }

    pub fn download_speed(&self) -> &str {
        &self.download_speed
    }

    pub fn upload_speed(&self) -> &str {
        &self.upload_speed
    }

    pub fn eta(&self) -> &str {
        &self.eta
    }

    pub fn can_pause(&self) -> bool {
        self.can_pause
    }

    pub fn can_unpause(&self) -> bool {
        self.can_unpause
    }

    pub fn can_remove(&self) -> bool {
        self.can_remove
    }

    #[cfg(test)]
    pub fn pending(&self) -> bool {
        self.pending
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    pub fn selected(&self) -> bool {
        self.selected
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadDetailView {
    name: String,
    gid: String,
    file_icon: FileIcon,
    status: String,
    directory: Option<String>,
    progress: String,
    speeds: String,
    totals: String,
    technical: Vec<String>,
    torrent: Vec<String>,
    files: Vec<String>,
    error: Option<String>,
}

impl DownloadDetailView {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn gid(&self) -> &str {
        &self.gid
    }

    pub fn file_icon(&self) -> FileIcon {
        self.file_icon
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn directory(&self) -> Option<&str> {
        self.directory.as_deref()
    }

    pub fn progress(&self) -> &str {
        &self.progress
    }

    pub fn speeds(&self) -> &str {
        &self.speeds
    }

    pub fn totals(&self) -> &str {
        &self.totals
    }

    pub fn technical(&self) -> &[String] {
        &self.technical
    }

    pub fn torrent(&self) -> &[String] {
        &self.torrent
    }

    pub fn files(&self) -> &[String] {
        &self.files
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileIcon {
    Archive,
    Audio,
    Document,
    Executable,
    File,
    Folder,
    Image,
    Torrent,
    Video,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    add: AddState,
    connection: ConnectionState,
    settings: SettingsState,
    stats: StatsState,
    downloads: DownloadsState,
    scheduler: RefreshScheduler,
    actions: ActionsState,
    selection: SelectionState,
    popovers: PopoverState,
    viewport_width: u32,
    viewport_height: u32,
}

impl State {
    #[cfg(test)]
    pub fn initial() -> Self {
        Self::from_persisted_config(PersistedConfig::default(), None, None)
    }

    pub fn load() -> Self {
        let config_path = default_config_path();
        Self::load_from_path(config_path)
    }

    pub fn load_from_path(config_path: PathBuf) -> Self {
        let loaded = load_config(&config_path);
        Self::from_config_load(loaded, Some(config_path))
    }

    fn from_config_load(loaded: ConfigLoad, config_path: Option<PathBuf>) -> Self {
        let feedback = loaded.feedback().map(FormFeedback::warning);
        Self::from_persisted_config(loaded.into_config(), config_path, feedback)
    }

    fn from_persisted_config(
        config: PersistedConfig,
        config_path: Option<PathBuf>,
        feedback: Option<FormFeedback>,
    ) -> Self {
        let applied_settings = config.settings().clone();
        let draft = SettingsDraft::from_settings(&applied_settings);
        let selected_filter = DownloadFilter::from_config_value(config.selected_filter());

        Self {
            add: AddState {
                open: false,
                input: String::new(),
                generation: 0,
                pending: false,
                feedback: None,
            },
            connection: ConnectionState {
                status: ConnectionStatus::Offline,
                generation: 0,
                version: None,
                settings: None,
            },
            settings: SettingsState {
                applied: applied_settings,
                draft,
                theme_preference: config.theme_preference(),
                draft_theme_preference: config.theme_preference(),
                open: false,
                feedback,
                config_path,
                auth_storage: config.auth_storage(),
                pending_plaintext_fallback: None,
            },
            stats: StatsState { global: None },
            downloads: DownloadsState {
                generation: 0,
                items_by_gid: HashMap::new(),
                active_order: Vec::new(),
                waiting_order: Vec::new(),
                stopped_order: Vec::new(),
                merge_tick: 0,
                filter: selected_filter,
                refresh_state: RefreshState::NeverRefreshed,
                feedback: None,
            },
            scheduler: RefreshScheduler::default(),
            actions: ActionsState {
                generation: 0,
                pending: None,
                feedback: None,
            },
            selection: SelectionState { selected_gid: None },
            popovers: PopoverState::default(),
            viewport_width: 1280,
            viewport_height: 800,
        }
    }

    pub fn connection_status(&self) -> ConnectionStatus {
        self.connection.status
    }

    pub fn is_add_open(&self) -> bool {
        self.add.open
    }

    pub fn is_add_pending(&self) -> bool {
        self.add.pending
    }

    pub fn add_input(&self) -> &str {
        &self.add.input
    }

    pub fn add_feedback(&self) -> Option<&FormFeedback> {
        self.add.feedback.as_ref()
    }

    pub fn is_add_ready(&self) -> bool {
        validate_add_input(&self.add.input).is_ok() && self.is_connected() && !self.add.pending
    }

    pub fn is_settings_ready(&self) -> bool {
        self.settings.draft.apply().is_ok()
    }

    pub fn is_settings_open(&self) -> bool {
        self.settings.open
    }

    pub fn is_popover_open(&self, id: PopoverId) -> bool {
        self.popovers.is_open(id)
    }

    pub fn applied_endpoint(&self) -> &str {
        self.settings.applied.endpoint()
    }

    pub fn applied_auth_label(&self) -> &'static str {
        self.settings.applied.auth().display_label()
    }

    pub fn draft_endpoint(&self) -> &str {
        self.settings.draft.endpoint()
    }

    pub fn draft_auth(&self) -> RpcAuthDraft {
        self.settings.draft.auth()
    }

    pub fn draft_secret(&self) -> &str {
        self.settings.draft.secret()
    }

    pub fn draft_polling_interval_seconds(&self) -> u16 {
        self.settings.draft.polling_interval_seconds()
    }

    pub fn theme_preference(&self) -> ThemePreference {
        self.settings.theme_preference
    }

    pub fn draft_theme_preference(&self) -> ThemePreference {
        self.settings.draft_theme_preference
    }

    pub fn settings_feedback(&self) -> Option<&FormFeedback> {
        self.settings.feedback.as_ref()
    }

    pub fn is_plaintext_fallback_pending(&self) -> bool {
        self.settings.pending_plaintext_fallback.is_some()
    }

    pub fn download_speed_text(&self) -> String {
        format_speed(
            self.stats
                .global
                .map(|stats| stats.download_speed_bytes_per_second())
                .unwrap_or(0),
        )
    }

    pub fn upload_speed_text(&self) -> String {
        format_speed(
            self.stats
                .global
                .map(|stats| stats.upload_speed_bytes_per_second())
                .unwrap_or(0),
        )
    }

    pub fn counts_text(&self) -> String {
        let Some(stats) = self.stats.global else {
            return "Active 0 | Waiting 0 | Stopped 0".to_owned();
        };

        format!(
            "{} | {} | {}",
            format_count("Active", stats.active_downloads() as usize),
            format_count("Waiting", stats.waiting_downloads() as usize),
            format_count("Stopped", stats.stopped_downloads() as usize)
        )
    }

    pub fn refresh_state(&self) -> RefreshState {
        self.downloads.refresh_state
    }

    pub fn refresh_feedback(&self) -> Option<&str> {
        self.downloads
            .feedback
            .as_deref()
            .or(self.actions.feedback.as_deref())
    }

    pub fn selected_filter(&self) -> DownloadFilter {
        self.downloads.filter
    }

    #[cfg(test)]
    pub fn selected_gid(&self) -> Option<&Gid> {
        self.selection.selected_gid.as_ref()
    }

    pub fn filter_count(&self, filter: DownloadFilter) -> usize {
        self.downloads
            .items_by_gid
            .values()
            .filter(|record| filter.matches(&record.item))
            .count()
    }

    pub fn download_rows(&self) -> Vec<DownloadRowView> {
        self.downloads
            .ordered_items_for_filter(self.downloads.filter)
            .into_iter()
            .map(|item| {
                download_row_view(item, &self.actions, self.selection.selected_gid.as_ref())
            })
            .collect()
    }

    pub fn selected_download_detail(&self) -> Option<DownloadDetailView> {
        let selected_gid = self.selection.selected_gid.as_ref()?;
        self.downloads
            .items_by_gid
            .get(selected_gid)
            .map(download_detail_view)
    }

    pub fn can_purge_stopped(&self) -> bool {
        self.is_connected()
            && self.actions.pending.is_none()
            && self.downloads.items_by_gid.values().any(|record| {
                matches!(
                    record.item.status(),
                    DownloadStatus::Complete | DownloadStatus::Error
                )
            })
    }

    pub fn downloads_empty_text(&self) -> Option<String> {
        if matches!(self.downloads.refresh_state, RefreshState::Refreshing)
            && self.downloads.is_empty()
        {
            return Some("Loading downloads...".to_owned());
        }

        if self.downloads.is_empty() {
            return Some("No downloads found.".to_owned());
        }

        if self.download_rows().is_empty() {
            return Some(format!(
                "No {} downloads.",
                self.downloads.filter.label().to_ascii_lowercase()
            ));
        }

        None
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.connection.status, ConnectionStatus::Connected)
    }

    pub fn is_compact_layout(&self) -> bool {
        self.viewport_width < 900
    }

    pub(super) fn set_viewport_size(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    pub fn modal_max_width(&self, target_width: f32) -> f32 {
        target_width.min(self.viewport_width as f32 * 0.9)
    }

    pub fn modal_max_height(&self) -> f32 {
        self.viewport_height as f32 * 0.75
    }

    pub(super) fn polling_interval_seconds(&self) -> u16 {
        self.settings.applied.polling_interval_seconds()
    }

    #[cfg(test)]
    pub fn global_stats(&self) -> Option<GlobalStats> {
        self.stats.global
    }

    #[cfg(test)]
    pub fn download_items(&self) -> Vec<&DownloadItem> {
        self.downloads.ordered_items_for_filter(DownloadFilter::All)
    }

    pub fn status_text(&self) -> String {
        format!(
            "{} | {}",
            connection_label(self.connection.status),
            self.settings.applied.auth().display_label()
        )
    }

    #[cfg(test)]
    pub fn connected_version(&self) -> Option<&str> {
        self.connection.version.as_deref()
    }

    pub(super) fn begin_connection_test(&mut self) -> Option<(u64, Settings)> {
        let settings = if self.settings.open {
            match self.settings.draft.apply() {
                Ok(settings) => settings,
                Err(error) => {
                    self.connection.status = ConnectionStatus::Failed;
                    self.settings.feedback = Some(FormFeedback::error(error.message()));
                    return None;
                }
            }
        } else {
            self.settings.applied.clone()
        };

        self.connection.generation += 1;
        self.connection.status = ConnectionStatus::Testing;
        self.connection.version = None;
        self.connection.settings = None;
        self.clear_snapshot();
        self.settings.feedback = None;

        Some((self.connection.generation, settings))
    }

    pub(super) fn finish_connection_test(
        &mut self,
        generation: u64,
        settings: Settings,
        result: Result<ConnectionTest, ClientError>,
    ) -> bool {
        if generation != self.connection.generation {
            return false;
        }

        match result {
            Ok(result) => {
                self.connection.status = ConnectionStatus::Connected;
                self.connection.version = Some(result.version().version().to_owned());
                self.connection.settings = Some(settings.clone());
                if self.settings.open {
                    let previous_endpoint = Some(self.settings.applied.endpoint().to_owned());
                    self.commit_settings(
                        settings,
                        previous_endpoint,
                        false,
                        "Connection test succeeded and settings saved.",
                    );
                } else {
                    self.settings.feedback =
                        Some(FormFeedback::success("Connection test succeeded."));
                }
                true
            }
            Err(error) => {
                self.connection.status = ConnectionStatus::Failed;
                self.connection.version = None;
                self.connection.settings = None;
                self.clear_snapshot();
                self.settings.feedback = Some(FormFeedback::error(error.display_message()));
                false
            }
        }
    }

    pub(super) fn begin_downloads_refresh(
        &mut self,
    ) -> Option<(u64, Settings, BatchRefreshRequest)> {
        self.begin_downloads_refresh_with_trigger(RefreshTrigger::Manual)
    }

    pub(super) fn begin_scheduled_downloads_refresh(
        &mut self,
    ) -> Option<(u64, Settings, BatchRefreshRequest)> {
        self.begin_downloads_refresh_with_trigger(RefreshTrigger::Scheduled)
    }

    pub(super) fn begin_dirty_downloads_refresh(
        &mut self,
    ) -> Option<(u64, Settings, BatchRefreshRequest)> {
        self.begin_downloads_refresh_with_trigger(RefreshTrigger::Dirty)
    }

    fn begin_downloads_refresh_with_trigger(
        &mut self,
        trigger: RefreshTrigger,
    ) -> Option<(u64, Settings, BatchRefreshRequest)> {
        let settings = self.connection.settings.clone()?;
        let selected_gid = self.selection.selected_gid.as_ref();
        let has_active_downloads = !self.downloads.active_order.is_empty();
        let (generation, request) = self.scheduler.begin_refresh(
            trigger,
            has_active_downloads,
            self.settings.open,
            selected_gid,
        )?;

        self.downloads.generation = generation;
        self.downloads.refresh_state = RefreshState::Refreshing;
        self.downloads.feedback = None;

        Some((generation, settings, request))
    }

    pub(super) fn finish_downloads_refresh(
        &mut self,
        generation: u64,
        result: Result<DownloadSnapshot, ClientError>,
    ) {
        match result {
            Ok(snapshot) => {
                let Some(request) = self.scheduler.complete_success(generation) else {
                    return;
                };
                self.merge_download_snapshot(request, snapshot);
                self.clear_missing_selection();
                self.downloads.refresh_state = RefreshState::Fresh;
                self.downloads.feedback = None;
            }
            Err(error) => {
                if !self.scheduler.complete_failure(generation) {
                    return;
                }
                self.downloads.refresh_state = if self.downloads.is_empty() {
                    RefreshState::NeverRefreshed
                } else {
                    RefreshState::Stale
                };
                self.downloads.feedback = Some(error.display_message().to_owned());
            }
        }
    }

    pub(super) fn invalidate_refresh(&mut self, invalidation: RefreshInvalidation) -> bool {
        let dirty = match invalidation {
            RefreshInvalidation::Active => RefreshDirtyFlags {
                active: true,
                ..RefreshDirtyFlags::default()
            },
            RefreshInvalidation::Waiting => RefreshDirtyFlags {
                waiting: true,
                ..RefreshDirtyFlags::default()
            },
            RefreshInvalidation::Stopped => RefreshDirtyFlags {
                stopped: true,
                ..RefreshDirtyFlags::default()
            },
            RefreshInvalidation::Selected => RefreshDirtyFlags {
                selected: true,
                ..RefreshDirtyFlags::default()
            },
            RefreshInvalidation::All => RefreshDirtyFlags {
                active: true,
                waiting: true,
                stopped: true,
                selected: true,
            },
            RefreshInvalidation::Aria2Notification(notification) => {
                self.notification_dirty_flags(&notification)
            }
        };

        let has_dirty = dirty.active || dirty.waiting || dirty.stopped || dirty.selected;
        if !has_dirty {
            return false;
        }

        self.scheduler.mark_dirty(dirty);
        true
    }

    fn notification_dirty_flags(&self, notification: &Aria2Notification) -> RefreshDirtyFlags {
        let selected = notification
            .gid()
            .is_some_and(|gid| self.selection.selected_gid.as_ref() == Some(gid));

        match notification {
            Aria2Notification::DownloadStart(_) | Aria2Notification::DownloadPause(_) => {
                RefreshDirtyFlags {
                    active: true,
                    waiting: true,
                    selected,
                    ..RefreshDirtyFlags::default()
                }
            }
            Aria2Notification::DownloadStop(_)
            | Aria2Notification::DownloadComplete(_)
            | Aria2Notification::DownloadError(_)
            | Aria2Notification::BtDownloadComplete(_) => RefreshDirtyFlags {
                active: true,
                waiting: true,
                stopped: true,
                selected,
            },
            Aria2Notification::Unknown { gid: Some(_), .. } => RefreshDirtyFlags {
                active: true,
                waiting: true,
                stopped: true,
                selected,
            },
            Aria2Notification::Unknown { gid: None, .. } => RefreshDirtyFlags::default(),
        }
    }

    pub(super) fn set_download_filter(&mut self, filter: DownloadFilter) {
        self.downloads.filter = filter.visible_sidebar_filter();
        self.persist_config(None, None);
    }

    pub(super) fn select_download(&mut self, gid: Gid) -> bool {
        if self.downloads.items_by_gid.contains_key(&gid) {
            self.selection.selected_gid = Some(gid);
            self.scheduler.mark_dirty(RefreshDirtyFlags {
                selected: true,
                ..RefreshDirtyFlags::default()
            });
            return true;
        }

        false
    }

    pub(super) fn clear_selection(&mut self) {
        self.selection.selected_gid = None;
    }

    pub(super) fn begin_action(
        &mut self,
        message: ActionMessage,
    ) -> Option<(u64, Settings, RunningAction)> {
        if self.actions.pending.is_some() {
            return None;
        }

        let settings = self.connection.settings.clone()?;
        let action = match message {
            ActionMessage::Pause(gid) => {
                if !self.can_pause(&gid) {
                    return None;
                }
                RunningAction::Pause(gid)
            }
            ActionMessage::Unpause(gid) => {
                if !self.can_unpause(&gid) {
                    return None;
                }
                RunningAction::Unpause(gid)
            }
            ActionMessage::Remove(gid) => {
                if !self.can_remove(&gid) {
                    return None;
                }
                RunningAction::Remove(gid)
            }
            ActionMessage::PurgeStopped => {
                if !self.can_purge_stopped() {
                    return None;
                }
                RunningAction::PurgeStopped
            }
            ActionMessage::Finished { .. } => return None,
        };

        self.actions.generation += 1;
        self.actions.feedback = None;
        self.actions.pending = Some(action.clone());

        Some((self.actions.generation, settings, action))
    }

    pub(super) fn finish_action(
        &mut self,
        generation: u64,
        target: ActionTarget,
        result: Result<(), ClientError>,
    ) -> bool {
        if generation != self.actions.generation {
            return false;
        }

        self.actions.pending = None;

        match result {
            Ok(()) => {
                self.actions.feedback = None;
                self.downloads.feedback = None;
                true
            }
            Err(error) => {
                let message = error.display_message().to_owned();
                match target {
                    ActionTarget::Download(gid) => {
                        self.set_item_error(&gid, message);
                    }
                    ActionTarget::PurgeStopped => {
                        self.actions.feedback = Some(message);
                    }
                }
                false
            }
        }
    }

    pub(super) fn open_settings(&mut self) {
        if self.add.pending {
            return;
        }

        self.popovers.close();

        if self.add.open {
            self.cancel_add_dialog();
        }

        self.settings.open = true;
        self.settings.feedback = self
            .settings
            .draft
            .endpoint_validation_message()
            .map(FormFeedback::error);
    }

    pub(super) fn open_add_dialog(&mut self) {
        self.popovers.close();

        if self.settings.open {
            self.cancel_settings();
        }

        self.add.open = true;
        self.add.feedback = None;
    }

    pub(super) fn cancel_add_dialog(&mut self) {
        if self.add.pending {
            return;
        }

        self.add.open = false;
        self.add.input.clear();
        self.add.feedback = None;
    }

    pub(super) fn cancel_active_modal(&mut self) {
        if self.add.open {
            self.cancel_add_dialog();
        } else if self.settings.open {
            self.cancel_settings();
        }
    }

    pub(super) fn toggle_popover(&mut self, id: PopoverId) {
        if self.add.open || self.settings.open {
            return;
        }

        self.popovers.toggle(id);
    }

    pub(super) fn close_popover(&mut self) {
        self.popovers.close();
    }

    pub(super) fn set_add_input(&mut self, input: String) {
        self.add.input = input;
        self.add.feedback = None;
    }

    pub(super) fn begin_add_uri(&mut self) -> Option<(u64, Settings, String)> {
        let uri = match validate_add_input(&self.add.input) {
            Ok(uri) => uri,
            Err(message) => {
                self.add.feedback = Some(FormFeedback::error(message));
                return None;
            }
        };

        let Some(settings) = self.connection.settings.clone() else {
            self.add.feedback = Some(FormFeedback::warning(
                "Connect to aria2 before adding a download.",
            ));
            return None;
        };

        self.add.generation += 1;
        self.add.pending = true;
        self.add.feedback = Some(FormFeedback::info("Adding download..."));

        Some((self.add.generation, settings, uri))
    }

    pub(super) fn finish_add_uri(
        &mut self,
        generation: u64,
        result: Result<crate::aria2::domain::Gid, ClientError>,
    ) -> bool {
        if generation != self.add.generation {
            return false;
        }

        self.add.pending = false;

        match result {
            Ok(_) => {
                self.add.input.clear();
                self.add.feedback = Some(FormFeedback::success("Download added."));
                true
            }
            Err(error) => {
                self.add.feedback = Some(FormFeedback::error(error.display_message()));
                false
            }
        }
    }

    pub(super) fn cancel_settings(&mut self) {
        if let Some(pending) = self.settings.pending_plaintext_fallback.take() {
            self.settings.applied = pending.previous_settings;
            self.settings.draft = SettingsDraft::from_settings(&self.settings.applied);
            self.settings.theme_preference = pending.previous_theme_preference;
            self.settings.draft_theme_preference = self.settings.theme_preference;
            self.settings.auth_storage = pending.previous_auth_storage;
            if self.connection.settings.as_ref() == Some(&pending.settings) {
                self.connection.status = ConnectionStatus::Offline;
                self.connection.version = None;
                self.connection.settings = None;
                self.clear_snapshot();
            }
        }
        self.settings.draft.cancel_to(&self.settings.applied);
        self.settings.draft_theme_preference = self.settings.theme_preference;
        self.settings.feedback = None;
        self.settings.open = false;
    }

    pub(super) fn set_draft_endpoint(&mut self, endpoint: String) {
        self.settings.draft.set_endpoint(endpoint);
        self.settings.feedback = self
            .settings
            .draft
            .endpoint_validation_message()
            .map(FormFeedback::error);
    }

    pub(super) fn set_draft_auth(&mut self, auth: RpcAuthDraft) {
        self.settings.draft.set_auth(auth);
        self.settings.feedback = None;
    }

    pub(super) fn set_draft_secret(&mut self, secret: String) {
        self.settings.draft.set_secret(secret);
        self.settings.feedback = None;
    }

    pub(super) fn set_draft_polling_interval(&mut self, value: String) {
        if let Ok(seconds) = value.parse::<u16>() {
            self.settings.draft.set_polling_interval_seconds(seconds);
            self.settings.feedback = None;
        }
    }

    pub(super) fn set_draft_theme_preference(&mut self, theme_preference: ThemePreference) {
        self.settings.draft_theme_preference = theme_preference;
        self.settings.feedback = None;
    }

    pub(super) fn save_settings(&mut self) {
        match self.settings.draft.apply() {
            Ok(settings) => {
                let previous_endpoint = Some(self.settings.applied.endpoint().to_owned());
                self.commit_settings(settings, previous_endpoint, true, "Settings saved.");
            }
            Err(error) => {
                self.settings.feedback = Some(FormFeedback::error(error.message()));
                self.settings.open = true;
            }
        }
    }

    pub(super) fn save_plaintext_fallback(&mut self) {
        let Some(pending) = self.settings.pending_plaintext_fallback.clone() else {
            return;
        };

        self.settings.applied = pending.settings.clone();
        self.settings.draft = SettingsDraft::from_settings(&self.settings.applied);
        self.settings.theme_preference = pending.theme_preference;
        self.settings.draft_theme_preference = self.settings.theme_preference;
        self.settings.pending_plaintext_fallback = None;
        self.settings.auth_storage = AuthStorage::PlaintextFallback;
        self.persist_config_with_auth_storage(
            AuthStorage::PlaintextFallback,
            pending.previous_endpoint,
            None,
        );
        self.settings.feedback = Some(FormFeedback::success(pending.success_feedback));
        self.settings.open = !pending.close_on_success;
    }

    pub(super) fn keep_secret_session_only(&mut self) {
        let Some(pending) = self.settings.pending_plaintext_fallback.clone() else {
            return;
        };

        self.settings.applied = pending.settings.clone();
        self.settings.draft = SettingsDraft::from_settings(&self.settings.applied);
        self.settings.theme_preference = pending.theme_preference;
        self.settings.draft_theme_preference = self.settings.theme_preference;
        self.settings.pending_plaintext_fallback = None;
        self.settings.auth_storage = AuthStorage::SessionOnly;
        self.persist_config_with_auth_storage(
            AuthStorage::SessionOnly,
            pending.previous_endpoint,
            None,
        );
        self.settings.feedback = Some(FormFeedback::success(
            "Settings saved. Token will be required again next launch.",
        ));
        self.settings.open = !pending.close_on_success;
    }

    fn commit_settings(
        &mut self,
        settings: Settings,
        previous_endpoint: Option<String>,
        close_on_success: bool,
        success_feedback: &'static str,
    ) {
        let previous_settings = self.settings.applied.clone();
        let previous_auth_storage = self.settings.auth_storage;
        let previous_theme_preference = self.settings.theme_preference;
        self.settings.applied = settings;
        self.settings.draft = SettingsDraft::from_settings(&self.settings.applied);
        self.settings.theme_preference = self.settings.draft_theme_preference;
        self.settings.auth_storage = self.next_auth_storage();
        self.settings.pending_plaintext_fallback = None;
        self.settings.feedback = None;
        self.settings.open = !close_on_success;

        let persisted = self.persist_config(
            previous_endpoint.clone(),
            Some((
                previous_settings,
                previous_auth_storage,
                previous_theme_preference,
            )),
        );
        if persisted {
            self.settings.feedback = Some(FormFeedback::success(success_feedback));
        } else if self.settings.pending_plaintext_fallback.is_some()
            && let Some(pending) = self.settings.pending_plaintext_fallback.as_mut()
        {
            pending.close_on_success = close_on_success;
            pending.success_feedback = success_feedback;
        }
    }

    fn next_auth_storage(&self) -> AuthStorage {
        if matches!(
            self.settings.applied.auth(),
            crate::config::RpcAuth::NoSecret
        ) {
            return AuthStorage::None;
        }

        if matches!(self.settings.auth_storage, AuthStorage::PlaintextFallback) {
            AuthStorage::PlaintextFallback
        } else {
            AuthStorage::Keyring
        }
    }

    fn persist_config(
        &mut self,
        previous_endpoint: Option<String>,
        rollback: Option<(Settings, AuthStorage, ThemePreference)>,
    ) -> bool {
        self.persist_config_with_auth_storage(
            self.settings.auth_storage,
            previous_endpoint,
            rollback,
        )
    }

    fn persist_config_with_auth_storage(
        &mut self,
        auth_storage: AuthStorage,
        previous_endpoint: Option<String>,
        rollback: Option<(Settings, AuthStorage, ThemePreference)>,
    ) -> bool {
        let config = PersistedConfig::with_auth_storage_and_theme(
            self.settings.applied.clone(),
            self.downloads.filter.config_value(),
            auth_storage,
            self.settings.theme_preference,
        );

        if let Some(path) = self.settings.config_path.as_ref()
            && let Err(error) = save_config_with_token_store(
                path,
                &config,
                previous_endpoint.as_deref(),
                &SystemTokenStore,
            )
        {
            if error.is_token_store_error()
                && matches!(auth_storage, AuthStorage::Keyring)
                && !matches!(
                    self.settings.applied.auth(),
                    crate::config::RpcAuth::NoSecret
                )
            {
                self.settings.pending_plaintext_fallback = Some(PendingSettingsSave {
                    settings: self.settings.applied.clone(),
                    previous_settings: rollback.as_ref().map_or_else(
                        || self.settings.applied.clone(),
                        |(settings, _, _)| settings.clone(),
                    ),
                    previous_auth_storage: rollback
                        .as_ref()
                        .map_or(self.settings.auth_storage, |(_, auth_storage, _)| {
                            *auth_storage
                        }),
                    previous_theme_preference: rollback.as_ref().map_or(
                        self.settings.theme_preference,
                        |(_, _, theme_preference)| *theme_preference,
                    ),
                    theme_preference: self.settings.theme_preference,
                    previous_endpoint,
                    close_on_success: false,
                    success_feedback: "Settings saved.",
                });
                self.settings.feedback = Some(FormFeedback::warning(error.message()));
                self.settings.open = true;
                return false;
            }

            self.settings.feedback = Some(FormFeedback::error(error.message()));
            return false;
        }

        true
    }

    fn merge_download_snapshot(
        &mut self,
        request: BatchRefreshRequest,
        snapshot: DownloadSnapshot,
    ) {
        let (global_stats, items, selected_detail) = snapshot.into_parts();
        self.stats.global = Some(global_stats);
        self.downloads.merge_tick += 1;
        let merge_tick = self.downloads.merge_tick;

        let mut active_order = Vec::new();
        let mut waiting_order = Vec::new();
        let mut stopped_order = Vec::new();

        for item in items {
            let gid = item.gid().clone();
            match DownloadSection::from_status(item.status()) {
                DownloadSection::Active => active_order.push(gid.clone()),
                DownloadSection::Waiting => waiting_order.push(gid.clone()),
                DownloadSection::Stopped => stopped_order.push(gid.clone()),
            }

            if let Some(record) = self.downloads.items_by_gid.get_mut(&gid) {
                record.merge(item, merge_tick);
            } else {
                self.downloads
                    .items_by_gid
                    .insert(gid, DownloadRecord::new(item, merge_tick));
            }
        }

        if let Some(detail) = selected_detail {
            self.merge_selected_detail(detail, merge_tick);
        }

        if request.include_active() {
            self.downloads.active_order = active_order;
        }
        if request.include_waiting() {
            self.downloads.waiting_order = waiting_order;
        }
        if request.include_stopped() {
            self.downloads.stopped_order = stopped_order;
        }
        self.downloads.retain_ordered_records();
    }

    fn merge_selected_detail(&mut self, detail: DownloadDetail, merge_tick: u64) {
        let gid = detail.item().gid().clone();
        let item = detail.item().clone();

        if let Some(record) = self.downloads.items_by_gid.get_mut(&gid) {
            record.merge(item, merge_tick);
            record.detail = Some(detail);
            return;
        }

        match DownloadSection::from_status(item.status()) {
            DownloadSection::Active => self.downloads.active_order.push(gid.clone()),
            DownloadSection::Waiting => self.downloads.waiting_order.push(gid.clone()),
            DownloadSection::Stopped => self.downloads.stopped_order.push(gid.clone()),
        }

        let mut record = DownloadRecord::new(item, merge_tick);
        record.detail = Some(detail);
        self.downloads.items_by_gid.insert(gid, record);
    }

    fn clear_snapshot(&mut self) {
        self.stats.global = None;
        self.downloads.items_by_gid.clear();
        self.downloads.active_order.clear();
        self.downloads.waiting_order.clear();
        self.downloads.stopped_order.clear();
        self.downloads.refresh_state = RefreshState::NeverRefreshed;
        self.downloads.feedback = None;
        self.actions.pending = None;
        self.actions.feedback = None;
        self.selection.selected_gid = None;
    }

    fn can_pause(&self, gid: &Gid) -> bool {
        self.is_connected()
            && self.actions.pending.is_none()
            && self
                .downloads
                .items_by_gid
                .get(gid)
                .is_some_and(|record| matches!(record.item.status(), DownloadStatus::Active))
    }

    fn can_unpause(&self, gid: &Gid) -> bool {
        self.is_connected()
            && self.actions.pending.is_none()
            && self.downloads.items_by_gid.get(gid).is_some_and(|record| {
                matches!(
                    record.item.status(),
                    DownloadStatus::Paused | DownloadStatus::Waiting
                )
            })
    }

    fn can_remove(&self, gid: &Gid) -> bool {
        self.is_connected()
            && self.actions.pending.is_none()
            && self.downloads.items_by_gid.get(gid).is_some_and(|record| {
                !matches!(
                    record.item.status(),
                    DownloadStatus::Complete | DownloadStatus::Removed
                )
            })
    }

    fn set_item_error(&mut self, gid: &Gid, message: String) {
        if let Some(record) = self.downloads.items_by_gid.get_mut(gid) {
            record.item.set_command_error(Some(message));
        } else {
            self.actions.feedback = Some(message);
        }
    }

    fn clear_missing_selection(&mut self) {
        let Some(selected_gid) = self.selection.selected_gid.as_ref() else {
            return;
        };

        if !self.downloads.items_by_gid.contains_key(selected_gid) {
            self.selection.selected_gid = None;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConnectionState {
    status: ConnectionStatus,
    generation: u64,
    version: Option<String>,
    settings: Option<Settings>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AddState {
    open: bool,
    input: String,
    generation: u64,
    pending: bool,
    feedback: Option<FormFeedback>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SettingsState {
    applied: Settings,
    draft: SettingsDraft,
    theme_preference: ThemePreference,
    draft_theme_preference: ThemePreference,
    open: bool,
    feedback: Option<FormFeedback>,
    config_path: Option<PathBuf>,
    auth_storage: AuthStorage,
    pending_plaintext_fallback: Option<PendingSettingsSave>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingSettingsSave {
    settings: Settings,
    previous_settings: Settings,
    previous_auth_storage: AuthStorage,
    previous_theme_preference: ThemePreference,
    theme_preference: ThemePreference,
    previous_endpoint: Option<String>,
    close_on_success: bool,
    success_feedback: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StatsState {
    global: Option<GlobalStats>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DownloadsState {
    generation: u64,
    items_by_gid: HashMap<Gid, DownloadRecord>,
    active_order: Vec<Gid>,
    waiting_order: Vec<Gid>,
    stopped_order: Vec<Gid>,
    merge_tick: u64,
    filter: DownloadFilter,
    refresh_state: RefreshState,
    feedback: Option<String>,
}

impl DownloadsState {
    fn is_empty(&self) -> bool {
        self.active_order.is_empty()
            && self.waiting_order.is_empty()
            && self.stopped_order.is_empty()
    }

    fn ordered_items_for_filter(&self, filter: DownloadFilter) -> Vec<&DownloadItem> {
        let mut items = Vec::new();

        match filter {
            DownloadFilter::All => {
                self.push_ordered_matches(&mut items, &self.active_order, |_| true);
                self.push_ordered_matches(&mut items, &self.waiting_order, |_| true);
                self.push_ordered_matches(&mut items, &self.stopped_order, |_| true);
            }
            DownloadFilter::Active => {
                self.push_ordered_matches(&mut items, &self.waiting_order, |status| {
                    matches!(status, DownloadStatus::Waiting)
                });
                self.push_ordered_matches(&mut items, &self.waiting_order, |status| {
                    matches!(status, DownloadStatus::Paused)
                });
                self.push_ordered_matches(&mut items, &self.active_order, |status| {
                    matches!(status, DownloadStatus::Active)
                });
            }
            DownloadFilter::Waiting => {
                self.push_ordered_matches(&mut items, &self.waiting_order, |status| {
                    matches!(status, DownloadStatus::Waiting)
                });
            }
            DownloadFilter::Paused => {
                self.push_ordered_matches(&mut items, &self.waiting_order, |status| {
                    matches!(status, DownloadStatus::Paused)
                });
            }
            DownloadFilter::Complete => {
                self.push_ordered_matches(&mut items, &self.stopped_order, |status| {
                    matches!(status, DownloadStatus::Error)
                });
                self.push_ordered_matches(&mut items, &self.stopped_order, |status| {
                    matches!(status, DownloadStatus::Complete)
                });
            }
            DownloadFilter::Error => {
                self.push_ordered_matches(&mut items, &self.stopped_order, |status| {
                    matches!(status, DownloadStatus::Error)
                });
            }
        }

        items
    }

    fn push_ordered_matches<'a>(
        &'a self,
        items: &mut Vec<&'a DownloadItem>,
        order: &[Gid],
        matches_status: impl Fn(&DownloadStatus) -> bool,
    ) {
        for gid in order {
            let Some(record) = self.items_by_gid.get(gid) else {
                continue;
            };
            if matches_status(record.item.status()) {
                items.push(&record.item);
            }
        }
    }

    fn retain_ordered_records(&mut self) {
        self.items_by_gid.retain(|gid, _| {
            self.active_order.contains(gid)
                || self.waiting_order.contains(gid)
                || self.stopped_order.contains(gid)
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DownloadRecord {
    item: DownloadItem,
    detail: Option<DownloadDetail>,
    revision: u64,
    last_seen_at: u64,
    last_changed_at: u64,
    last_rpc_update_at: u64,
}

impl DownloadRecord {
    fn new(item: DownloadItem, merge_tick: u64) -> Self {
        Self {
            item,
            detail: None,
            revision: 1,
            last_seen_at: merge_tick,
            last_changed_at: merge_tick,
            last_rpc_update_at: merge_tick,
        }
    }

    fn merge(&mut self, mut item: DownloadItem, merge_tick: u64) {
        if let Some(error) = self.item.command_error().map(str::to_owned) {
            item.set_command_error(Some(error));
        }

        if self.item != item {
            self.item = item;
            self.revision += 1;
            self.last_changed_at = merge_tick;
        }

        self.last_seen_at = merge_tick;
        self.last_rpc_update_at = merge_tick;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActionsState {
    generation: u64,
    pending: Option<RunningAction>,
    feedback: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectionState {
    selected_gid: Option<Gid>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DownloadSection {
    Active,
    Waiting,
    Stopped,
}

impl DownloadSection {
    fn from_status(status: &DownloadStatus) -> Self {
        match status {
            DownloadStatus::Active => Self::Active,
            DownloadStatus::Waiting | DownloadStatus::Paused => Self::Waiting,
            DownloadStatus::Complete
            | DownloadStatus::Error
            | DownloadStatus::Removed
            | DownloadStatus::Unknown(_) => Self::Stopped,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunningAction {
    Pause(Gid),
    Unpause(Gid),
    Remove(Gid),
    PurgeStopped,
}

impl RunningAction {
    pub fn target(&self) -> ActionTarget {
        match self {
            Self::Pause(gid) | Self::Unpause(gid) | Self::Remove(gid) => {
                ActionTarget::Download(gid.clone())
            }
            Self::PurgeStopped => ActionTarget::PurgeStopped,
        }
    }

    fn gid(&self) -> Option<&Gid> {
        match self {
            Self::Pause(gid) | Self::Unpause(gid) | Self::Remove(gid) => Some(gid),
            Self::PurgeStopped => None,
        }
    }
}

fn connection_label(status: ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::Offline => "Offline",
        ConnectionStatus::Testing => "Testing...",
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Failed => "Connection failed",
    }
}

fn download_row_view(
    item: &DownloadItem,
    actions: &ActionsState,
    selected_gid: Option<&Gid>,
) -> DownloadRowView {
    let pending = actions
        .pending
        .as_ref()
        .and_then(RunningAction::gid)
        .is_some_and(|gid| gid == item.gid());
    let action_available = actions.pending.is_none();
    let speed = row_speed_parts(item);

    DownloadRowView {
        name: download_name(item),
        gid: item.gid().as_str().to_owned(),
        gid_value: item.gid().clone(),
        file_icon: file_icon_for_item(item),
        metadata: download_card_metadata(item),
        progress: progress_text(item),
        progress_per_mille: progress_per_mille(item),
        download_speed: speed.download,
        upload_speed: speed.upload,
        eta: speed.eta,
        can_pause: action_available && matches!(item.status(), DownloadStatus::Active),
        can_unpause: action_available
            && matches!(
                item.status(),
                DownloadStatus::Paused | DownloadStatus::Waiting
            ),
        can_remove: action_available
            && !matches!(
                item.status(),
                DownloadStatus::Complete | DownloadStatus::Removed
            ),
        pending,
        error: item.command_error().map(str::to_owned),
        selected: selected_gid.is_some_and(|gid| gid == item.gid()),
    }
}

fn download_detail_view(record: &DownloadRecord) -> DownloadDetailView {
    let item = &record.item;
    let detail = record.detail.as_ref();
    let mut technical = Vec::new();
    let mut torrent = Vec::new();

    if let Some(detail) = detail {
        if detail.connections() > 0 {
            technical.push(format!("Connections {}", detail.connections()));
        }
        if detail.piece_length_bytes() > 0 {
            technical.push(format!(
                "Piece length {}",
                format_bytes(detail.piece_length_bytes())
            ));
        }
        if detail.piece_count() > 0 {
            technical.push(format!("Pieces {}", detail.piece_count()));
        }
        if let Some(error_code) = detail.error_code() {
            technical.push(format!("aria2 error code {error_code}"));
        }
        if let Some(error_message) = detail.error_message() {
            technical.push(format!("aria2 error {error_message}"));
        }
        if let Some(torrent_detail) = detail.torrent() {
            if let Some(info_hash) = torrent_detail.info_hash() {
                torrent.push(format!("Info hash {info_hash}"));
            }
            torrent.push(if torrent_detail.seeder() {
                "Seeding yes".to_owned()
            } else {
                "Seeding no".to_owned()
            });
            torrent.push(format!("Seeders {}", torrent_detail.num_seeders()));
        }
    }

    DownloadDetailView {
        name: download_name(item),
        gid: item.gid().as_str().to_owned(),
        file_icon: file_icon_for_item(item),
        status: item.status().display_label().to_owned(),
        directory: detail
            .and_then(DownloadDetail::directory)
            .map(str::to_owned),
        progress: progress_text(item),
        speeds: speed_text(item),
        totals: format!(
            "Completed {} | Total {}",
            format_bytes(item.completed_length_bytes()),
            if item.total_length_bytes() == 0 {
                "unknown".to_owned()
            } else {
                format_bytes(item.total_length_bytes())
            }
        ),
        files: item
            .files()
            .iter()
            .map(|file| {
                format!(
                    "{} | {} / {}",
                    file.path(),
                    format_bytes(file.completed_length_bytes()),
                    if file.length_bytes() == 0 {
                        "unknown".to_owned()
                    } else {
                        format_bytes(file.length_bytes())
                    }
                )
            })
            .collect(),
        technical,
        torrent,
        error: item.command_error().map(str::to_owned),
    }
}

fn download_name(item: &DownloadItem) -> String {
    if let Some(folder) = folder_download(item) {
        return folder.name.to_owned();
    }

    item.files()
        .iter()
        .find(|file| file.selected())
        .or_else(|| item.files().first())
        .map(|file| {
            file.path()
                .rsplit('/')
                .next()
                .filter(|name| !name.is_empty())
                .unwrap_or(file.path())
                .to_owned()
        })
        .unwrap_or_else(|| item.gid().as_str().to_owned())
}

fn file_icon_for_item(item: &DownloadItem) -> FileIcon {
    if folder_download(item).is_some() {
        return FileIcon::Folder;
    }

    let path = item
        .files()
        .iter()
        .find(|file| file.selected())
        .or_else(|| item.files().first())
        .map(DownloadFile::path)
        .unwrap_or_default();
    let lower = path.to_ascii_lowercase();

    if lower.starts_with("magnet:") || matches_extension(&lower, &["torrent"]) {
        return FileIcon::Torrent;
    }
    if matches_extension(&lower, &["zip", "rar", "7z", "tar", "gz", "bz2", "xz"]) {
        return FileIcon::Archive;
    }
    if matches_extension(&lower, &["mp4", "mkv", "webm", "avi", "mov"]) {
        return FileIcon::Video;
    }
    if matches_extension(&lower, &["mp3", "flac", "wav", "ogg", "m4a"]) {
        return FileIcon::Audio;
    }
    if matches_extension(&lower, &["png", "jpg", "jpeg", "gif", "webp", "svg"]) {
        return FileIcon::Image;
    }
    if matches_extension(
        &lower,
        &[
            "pdf", "txt", "md", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        ],
    ) {
        return FileIcon::Document;
    }
    if matches_extension(&lower, &["exe", "msi", "appimage", "deb", "rpm", "apk"]) {
        return FileIcon::Executable;
    }

    FileIcon::File
}

fn download_card_metadata(item: &DownloadItem) -> String {
    if let Some(folder) = folder_download(item) {
        return format!(
            "{} | {} | GID {}",
            file_count_label(folder.file_count),
            item.status().display_label(),
            item.gid().as_str()
        );
    }

    format!(
        "{} | GID {}",
        item.status().display_label(),
        item.gid().as_str()
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FolderDownload<'a> {
    name: &'a str,
    file_count: usize,
}

fn folder_download(item: &DownloadItem) -> Option<FolderDownload<'_>> {
    let files = item.files();
    if files.len() < 2 {
        return None;
    }

    let directory = item.directory()?;
    let mut shared_folder = None;

    for file in files {
        let relative_path = relative_file_path(directory, file.path())?;
        let (folder, child_path) = relative_path.split_once('/')?;
        if folder.is_empty() || child_path.is_empty() {
            return None;
        }

        match shared_folder {
            Some(existing) if existing != folder => return None,
            Some(_) => {}
            None => shared_folder = Some(folder),
        }
    }

    shared_folder.map(|name| FolderDownload {
        name,
        file_count: files.len(),
    })
}

fn relative_file_path<'a>(directory: &str, path: &'a str) -> Option<&'a str> {
    let directory = directory.trim_end_matches('/');
    if directory.is_empty() {
        return path.strip_prefix('/');
    }

    path.strip_prefix(directory)
        .and_then(|relative| relative.strip_prefix('/'))
}

fn file_count_label(count: usize) -> String {
    if count == 1 {
        "1 file".to_owned()
    } else {
        format!("{count} files")
    }
}

fn matches_extension(path: &str, extensions: &[&str]) -> bool {
    path.rsplit_once('.')
        .is_some_and(|(_, extension)| extensions.contains(&extension))
}

fn progress_text(item: &DownloadItem) -> String {
    format_progress(item.completed_length_bytes(), item.total_length_bytes())
}

fn progress_per_mille(item: &DownloadItem) -> u16 {
    if item.total_length_bytes() == 0 {
        return 0;
    }

    let ratio = item.completed_length_bytes() as f64 / item.total_length_bytes() as f64;
    (ratio * 1000.0).clamp(0.0, 1000.0) as u16
}

fn speed_text(item: &DownloadItem) -> String {
    let download_speed = item.download_speed_bytes_per_second();
    let upload_speed = item.upload_speed_bytes_per_second();

    if upload_speed > 0 {
        return format!(
            "Down {} | Up {}",
            format_speed(download_speed),
            format_speed(upload_speed)
        );
    }

    let eta = item
        .total_length_bytes()
        .saturating_sub(item.completed_length_bytes());
    format!(
        "Down {} | {}",
        format_speed(download_speed),
        format_eta(eta, download_speed)
    )
}

struct RowSpeedParts {
    download: String,
    upload: String,
    eta: String,
}

fn row_speed_parts(item: &DownloadItem) -> RowSpeedParts {
    let download_speed = item.download_speed_bytes_per_second();
    let upload_speed = item.upload_speed_bytes_per_second();
    let eta = item
        .total_length_bytes()
        .saturating_sub(item.completed_length_bytes());

    RowSpeedParts {
        download: format_speed(download_speed),
        upload: format_speed(upload_speed),
        eta: format_eta_duration(eta, download_speed),
    }
}

fn validate_add_input(input: &str) -> Result<String, &'static str> {
    let input = input.trim();

    if input.is_empty() {
        return Err("Enter one URI or magnet link.");
    }

    if input.contains('\n') {
        return Err("Enter only one URI or magnet link.");
    }

    if input.starts_with("http://")
        || input.starts_with("https://")
        || input.starts_with("magnet:?")
    {
        return Ok(input.to_owned());
    }

    Err("Enter an http, https, or magnet link.")
}
