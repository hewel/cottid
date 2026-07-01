use crate::app::{ActionMessage, ActionTarget};
use crate::aria2::client::ConnectionTest;
use crate::aria2::domain::{DownloadItem, DownloadSnapshot, DownloadStatus, Gid, GlobalStats};
use crate::aria2::errors::ClientError;
use crate::config::{RpcAuthDraft, Settings, SettingsDraft};
use crate::util::format::{format_bytes, format_speed};

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

    fn matches(self, item: &DownloadItem) -> bool {
        match self {
            Self::All => true,
            Self::Active => matches!(item.status(), DownloadStatus::Active),
            Self::Waiting => matches!(item.status(), DownloadStatus::Waiting),
            Self::Paused => matches!(item.status(), DownloadStatus::Paused),
            Self::Complete => matches!(item.status(), DownloadStatus::Complete),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadRowView {
    name: String,
    gid: String,
    gid_value: Gid,
    status: String,
    progress: String,
    speed: String,
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

    pub fn gid(&self) -> &str {
        &self.gid
    }

    pub fn gid_value(&self) -> Gid {
        self.gid_value.clone()
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn progress(&self) -> &str {
        &self.progress
    }

    pub fn speed(&self) -> &str {
        &self.speed
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
    status: String,
    progress: String,
    speeds: String,
    totals: String,
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

    pub fn status(&self) -> &str {
        &self.status
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

    pub fn files(&self) -> &[String] {
        &self.files
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    add: AddState,
    connection: ConnectionState,
    settings: SettingsState,
    stats: StatsState,
    downloads: DownloadsState,
    actions: ActionsState,
    selection: SelectionState,
}

impl State {
    pub fn initial() -> Self {
        let applied_settings = Settings::default();
        let draft = SettingsDraft::from_settings(&applied_settings);

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
                open: false,
                feedback: None,
            },
            stats: StatsState { global: None },
            downloads: DownloadsState {
                generation: 0,
                items: Vec::new(),
                filter: DownloadFilter::All,
                refresh_state: RefreshState::NeverRefreshed,
                feedback: None,
            },
            actions: ActionsState {
                generation: 0,
                pending: None,
                feedback: None,
            },
            selection: SelectionState { selected_gid: None },
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

    pub fn add_feedback(&self) -> Option<&str> {
        self.add.feedback.as_deref()
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

    pub fn settings_feedback(&self) -> Option<&str> {
        self.settings.feedback.as_deref()
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
            "Active {} | Waiting {} | Stopped {}",
            stats.active_downloads(),
            stats.waiting_downloads(),
            stats.stopped_downloads()
        )
    }

    pub fn refresh_state(&self) -> RefreshState {
        self.downloads.refresh_state
    }

    pub fn refresh_state_text(&self) -> &'static str {
        match self.downloads.refresh_state {
            RefreshState::NeverRefreshed => "Never refreshed",
            RefreshState::Refreshing => "Refreshing",
            RefreshState::Fresh => "Fresh",
            RefreshState::Stale => "Stale",
        }
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
            .items
            .iter()
            .filter(|item| filter.matches(item))
            .count()
    }

    pub fn download_rows(&self) -> Vec<DownloadRowView> {
        self.downloads
            .items
            .iter()
            .filter(|item| self.downloads.filter.matches(item))
            .map(|item| {
                download_row_view(item, &self.actions, self.selection.selected_gid.as_ref())
            })
            .collect()
    }

    pub fn selected_download_detail(&self) -> Option<DownloadDetailView> {
        let selected_gid = self.selection.selected_gid.as_ref()?;
        self.downloads
            .items
            .iter()
            .find(|item| item.gid() == selected_gid)
            .map(download_detail_view)
    }

    pub fn detail_empty_text(&self) -> &'static str {
        if self.downloads.items.is_empty() {
            "No download selected."
        } else {
            "Select a download to inspect details."
        }
    }

    pub fn can_purge_stopped(&self) -> bool {
        self.is_connected()
            && self.actions.pending.is_none()
            && self.downloads.items.iter().any(|item| {
                matches!(
                    item.status(),
                    DownloadStatus::Complete | DownloadStatus::Error
                )
            })
    }

    pub fn downloads_empty_text(&self) -> Option<String> {
        if matches!(self.downloads.refresh_state, RefreshState::Refreshing)
            && self.downloads.items.is_empty()
        {
            return Some("Loading downloads...".to_owned());
        }

        if self.downloads.items.is_empty() {
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

    pub(super) fn polling_interval_seconds(&self) -> u16 {
        self.settings.applied.polling_interval_seconds()
    }

    #[cfg(test)]
    pub fn global_stats(&self) -> Option<GlobalStats> {
        self.stats.global
    }

    #[cfg(test)]
    pub fn download_items(&self) -> &[DownloadItem] {
        &self.downloads.items
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
                    self.settings.feedback = Some(error.message().to_owned());
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
                self.connection.settings = Some(settings);
                self.settings.feedback = Some("Connection test succeeded.".to_owned());
                true
            }
            Err(error) => {
                self.connection.status = ConnectionStatus::Failed;
                self.connection.version = None;
                self.connection.settings = None;
                self.clear_snapshot();
                self.settings.feedback = Some(error.display_message().to_owned());
                false
            }
        }
    }

    pub(super) fn begin_downloads_refresh(&mut self) -> Option<(u64, Settings)> {
        let settings = self.connection.settings.clone()?;

        self.downloads.generation += 1;
        self.downloads.refresh_state = RefreshState::Refreshing;
        self.downloads.feedback = None;

        Some((self.downloads.generation, settings))
    }

    pub(super) fn finish_downloads_refresh(
        &mut self,
        generation: u64,
        result: Result<DownloadSnapshot, ClientError>,
    ) {
        if generation != self.downloads.generation {
            return;
        }

        match result {
            Ok(snapshot) => {
                let (global_stats, items) = snapshot.into_parts();
                self.stats.global = Some(global_stats);
                self.downloads.items = items;
                self.clear_missing_selection();
                self.downloads.refresh_state = RefreshState::Fresh;
                self.downloads.feedback = None;
            }
            Err(error) => {
                self.downloads.refresh_state = if self.downloads.items.is_empty() {
                    RefreshState::NeverRefreshed
                } else {
                    RefreshState::Stale
                };
                self.downloads.feedback = Some(error.display_message().to_owned());
            }
        }
    }

    pub(super) fn set_download_filter(&mut self, filter: DownloadFilter) {
        self.downloads.filter = filter;
    }

    pub(super) fn select_download(&mut self, gid: Gid) {
        if self.downloads.items.iter().any(|item| item.gid() == &gid) {
            self.selection.selected_gid = Some(gid);
        }
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
        self.settings.open = true;
        self.settings.feedback = self
            .settings
            .draft
            .endpoint_validation_message()
            .map(str::to_owned);
    }

    pub(super) fn open_add_dialog(&mut self) {
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

    pub(super) fn set_add_input(&mut self, input: String) {
        self.add.input = input;
        self.add.feedback = None;
    }

    pub(super) fn begin_add_uri(&mut self) -> Option<(u64, Settings, String)> {
        let uri = match validate_add_input(&self.add.input) {
            Ok(uri) => uri,
            Err(message) => {
                self.add.feedback = Some(message.to_owned());
                return None;
            }
        };

        let Some(settings) = self.connection.settings.clone() else {
            self.add.feedback = Some("Connect to aria2 before adding a download.".to_owned());
            return None;
        };

        self.add.generation += 1;
        self.add.pending = true;
        self.add.feedback = Some("Adding download...".to_owned());

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
                self.add.feedback = Some("Download added.".to_owned());
                true
            }
            Err(error) => {
                self.add.feedback = Some(error.display_message().to_owned());
                false
            }
        }
    }

    pub(super) fn cancel_settings(&mut self) {
        self.settings.draft.cancel_to(&self.settings.applied);
        self.settings.feedback = None;
        self.settings.open = false;
    }

    pub(super) fn set_draft_endpoint(&mut self, endpoint: String) {
        self.settings.draft.set_endpoint(endpoint);
        self.settings.feedback = self
            .settings
            .draft
            .endpoint_validation_message()
            .map(str::to_owned);
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

    pub(super) fn save_settings(&mut self) {
        match self.settings.draft.apply() {
            Ok(settings) => {
                self.settings.applied = settings;
                self.settings.draft = SettingsDraft::from_settings(&self.settings.applied);
                self.settings.feedback = None;
                self.settings.open = false;
            }
            Err(error) => {
                self.settings.feedback = Some(error.message().to_owned());
                self.settings.open = true;
            }
        }
    }

    fn clear_snapshot(&mut self) {
        self.stats.global = None;
        self.downloads.items.clear();
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
                .items
                .iter()
                .any(|item| item.gid() == gid && matches!(item.status(), DownloadStatus::Active))
    }

    fn can_unpause(&self, gid: &Gid) -> bool {
        self.is_connected()
            && self.actions.pending.is_none()
            && self.downloads.items.iter().any(|item| {
                item.gid() == gid
                    && matches!(
                        item.status(),
                        DownloadStatus::Paused | DownloadStatus::Waiting
                    )
            })
    }

    fn can_remove(&self, gid: &Gid) -> bool {
        self.is_connected()
            && self.actions.pending.is_none()
            && self.downloads.items.iter().any(|item| {
                item.gid() == gid
                    && !matches!(
                        item.status(),
                        DownloadStatus::Complete | DownloadStatus::Removed
                    )
            })
    }

    fn set_item_error(&mut self, gid: &Gid, message: String) {
        if let Some(item) = self
            .downloads
            .items
            .iter_mut()
            .find(|item| item.gid() == gid)
        {
            item.set_command_error(Some(message));
        } else {
            self.actions.feedback = Some(message);
        }
    }

    fn clear_missing_selection(&mut self) {
        let Some(selected_gid) = self.selection.selected_gid.as_ref() else {
            return;
        };

        if !self
            .downloads
            .items
            .iter()
            .any(|item| item.gid() == selected_gid)
        {
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
    feedback: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SettingsState {
    applied: Settings,
    draft: SettingsDraft,
    open: bool,
    feedback: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StatsState {
    global: Option<GlobalStats>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DownloadsState {
    generation: u64,
    items: Vec<DownloadItem>,
    filter: DownloadFilter,
    refresh_state: RefreshState,
    feedback: Option<String>,
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

    DownloadRowView {
        name: download_name(item),
        gid: item.gid().as_str().to_owned(),
        gid_value: item.gid().clone(),
        status: item.status().display_label().to_owned(),
        progress: progress_text(item),
        speed: speed_text(item),
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

fn download_detail_view(item: &DownloadItem) -> DownloadDetailView {
    DownloadDetailView {
        name: download_name(item),
        gid: item.gid().as_str().to_owned(),
        status: item.status().display_label().to_owned(),
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
        error: item.command_error().map(str::to_owned),
    }
}

fn download_name(item: &DownloadItem) -> String {
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

fn progress_text(item: &DownloadItem) -> String {
    if item.total_length_bytes() == 0 {
        return format!("{} / unknown", format_bytes(item.completed_length_bytes()));
    }

    let percentage =
        item.completed_length_bytes() as f64 * 100.0 / item.total_length_bytes() as f64;

    format!(
        "{percentage:.0}% | {} / {}",
        format_bytes(item.completed_length_bytes()),
        format_bytes(item.total_length_bytes())
    )
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

    format!("Down {}", format_speed(download_speed))
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
