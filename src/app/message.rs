use crate::aria2::client::ConnectionTest;
use crate::aria2::errors::ClientError;
use crate::config::RpcAuthDraft;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Connection(ConnectionMessage),
    Toolbar(ToolbarMessage),
    Settings(SettingsMessage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionMessage {
    TestRequested,
    TestFinished {
        generation: u64,
        result: Result<ConnectionTest, ClientError>,
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
