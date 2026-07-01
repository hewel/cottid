use crate::aria2::client::ConnectionTest;
use crate::aria2::domain::GlobalStats;
use crate::aria2::errors::ClientError;
use crate::config::{RpcAuthDraft, Settings, SettingsDraft};
use crate::util::format::format_speed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Offline,
    Testing,
    Connected,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    connection: ConnectionState,
    settings: SettingsState,
    stats: StatsState,
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
            },
            settings: SettingsState {
                applied: applied_settings,
                draft,
                open: false,
                feedback: None,
            },
            stats: StatsState {
                generation: 0,
                global: None,
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

    pub fn stats_feedback(&self) -> Option<&str> {
        self.stats.feedback.as_deref()
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

    #[cfg(test)]
    pub fn global_stats(&self) -> Option<GlobalStats> {
        self.stats.global
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
        self.stats.global = None;
        self.stats.feedback = None;
        self.settings.feedback = None;

        Some((self.connection.generation, settings))
    }

    pub(super) fn finish_connection_test(
        &mut self,
        generation: u64,
        result: Result<ConnectionTest, ClientError>,
    ) -> bool {
        if generation != self.connection.generation {
            return false;
        }

        match result {
            Ok(result) => {
                self.connection.status = ConnectionStatus::Connected;
                self.connection.version = Some(result.version().version().to_owned());
                self.settings.feedback = Some("Connection test succeeded.".to_owned());
                true
            }
            Err(error) => {
                self.connection.status = ConnectionStatus::Failed;
                self.connection.version = None;
                self.stats.global = None;
                self.stats.feedback = None;
                self.settings.feedback = Some(error.display_message().to_owned());
                false
            }
        }
    }

    pub(super) fn begin_stats_refresh(&mut self) -> u64 {
        self.stats.generation += 1;
        self.stats.feedback = None;
        self.stats.generation
    }

    pub(super) fn finish_stats_refresh(
        &mut self,
        generation: u64,
        result: Result<GlobalStats, ClientError>,
    ) {
        if generation != self.stats.generation {
            return;
        }

        match result {
            Ok(stats) => {
                self.stats.global = Some(stats);
                self.stats.feedback = None;
            }
            Err(error) => {
                self.stats.global = None;
                self.stats.feedback = Some(error.display_message().to_owned());
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConnectionState {
    status: ConnectionStatus,
    generation: u64,
    version: Option<String>,
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
    generation: u64,
    global: Option<GlobalStats>,
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
