use crate::aria2::client::ConnectionTest;
use crate::aria2::domain::GlobalStats;
use crate::aria2::errors::ClientError;
use crate::config::RpcAuthDraft;
use crate::config::Settings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Connection(ConnectionMessage),
    Stats(StatsMessage),
    Toolbar(ToolbarMessage),
    Settings(SettingsMessage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionMessage {
    TestRequested,
    TestFinished {
        generation: u64,
        settings: Settings,
        result: Result<ConnectionTest, ClientError>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatsMessage {
    RefreshFinished {
        generation: u64,
        result: Result<GlobalStats, ClientError>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolbarMessage {
    OpenSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsMessage {
    Cancel,
    Save,
    EndpointChanged(String),
    AuthChanged(RpcAuthDraft),
    SecretChanged(String),
    PollingIntervalChanged(String),
}
