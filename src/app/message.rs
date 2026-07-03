use crate::app::state::DownloadFilter;
use crate::aria2::client::ConnectionTest;
use crate::aria2::domain::DownloadSnapshot;
use crate::aria2::domain::Gid;
use crate::aria2::errors::ClientError;
use crate::aria2::notifications::Aria2Notification;
use crate::config::RpcAuthDraft;
use crate::config::Settings;
use crate::config::ThemePreference;
use crate::ui::overlay::PopoverId;
use crate::ui::widgets::tree_list::TreeMessage;

pub const ADD_URI_INPUT_ID: &str = "add-uri-input";
pub const SETTINGS_ENDPOINT_INPUT_ID: &str = "settings-endpoint-input";
pub const SETTINGS_SECRET_INPUT_ID: &str = "settings-secret-input";
pub const SETTINGS_POLLING_INTERVAL_INPUT_ID: &str = "settings-polling-interval-input";

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
    FocusTextInput(TextInputFocusTarget),
    WindowResized { width: u32, height: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextInputFocusTarget {
    AddUri,
    SettingsEndpoint,
    SettingsSecret,
    SettingsPollingInterval,
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
    AuthChanged(RpcAuthDraft),
    SecretChanged(String),
    PollingIntervalChanged(String),
    ThemePreferenceChanged(ThemePreference),
}
