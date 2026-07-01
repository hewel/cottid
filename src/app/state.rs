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
        Self {
            connection: ConnectionState {
                status: ConnectionStatus::Offline,
            },
            settings: SettingsState {
                ready: true,
                open: false,
            },
        }
    }

    pub fn connection_status(&self) -> ConnectionStatus {
        self.connection.status
    }

    pub fn is_settings_ready(&self) -> bool {
        self.settings.ready
    }

    pub fn is_settings_open(&self) -> bool {
        self.settings.open
    }

    pub(super) fn open_settings(&mut self) {
        self.settings.open = true;
    }

    pub(super) fn close_settings(&mut self) {
        self.settings.open = false;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConnectionState {
    status: ConnectionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SettingsState {
    ready: bool,
    open: bool,
}
