use std::path::PathBuf;

use crate::app::{ActionMessage, ActionTarget};
use crate::aria2::client::ConnectionTest;
use crate::aria2::domain::{DownloadItem, DownloadSnapshot, DownloadStatus, Gid, GlobalStats};
use crate::aria2::errors::ClientError;
use crate::config::{
    AuthStorage, ConfigLoad, PersistedConfig, RpcAuthDraft, Settings, SettingsDraft,
    SystemTokenStore, default_config_path, load_config, save_config_with_token_store,
};
use crate::util::format::{format_bytes, format_count, format_eta, format_progress, format_speed};

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

    pub fn config_value(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Active => "active",
            Self::Waiting => "waiting",
            Self::Paused => "paused",
            Self::Complete => "complete",
            Self::Error => "error",
        }
    }

    pub fn from_config_value(value: &str) -> Self {
        match value {
            "active" => Self::Active,
            "waiting" => Self::Waiting,
            "paused" => Self::Paused,
            "complete" => Self::Complete,
            "error" => Self::Error,
            _ => Self::All,
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
        let feedback = loaded.feedback().map(str::to_owned);
        Self::from_persisted_config(loaded.into_config(), config_path, feedback)
    }

    fn from_persisted_config(
        config: PersistedConfig,
        config_path: Option<PathBuf>,
        feedback: Option<String>,
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
                open: false,
                feedback,
                config_path,
                auth_storage: config.auth_storage(),
                pending_plaintext_fallback: None,
            },
            stats: StatsState { global: None },
            downloads: DownloadsState {
                generation: 0,
                items: Vec::new(),
                filter: selected_filter,
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
                    self.settings.feedback = Some("Connection test succeeded.".to_owned());
                }
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
        self.persist_config(None, None);
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
        if let Some(pending) = self.settings.pending_plaintext_fallback.take() {
            self.settings.applied = pending.previous_settings;
            self.settings.draft = SettingsDraft::from_settings(&self.settings.applied);
            self.settings.auth_storage = pending.previous_auth_storage;
            if self.connection.settings.as_ref() == Some(&pending.settings) {
                self.connection.status = ConnectionStatus::Offline;
                self.connection.version = None;
                self.connection.settings = None;
                self.clear_snapshot();
            }
        }
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
                let previous_endpoint = Some(self.settings.applied.endpoint().to_owned());
                self.commit_settings(settings, previous_endpoint, true, "Settings saved.");
            }
            Err(error) => {
                self.settings.feedback = Some(error.message().to_owned());
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
        self.settings.pending_plaintext_fallback = None;
        self.settings.auth_storage = AuthStorage::PlaintextFallback;
        self.persist_config_with_auth_storage(
            AuthStorage::PlaintextFallback,
            pending.previous_endpoint,
            None,
        );
        self.settings.feedback = Some(pending.success_feedback.to_owned());
        self.settings.open = !pending.close_on_success;
    }

    pub(super) fn keep_secret_session_only(&mut self) {
        let Some(pending) = self.settings.pending_plaintext_fallback.clone() else {
            return;
        };

        self.settings.applied = pending.settings.clone();
        self.settings.draft = SettingsDraft::from_settings(&self.settings.applied);
        self.settings.pending_plaintext_fallback = None;
        self.settings.auth_storage = AuthStorage::SessionOnly;
        self.persist_config_with_auth_storage(
            AuthStorage::SessionOnly,
            pending.previous_endpoint,
            None,
        );
        self.settings.feedback =
            Some("Settings saved. Token will be required again next launch.".to_owned());
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
        self.settings.applied = settings;
        self.settings.draft = SettingsDraft::from_settings(&self.settings.applied);
        self.settings.auth_storage = self.next_auth_storage();
        self.settings.pending_plaintext_fallback = None;
        self.settings.feedback = None;
        self.settings.open = !close_on_success;

        let persisted = self.persist_config(
            previous_endpoint.clone(),
            Some((previous_settings, previous_auth_storage)),
        );
        if persisted {
            self.settings.feedback = Some(success_feedback.to_owned());
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
        rollback: Option<(Settings, AuthStorage)>,
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
        rollback: Option<(Settings, AuthStorage)>,
    ) -> bool {
        let config = PersistedConfig::new(
            self.settings.applied.clone(),
            self.downloads.filter.config_value(),
        );
        let config = PersistedConfig::with_auth_storage(
            self.settings.applied.clone(),
            config.selected_filter(),
            auth_storage,
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
                        |(settings, _)| settings.clone(),
                    ),
                    previous_auth_storage: rollback
                        .as_ref()
                        .map_or(self.settings.auth_storage, |(_, auth_storage)| {
                            *auth_storage
                        }),
                    previous_endpoint,
                    close_on_success: false,
                    success_feedback: "Settings saved.",
                });
                self.settings.feedback = Some(error.message().to_owned());
                self.settings.open = true;
                return false;
            }

            self.settings.feedback = Some(error.message().to_owned());
            return false;
        }

        true
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
    config_path: Option<PathBuf>,
    auth_storage: AuthStorage,
    pending_plaintext_fallback: Option<PendingSettingsSave>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingSettingsSave {
    settings: Settings,
    previous_settings: Settings,
    previous_auth_storage: AuthStorage,
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
    format_progress(item.completed_length_bytes(), item.total_length_bytes())
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
