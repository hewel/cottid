use crate::config::{RpcAuthDraft, Settings, SettingsDraft};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Offline,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct State {
    connection: ConnectionState,
    settings: SettingsState,
}

impl State {
    pub fn initial() -> Self {
        let applied_settings = Settings::default();
        let draft = SettingsDraft::from_settings(&applied_settings);

        Self {
            connection: ConnectionState {
                status: ConnectionStatus::Offline,
            },
            settings: SettingsState {
                applied: applied_settings,
                draft,
                open: false,
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

    pub fn status_text(&self) -> String {
        format!(
            "{} | {}",
            connection_label(self.connection.status),
            self.settings.applied.auth().display_label()
        )
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SettingsState {
    applied: Settings,
    draft: SettingsDraft,
    open: bool,
    feedback: Option<String>,
}

fn connection_label(status: ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::Offline => "Offline",
    }
}
