use crate::aria2::client::ConnectionTest;
use crate::aria2::domain::{DownloadItem, DownloadSnapshot, DownloadStatus, GlobalStats};
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
    status: String,
    progress: String,
    speed: String,
}

impl DownloadRowView {
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

    pub fn speed(&self) -> &str {
        &self.speed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    connection: ConnectionState,
    settings: SettingsState,
    stats: StatsState,
    downloads: DownloadsState,
}

impl State {
    pub fn initial() -> Self {
        let applied_settings = Settings::default();
        let draft = SettingsDraft::from_settings(&applied_settings);

        Self {
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
        }
    }

    pub fn connection_status(&self) -> ConnectionStatus {
        self.connection.status
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
        self.downloads.feedback.as_deref()
    }

    pub fn selected_filter(&self) -> DownloadFilter {
        self.downloads.filter
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
            .map(download_row_view)
            .collect()
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

    pub(super) fn open_settings(&mut self) {
        self.settings.open = true;
        self.settings.feedback = self
            .settings
            .draft
            .endpoint_validation_message()
            .map(str::to_owned);
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

fn connection_label(status: ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::Offline => "Offline",
        ConnectionStatus::Testing => "Testing...",
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Failed => "Connection failed",
    }
}

fn download_row_view(item: &DownloadItem) -> DownloadRowView {
    DownloadRowView {
        name: download_name(item),
        gid: item.gid().as_str().to_owned(),
        status: item.status().display_label().to_owned(),
        progress: progress_text(item),
        speed: speed_text(item),
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
