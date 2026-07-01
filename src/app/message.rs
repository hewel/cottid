use crate::app::state::DownloadFilter;
use crate::aria2::client::ConnectionTest;
use crate::aria2::domain::DownloadSnapshot;
use crate::aria2::errors::ClientError;
use crate::config::RpcAuthDraft;
use crate::config::Settings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Connection(ConnectionMessage),
    Downloads(DownloadsMessage),
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
pub enum DownloadsMessage {
    RefreshRequested,
    FilterChanged(DownloadFilter),
    RefreshFinished {
        generation: u64,
        result: Result<DownloadSnapshot, ClientError>,
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
