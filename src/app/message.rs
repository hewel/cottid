#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Toolbar(ToolbarMessage),
    Settings(SettingsMessage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolbarMessage {
    OpenSettings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsMessage {
    Close,
}
