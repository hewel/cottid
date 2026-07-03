use crate::app::state::DownloadFilter;
use crate::aria2::client::ConnectionTest;
use crate::aria2::domain::DownloadSnapshot;
use crate::aria2::domain::Gid;
use crate::aria2::domain::RuntimeGlobalOptions;
use crate::aria2::errors::ClientError;
use crate::aria2::notifications::Aria2Notification;
use crate::aria2::websocket::WebSocketEvent;
use crate::config::Settings;
use crate::config::ThemePreference;
use crate::ui::overlay::PopoverId;
use crate::ui::widgets::tree_list::TreeMessage;

pub const ADD_URI_INPUT_ID: &str = "add-uri-input";
pub const SETTINGS_ENDPOINT_INPUT_ID: &str = "settings-endpoint-input";
pub const SETTINGS_SECRET_INPUT_ID: &str = "settings-secret-input";
pub const SETTINGS_POLLING_INTERVAL_INPUT_ID: &str = "settings-polling-interval-input";
pub const SETTINGS_NEW_DOWNLOAD_DIRECTORY_INPUT_ID: &str = "settings-new-download-directory-input";
pub const SETTINGS_NEW_DOWNLOAD_OUTPUT_INPUT_ID: &str = "settings-new-download-output-input";
pub const SETTINGS_NEW_DOWNLOAD_DOWNLOAD_LIMIT_INPUT_ID: &str =
    "settings-new-download-download-limit-input";
pub const SETTINGS_NEW_DOWNLOAD_UPLOAD_LIMIT_INPUT_ID: &str =
    "settings-new-download-upload-limit-input";
pub const SETTINGS_RUNTIME_MAX_CONCURRENT_INPUT_ID: &str = "settings-runtime-max-concurrent-input";
pub const SETTINGS_RUNTIME_DOWNLOAD_LIMIT_INPUT_ID: &str = "settings-runtime-download-limit-input";
pub const SETTINGS_RUNTIME_UPLOAD_LIMIT_INPUT_ID: &str = "settings-runtime-upload-limit-input";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Add(AddMessage),
    Action(ActionMessage),
    Connection(ConnectionMessage),
    Downloads(DownloadsMessage),
    ModalCancel,
    TogglePopover(PopoverId),
    ClosePopover,
    Selection(SelectionMessage),
    Tree(TreeMessage),
    Toolbar(ToolbarMessage),
    Settings(SettingsMessage),
    WebSocket(WebSocketMessage),
    FocusTextInput(TextInputFocusTarget),
    WindowResized { width: u32, height: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInputFocusTarget {
    AddUri,
    SettingsEndpoint,
    SettingsSecret,
    SettingsPollingInterval,
    SettingsNewDownloadDirectory,
    SettingsNewDownloadOutput,
    SettingsNewDownloadDownloadLimit,
    SettingsNewDownloadUploadLimit,
    SettingsRuntimeMaxConcurrent,
    SettingsRuntimeDownloadLimit,
    SettingsRuntimeUploadLimit,
}

impl TextInputFocusTarget {
    pub fn id(self) -> iced::widget::Id {
        iced::widget::Id::new(self.id_value())
    }

    pub fn id_value(self) -> &'static str {
        match self {
            Self::AddUri => ADD_URI_INPUT_ID,
            Self::SettingsEndpoint => SETTINGS_ENDPOINT_INPUT_ID,
            Self::SettingsSecret => SETTINGS_SECRET_INPUT_ID,
            Self::SettingsPollingInterval => SETTINGS_POLLING_INTERVAL_INPUT_ID,
            Self::SettingsNewDownloadDirectory => SETTINGS_NEW_DOWNLOAD_DIRECTORY_INPUT_ID,
            Self::SettingsNewDownloadOutput => SETTINGS_NEW_DOWNLOAD_OUTPUT_INPUT_ID,
            Self::SettingsNewDownloadDownloadLimit => SETTINGS_NEW_DOWNLOAD_DOWNLOAD_LIMIT_INPUT_ID,
            Self::SettingsNewDownloadUploadLimit => SETTINGS_NEW_DOWNLOAD_UPLOAD_LIMIT_INPUT_ID,
            Self::SettingsRuntimeMaxConcurrent => SETTINGS_RUNTIME_MAX_CONCURRENT_INPUT_ID,
            Self::SettingsRuntimeDownloadLimit => SETTINGS_RUNTIME_DOWNLOAD_LIMIT_INPUT_ID,
            Self::SettingsRuntimeUploadLimit => SETTINGS_RUNTIME_UPLOAD_LIMIT_INPUT_ID,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionMessage {
    Select(Gid),
    Clear,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionMessage {
    Pause(Gid),
    Unpause(Gid),
    Remove(Gid),
    PurgeStopped,
    ConfirmPending,
    CancelPending,
    Finished {
        generation: u64,
        target: ActionTarget,
        result: Result<(), ClientError>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionTarget {
    Download(Gid),
    PurgeStopped,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddMessage {
    Open,
    Cancel,
    InputChanged(String),
    OutputFilenameChanged(String),
    MaxDownloadLimitChanged(String),
    MaxUploadLimitChanged(String),
    Submit,
    SubmitFinished {
        generation: u64,
        result: Result<Gid, ClientError>,
    },
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
    RefreshTick,
    RefreshRequested,
    #[allow(dead_code, reason = "reserved for future WebSocket invalidation")]
    Invalidated(RefreshInvalidation),
    FilterChanged(DownloadFilter),
    RefreshFinished {
        generation: u64,
        result: Result<DownloadSnapshot, ClientError>,
    },
}

#[allow(dead_code, reason = "reserved for future WebSocket invalidation")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefreshInvalidation {
    Active,
    Waiting,
    Stopped,
    Selected,
    All,
    Aria2Notification(Aria2Notification),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolbarMessage {
    OpenSettings,
    ThemePreferenceSelected(ThemePreference),
    CycleThemePreference,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsMessage {
    Cancel,
    Save,
    SavePlaintextFallback,
    KeepSecretSessionOnly,
    EndpointChanged(String),
    SecretChanged(String),
    PollingIntervalChanged(String),
    NewDownloadDirectoryChanged(String),
    NewDownloadOutputFilenameChanged(String),
    NewDownloadMaxDownloadLimitChanged(String),
    NewDownloadMaxUploadLimitChanged(String),
    RuntimeMaxConcurrentDownloadsChanged(String),
    RuntimeMaxOverallDownloadLimitChanged(String),
    RuntimeMaxOverallUploadLimitChanged(String),
    RuntimeGlobalOptionsFetched {
        generation: u64,
        settings: Settings,
        result: Result<RuntimeGlobalOptions, ClientError>,
    },
    RuntimeGlobalOptionsSaved {
        generation: u64,
        settings: Settings,
        result: Result<(), ClientError>,
    },
    ConfirmDestructiveActionsChanged(bool),
    NotifyDownloadOutcomesChanged(bool),
    WebSocketEnabledChanged(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketMessage {
    Event(WebSocketEvent),
}
