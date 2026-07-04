mod message;
mod scheduler;
mod state;
mod subscriptions;
mod update;

use iced::{Element, Task, Theme};

use crate::config::ThemePreference;
use crate::ui::tokens::Mode;

pub use message::{
    ActionMessage, ActionTarget, AddMessage, ConnectionMessage, DaemonMessage, DownloadsMessage,
    Message, RefreshInvalidation, SelectionMessage, SettingsMessage, TextInputFocusTarget,
    ToolbarMessage, WebSocketMessage,
};
#[cfg(test)]
pub use state::DaemonStatus;
#[cfg(test)]
pub use state::NotificationOutcome;
pub use state::{
    ConnectionStatus, DownloadDetailView, DownloadFilter, DownloadRowTrailing, DownloadRowView,
    FeedbackTone, FileIcon, FormFeedback, PendingActionConfirmation, RefreshState, State,
};

pub fn run() -> iced::Result {
    iced::application(boot, update, view)
        .title("Cottid")
        .subscription(subscription)
        .theme(theme)
        .run()
}

fn boot() -> (State, Task<Message>) {
    boot_with(State::load())
}

fn boot_with(mut state: State) -> (State, Task<Message>) {
    let task = update::start_boot_connection(&mut state);
    (state, task)
}

#[cfg(test)]
fn boot_from_path(config_path: std::path::PathBuf) -> (State, Task<Message>) {
    boot_with(State::load_from_path(config_path))
}

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    update::update(state, message)
}

pub fn subscription(state: &State) -> iced::Subscription<Message> {
    subscriptions::subscription(state)
}

pub fn view(state: &State) -> Element<'_, Message> {
    crate::ui::shell::view(state)
}

pub fn theme(state: &State) -> Option<Theme> {
    match state.theme_preference() {
        ThemePreference::System => None,
        ThemePreference::Light => Some(crate::ui::theme::iced_theme_for_mode(Mode::Light)),
        ThemePreference::Dark => Some(crate::ui::theme::iced_theme_for_mode(Mode::Dark)),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::aria2::client::ConnectionTest;
    use crate::aria2::domain::{
        DownloadDetail, DownloadFile, DownloadItem, DownloadSnapshot, DownloadStatus, Gid,
        GlobalStats, RuntimeGlobalOptions, TorrentDetail, VersionInfo,
    };
    use crate::aria2::errors::ClientError;
    use crate::aria2::notifications::Aria2Notification;
    use crate::aria2::websocket::WebSocketEvent;
    use crate::config::{DaemonMode, PersistedConfig, Secret, Settings, ThemePreference};
    use crate::daemon::{
        DaemonManager, ManagedDaemonStart, ManagedRuntimeConfig,
        error::{DaemonError, DaemonErrorKind},
        paths::ManagedDaemonPaths,
    };
    use crate::ui::overlay::PopoverId;
    use crate::ui::widgets::tree_list::TreeMessage;

    use super::{
        ActionMessage, ActionTarget, AddMessage, ConnectionMessage, ConnectionStatus,
        DaemonMessage, DaemonStatus, DownloadFilter, DownloadRowTrailing, DownloadsMessage,
        FeedbackTone, FileIcon, Message, NotificationOutcome, PendingActionConfirmation,
        RefreshInvalidation, RefreshState, SelectionMessage, SettingsMessage, State,
        TextInputFocusTarget, ToolbarMessage, WebSocketMessage,
    };

    #[test]
    fn text_input_focus_targets_use_stable_widget_ids() {
        let ids = [
            TextInputFocusTarget::AddUri.id_value(),
            TextInputFocusTarget::SettingsEndpoint.id_value(),
            TextInputFocusTarget::SettingsSecret.id_value(),
            TextInputFocusTarget::SettingsPollingInterval.id_value(),
            TextInputFocusTarget::SettingsNewDownloadDirectory.id_value(),
            TextInputFocusTarget::SettingsNewDownloadOutput.id_value(),
            TextInputFocusTarget::SettingsNewDownloadDownloadLimit.id_value(),
            TextInputFocusTarget::SettingsNewDownloadUploadLimit.id_value(),
            TextInputFocusTarget::SettingsRuntimeMaxConcurrent.id_value(),
            TextInputFocusTarget::SettingsRuntimeDownloadLimit.id_value(),
            TextInputFocusTarget::SettingsRuntimeUploadLimit.id_value(),
        ];
        assert_eq!(
            ids,
            [
                "add-uri-input",
                "settings-endpoint-input",
                "settings-secret-input",
                "settings-polling-interval-input",
                "settings-new-download-directory-input",
                "settings-new-download-output-input",
                "settings-new-download-download-limit-input",
                "settings-new-download-upload-limit-input",
                "settings-runtime-max-concurrent-input",
                "settings-runtime-download-limit-input",
                "settings-runtime-upload-limit-input",
            ]
        );
        assert!(ids.iter().all(|id| {
            !id.contains("profile")
                && !id.contains("save-session")
                && !id.contains("browser-notification")
        }));
    }

    #[test]
    fn starts_offline_and_settings_ready() {
        let state = State::initial();

        assert_eq!(state.daemon_mode(), DaemonMode::Managed);
        assert_eq!(state.connection_status(), ConnectionStatus::Offline);
        assert!(state.is_settings_ready());
    }

    #[test]
    fn boot_waits_for_managed_startup_from_default_config() {
        let path = temp_config_path("boot-auto-connect");

        let (state, _task) = super::boot_from_path(path);

        assert_eq!(state.daemon_status(), DaemonStatus::Starting);
        assert_eq!(state.connection_status(), ConnectionStatus::Offline);
        assert_eq!(state.refresh_state(), RefreshState::NeverRefreshed);
    }

    #[test]
    fn managed_daemon_readiness_connects_and_starts_initial_refresh() {
        let path = temp_config_path("managed-ready");
        let (mut state, _task) = super::boot_from_path(path);
        let runtime = ManagedRuntimeConfig::new(68_01, Secret::session("managed-secret"), 2, true)
            .expect("runtime config");
        let manager = DaemonManager::test(
            runtime,
            ManagedDaemonPaths::from_root(temp_config_path("managed-root")),
        );
        let started = ManagedDaemonStart::test(
            manager,
            ConnectionTest::new(VersionInfo::new("1.37.0", Vec::new())),
        );

        let _task = super::update(
            &mut state,
            Message::Daemon(DaemonMessage::StartFinished {
                generation: 1,
                result: Ok(started),
            }),
        );

        assert_eq!(state.daemon_status(), DaemonStatus::Running);
        assert_eq!(state.connection_status(), ConnectionStatus::Connected);
        assert_eq!(state.connected_version(), Some("1.37.0"));
        assert_eq!(state.refresh_state(), RefreshState::Refreshing);
    }

    #[test]
    fn managed_daemon_start_failure_records_daemon_error_without_refresh() {
        let path = temp_config_path("managed-failure");
        let (mut state, _task) = super::boot_from_path(path);

        let _task = super::update(
            &mut state,
            Message::Daemon(DaemonMessage::StartFinished {
                generation: 1,
                result: Err(DaemonError::new(
                    DaemonErrorKind::BinaryNotFound,
                    "token:managed-secret",
                )),
            }),
        );

        assert_eq!(state.daemon_status(), DaemonStatus::Failed);
        assert_eq!(state.connection_status(), ConnectionStatus::Failed);
        assert_eq!(
            state.daemon_error().map(DaemonError::kind),
            Some(DaemonErrorKind::BinaryNotFound)
        );
        assert_eq!(state.refresh_state(), RefreshState::NeverRefreshed);
    }

    #[test]
    fn boot_starts_connection_test_from_external_config() {
        let path = temp_config_path("boot-external-auto-connect");
        let config = PersistedConfig::with_auth_storage(
            Settings::default(),
            "active",
            crate::config::AuthStorage::None,
        );
        crate::config::save_config_without_token_store(&path, &config).expect("config saves");

        let (state, _task) = super::boot_from_path(path);

        assert_eq!(state.daemon_mode(), DaemonMode::External);
        assert_eq!(state.connection_status(), ConnectionStatus::Testing);
    }

    #[test]
    fn boot_connection_success_triggers_initial_refresh() {
        let path = temp_config_path("boot-refresh");
        let config = PersistedConfig::with_auth_storage(
            Settings::default(),
            "active",
            crate::config::AuthStorage::None,
        );
        crate::config::save_config_without_token_store(&path, &config).expect("config saves");
        let (mut state, _task) = super::boot_from_path(path);

        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestFinished {
                generation: 1,
                settings: Settings::default(),
                result: Ok(ConnectionTest::new(VersionInfo::new("1.37.0", Vec::new()))),
            }),
        );

        assert_eq!(state.connection_status(), ConnectionStatus::Connected);
        assert_eq!(state.refresh_state(), RefreshState::Refreshing);
    }

    #[test]
    fn toolbar_message_opens_settings_without_changing_connection() {
        let mut state = State::initial();

        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));

        assert!(state.is_settings_open());
        assert_eq!(state.connection_status(), ConnectionStatus::Offline);
    }

    #[test]
    fn add_dialog_opens_and_cancels_without_changing_downloads() {
        let mut state = State::initial();

        let _task = super::update(&mut state, Message::Add(AddMessage::Open));
        assert!(state.is_add_open());

        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "https://example.test/file".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Add(AddMessage::Cancel));

        assert!(!state.is_add_open());
        assert_eq!(state.add_input(), "");
        assert_eq!(state.download_items().len(), 0);
    }

    #[test]
    fn settings_message_cancels_settings() {
        let mut state = State::initial();
        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));

        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Cancel));

        assert!(!state.is_settings_open());
    }

    #[test]
    fn opening_add_closes_settings() {
        let mut state = State::initial();
        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));

        let _task = super::update(&mut state, Message::Add(AddMessage::Open));

        assert!(state.is_add_open());
        assert!(!state.is_settings_open());
    }

    #[test]
    fn opening_settings_closes_idle_add() {
        let mut state = State::initial();
        let _task = super::update(&mut state, Message::Add(AddMessage::Open));

        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));

        assert!(state.is_settings_open());
        assert!(!state.is_add_open());
    }

    #[test]
    fn opening_settings_does_not_close_pending_add() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(&mut state, Message::Add(AddMessage::Open));
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "https://example.test/file".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Add(AddMessage::Submit));

        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));

        assert!(state.is_add_open());
        assert!(state.is_add_pending());
        assert!(!state.is_settings_open());
    }

    #[test]
    fn modal_cancel_closes_add() {
        let mut state = State::initial();
        let _task = super::update(&mut state, Message::Add(AddMessage::Open));

        let _task = super::update(&mut state, Message::ModalCancel);

        assert!(!state.is_add_open());
    }

    #[test]
    fn modal_cancel_closes_settings() {
        let mut state = State::initial();
        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));

        let _task = super::update(&mut state, Message::ModalCancel);

        assert!(!state.is_settings_open());
    }

    #[test]
    fn modal_cancel_closes_pending_action_confirmation() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Remove(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );

        let _task = super::update(&mut state, Message::ModalCancel);

        assert_eq!(state.pending_action_confirmation(), None);
        assert!(!state.download_rows()[0].pending());
    }

    #[test]
    fn toggle_popover_opens_and_closes_same_popover() {
        let mut state = State::initial();
        let id = PopoverId(1);

        let _task = super::update(&mut state, Message::TogglePopover(id));
        assert!(state.is_popover_open(id));

        let _task = super::update(&mut state, Message::TogglePopover(id));
        assert!(!state.is_popover_open(id));
    }

    #[test]
    fn close_popover_closes_open_popover() {
        let mut state = State::initial();
        let id = PopoverId(1);

        let _task = super::update(&mut state, Message::TogglePopover(id));
        let _task = super::update(&mut state, Message::ClosePopover);

        assert!(!state.is_popover_open(id));
    }

    #[test]
    fn modal_cancel_does_not_close_pending_add() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(&mut state, Message::Add(AddMessage::Open));
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "https://example.test/file".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Add(AddMessage::Submit));

        let _task = super::update(&mut state, Message::ModalCancel);

        assert!(state.is_add_open());
        assert!(state.is_add_pending());
    }

    #[test]
    fn subscription_hook_is_available_before_connection() {
        let state = State::initial();

        let _subscription = super::subscription(&state);
    }

    #[test]
    fn window_resize_updates_compact_layout_state() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::WindowResized {
                width: 720,
                height: 640,
            },
        );

        assert!(state.is_compact_layout());

        let _task = super::update(
            &mut state,
            Message::WindowResized {
                width: 1200,
                height: 800,
            },
        );

        assert!(!state.is_compact_layout());
    }

    #[test]
    fn window_resize_updates_modal_viewport_constraints() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::WindowResized {
                width: 500,
                height: 600,
            },
        );

        assert_eq!(state.modal_max_width(640.0), 450.0);
        assert_eq!(state.modal_max_height(), 450.0);
    }

    #[test]
    fn view_builds_from_normalized_app_state() {
        let mut state = State::initial();

        {
            let _element = super::view(&state);
        }

        let _task = super::update(&mut state, Message::Add(AddMessage::Open));
        {
            let _element = super::view(&state);
        }

        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));

        {
            let _element = super::view(&state);
        }
    }

    #[test]
    fn settings_draft_does_not_change_applied_settings_until_saved() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::EndpointChanged(
                "http://aria2.local:6800/jsonrpc".to_owned(),
            )),
        );

        assert_eq!(state.applied_endpoint(), "http://localhost:6800/jsonrpc");
        assert_eq!(state.draft_endpoint(), "http://aria2.local:6800/jsonrpc");

        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        assert_eq!(state.applied_endpoint(), "http://aria2.local:6800/jsonrpc");
        assert_eq!(state.draft_endpoint(), "http://aria2.local:6800/jsonrpc");
    }

    #[test]
    fn settings_draft_mode_does_not_change_applied_mode_until_saved() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::DaemonModeChanged(DaemonMode::External)),
        );

        assert_eq!(state.daemon_mode(), DaemonMode::Managed);
        assert_eq!(state.draft_daemon_mode(), DaemonMode::External);

        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        assert_eq!(state.daemon_mode(), DaemonMode::External);
        assert_eq!(state.draft_daemon_mode(), DaemonMode::External);
    }

    #[test]
    fn saved_settings_reload_from_config_path() {
        let path = temp_config_path("state-save-load");
        let mut state = State::load_from_path(path.clone());

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::EndpointChanged(
                "http://aria2.local:6800/jsonrpc".to_owned(),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::PollingIntervalChanged("7".to_owned())),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadDirectoryChanged(
                "/downloads".to_owned(),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadOutputFilenameChanged(
                "file.iso".to_owned(),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadMaxDownloadLimitChanged(
                "1024".to_owned(),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadMaxUploadLimitChanged(
                "2048".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        let reloaded = State::load_from_path(path);

        assert_eq!(
            reloaded.applied_endpoint(),
            "http://aria2.local:6800/jsonrpc"
        );
        assert_eq!(reloaded.draft_polling_interval_seconds(), 7);
        assert_eq!(reloaded.draft_new_download_directory(), "/downloads");
        assert_eq!(reloaded.draft_new_download_output_filename(), "file.iso");
        assert_eq!(reloaded.draft_new_download_max_download_limit(), "1024");
        assert_eq!(reloaded.draft_new_download_max_upload_limit(), "2048");
        assert_eq!(reloaded.applied_auth_label(), "No authentication");
    }

    #[test]
    fn saved_new_download_directory_flows_into_add_options() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadDirectoryChanged(
                "/downloads".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));
        let _task = super::update(&mut state, Message::Add(AddMessage::Open));
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "https://example.test/file".to_owned(),
            )),
        );

        let (_generation, _settings, _uri, options) =
            state.begin_add_uri().expect("valid add request");

        let rpc_options = options.into_rpc_options();
        assert_eq!(
            rpc_options.get("dir").map(String::as_str),
            Some("/downloads")
        );
    }

    #[test]
    fn add_dialog_uses_and_overrides_new_download_defaults() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadOutputFilenameChanged(
                "default.iso".to_owned(),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadMaxDownloadLimitChanged(
                "1024".to_owned(),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadMaxUploadLimitChanged(
                "2048".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        let _task = super::update(&mut state, Message::Add(AddMessage::Open));
        assert_eq!(state.add_output_filename(), "default.iso");
        assert_eq!(state.add_max_download_limit(), "1024");
        assert_eq!(state.add_max_upload_limit(), "2048");

        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "https://example.test/file".to_owned(),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::OutputFilenameChanged("override.iso".to_owned())),
        );
        let (_generation, _settings, _uri, options) =
            state.begin_add_uri().expect("valid add request");

        let rpc_options = options.into_rpc_options();
        assert_eq!(
            rpc_options.get("out").map(String::as_str),
            Some("override.iso")
        );
        assert_eq!(
            rpc_options.get("max-download-limit").map(String::as_str),
            Some("1024")
        );
        assert_eq!(
            rpc_options.get("max-upload-limit").map(String::as_str),
            Some("2048")
        );
    }

    #[test]
    fn invalid_add_defaults_block_add_rpc() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(&mut state, Message::Add(AddMessage::Open));
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "https://example.test/file".to_owned(),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::OutputFilenameChanged("bad/name.iso".to_owned())),
        );
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::MaxDownloadLimitChanged("fast".to_owned())),
        );

        assert_eq!(
            state.add_output_filename_validation_message(),
            Some("Output filename must not contain path separators.")
        );
        assert_eq!(
            state.add_max_download_limit_validation_message(),
            Some("Speed limit must be an unsigned integer in bytes per second.")
        );
        assert!(state.begin_add_uri().is_none());
    }

    #[test]
    fn fetched_runtime_directory_updates_displayed_directory_draft() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadDirectoryChanged(
                "/old".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));
        let (generation, settings) = state.begin_runtime_global_options_fetch(Settings::default());

        state.apply_runtime_global_options(
            generation,
            settings,
            Ok(RuntimeGlobalOptions::with_values(
                Some("/daemon".to_owned()),
                Some("5".to_owned()),
                Some("1024".to_owned()),
                Some("2048".to_owned()),
            )),
        );

        assert_eq!(state.draft_new_download_directory(), "/daemon");
        assert_eq!(state.draft_runtime_max_concurrent_downloads(), "5");
        assert_eq!(state.draft_runtime_max_overall_download_limit(), "1024");
        assert_eq!(state.draft_runtime_max_overall_upload_limit(), "2048");
    }

    #[test]
    fn stale_runtime_directory_fetch_is_ignored() {
        let mut state = State::initial();
        connect(&mut state);
        let (stale_generation, stale_settings) =
            state.begin_runtime_global_options_fetch(Settings::default());
        let (current_generation, current_settings) =
            state.begin_runtime_global_options_fetch(Settings::default());

        state.apply_runtime_global_options(
            stale_generation,
            stale_settings,
            Ok(RuntimeGlobalOptions::new(Some("/stale".to_owned()))),
        );
        state.apply_runtime_global_options(
            current_generation,
            current_settings,
            Ok(RuntimeGlobalOptions::new(Some("/current".to_owned()))),
        );

        assert_eq!(state.draft_new_download_directory(), "/current");
    }

    #[test]
    fn stale_runtime_directory_save_result_is_ignored() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadDirectoryChanged(
                "/downloads".to_owned(),
            )),
        );
        let (stale_generation, stale_settings, _options) =
            state.save_settings().expect("runtime save requested");
        let _current = state.begin_runtime_global_options_fetch(Settings::default());

        state.finish_runtime_global_options_save(
            stale_generation,
            stale_settings,
            Err(ClientError::Transport("connection refused".to_owned())),
        );

        assert!(!state.is_settings_open());
        assert_eq!(
            state.settings_feedback().map(|feedback| feedback.message()),
            Some("Settings saved.")
        );
    }

    #[test]
    fn saving_polling_and_directory_for_same_endpoint_updates_runtime_directory() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::PollingIntervalChanged("7".to_owned())),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadDirectoryChanged(
                "/downloads".to_owned(),
            )),
        );

        let (_generation, settings, options) =
            state.save_settings().expect("runtime save requested");

        assert_eq!(settings.endpoint(), "http://localhost:6800/jsonrpc");
        let rpc_options = options.into_rpc_options();
        assert_eq!(
            rpc_options.get("dir").map(String::as_str),
            Some("/downloads")
        );
    }

    #[test]
    fn clearing_new_download_directory_omits_add_options_and_runtime_save() {
        let mut state = State::initial();
        connect(&mut state);
        let (generation, settings) = state.begin_runtime_global_options_fetch(Settings::default());
        state.apply_runtime_global_options(
            generation,
            settings,
            Ok(RuntimeGlobalOptions::new(Some("/daemon".to_owned()))),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NewDownloadDirectoryChanged(String::new())),
        );

        assert!(state.save_settings().is_none());

        let _task = super::update(&mut state, Message::Add(AddMessage::Open));
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "https://example.test/file".to_owned(),
            )),
        );
        let (_generation, _settings, _uri, options) =
            state.begin_add_uri().expect("valid add request");

        assert!(options.into_rpc_options().is_empty());
    }

    #[test]
    fn runtime_quick_controls_save_only_modeled_options() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::RuntimeMaxConcurrentDownloadsChanged(
                "6".to_owned(),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::RuntimeMaxOverallDownloadLimitChanged(
                "4096".to_owned(),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::RuntimeMaxOverallUploadLimitChanged(
                "512".to_owned(),
            )),
        );

        let (_generation, _settings, options) =
            state.save_settings().expect("runtime save requested");
        let rpc_options = options.into_rpc_options();

        assert_eq!(
            rpc_options
                .get("max-concurrent-downloads")
                .map(String::as_str),
            Some("6")
        );
        assert_eq!(
            rpc_options
                .get("max-overall-download-limit")
                .map(String::as_str),
            Some("4096")
        );
        assert_eq!(
            rpc_options
                .get("max-overall-upload-limit")
                .map(String::as_str),
            Some("512")
        );
        assert!(!rpc_options.contains_key("save-session"));
    }

    #[test]
    fn invalid_runtime_quick_controls_block_save() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::RuntimeMaxConcurrentDownloadsChanged(
                "0".to_owned(),
            )),
        );

        assert!(state.save_settings().is_none());
        assert_eq!(
            state.settings_feedback().map(|feedback| feedback.message()),
            Some("Max concurrent downloads must be a positive integer.")
        );
        assert!(state.is_settings_open());
    }

    #[test]
    fn saving_after_endpoint_change_does_not_update_old_daemon_runtime_options() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::EndpointChanged(
                "http://aria2.local:6800/jsonrpc".to_owned(),
            )),
        );

        assert!(state.save_settings().is_none());
    }

    #[test]
    fn successful_settings_connection_test_applies_and_saves_settings() {
        let path = temp_config_path("test-connection-save");
        let mut state = State::load_from_path(path.clone());
        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::DaemonModeChanged(DaemonMode::External)),
        );

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::EndpointChanged(
                "http://aria2.local:6800/jsonrpc".to_owned(),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestRequested),
        );
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestFinished {
                generation: 1,
                settings: Settings::new_without_secret("http://aria2.local:6800/jsonrpc", 2)
                    .expect("settings"),
                result: Ok(ConnectionTest::new(VersionInfo::new("1.37.0", Vec::new()))),
            }),
        );

        let reloaded = State::load_from_path(path);

        assert!(state.is_settings_open());
        assert_eq!(
            state.settings_feedback().map(|feedback| feedback.message()),
            Some("Connection test succeeded and settings saved.")
        );
        assert_eq!(
            state.settings_feedback().map(|feedback| feedback.tone()),
            Some(FeedbackTone::Success)
        );
        assert_eq!(
            reloaded.applied_endpoint(),
            "http://aria2.local:6800/jsonrpc"
        );
    }

    #[test]
    fn selected_filter_persists_as_ui_preference() {
        let path = temp_config_path("filter");
        let mut state = State::load_from_path(path.clone());

        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::FilterChanged(DownloadFilter::Complete)),
        );

        let contents = fs::read_to_string(&path).expect("config written");
        let reloaded = State::load_from_path(path);

        assert!(contents.contains("selected_filter = \"complete\""));
        assert_eq!(reloaded.selected_filter(), DownloadFilter::Complete);
    }

    #[test]
    fn hidden_filter_message_persists_as_visible_sidebar_preference() {
        let path = temp_config_path("hidden-filter");
        let mut state = State::load_from_path(path.clone());

        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::FilterChanged(DownloadFilter::Paused)),
        );

        let contents = fs::read_to_string(&path).expect("config written");
        let reloaded = State::load_from_path(path);

        assert!(contents.contains("selected_filter = \"active\""));
        assert_eq!(reloaded.selected_filter(), DownloadFilter::Active);
    }

    #[test]
    fn hidden_saved_filter_values_remap_to_visible_sidebar_groups() {
        for (saved, expected) in [
            ("all", DownloadFilter::Active),
            ("waiting", DownloadFilter::Active),
            ("paused", DownloadFilter::Active),
            ("error", DownloadFilter::Complete),
            ("unknown", DownloadFilter::Active),
        ] {
            let path = temp_config_path(saved);
            fs::write(
                &path,
                format!(
                    "endpoint=http://aria2.local:6800/jsonrpc\npolling_interval_seconds=5\nselected_filter={saved}\nauth=none\n"
                ),
            )
            .expect("legacy config");

            let state = State::load_from_path(path);

            assert_eq!(state.selected_filter(), expected);
        }
    }

    #[test]
    fn toolbar_theme_preference_applies_and_persists_immediately() {
        let path = temp_config_path("toolbar-theme");
        let mut state = State::load_from_path(path.clone());

        let _task = super::update(
            &mut state,
            Message::Toolbar(ToolbarMessage::ThemePreferenceSelected(
                ThemePreference::Dark,
            )),
        );

        let contents = fs::read_to_string(&path).expect("config written");
        let reloaded = State::load_from_path(path);

        assert_eq!(state.theme_preference(), ThemePreference::Dark);
        assert!(contents.contains("theme = \"dark\""));
        assert_eq!(reloaded.theme_preference(), ThemePreference::Dark);
    }

    #[test]
    fn toolbar_theme_cycle_walks_system_light_dark() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Toolbar(ToolbarMessage::CycleThemePreference),
        );
        assert_eq!(state.theme_preference(), ThemePreference::Light);

        let _task = super::update(
            &mut state,
            Message::Toolbar(ToolbarMessage::CycleThemePreference),
        );
        assert_eq!(state.theme_preference(), ThemePreference::Dark);

        let _task = super::update(
            &mut state,
            Message::Toolbar(ToolbarMessage::CycleThemePreference),
        );
        assert_eq!(state.theme_preference(), ThemePreference::System);
    }

    #[test]
    fn destructive_confirmation_preferences_persist() {
        let path = temp_config_path("destructive-confirmation-preference");
        let mut state = State::load_from_path(path.clone());

        assert!(state.confirm_destructive_actions());
        assert!(!state.notify_download_outcomes());

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::ConfirmDestructiveActionsChanged(false)),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NotifyDownloadOutcomesChanged(true)),
        );

        let contents = fs::read_to_string(&path).expect("config written");
        let reloaded = State::load_from_path(path);

        assert!(contents.contains("confirm_destructive_actions = false"));
        assert!(contents.contains("notify_download_outcomes = true"));
        assert!(!reloaded.confirm_destructive_actions());
        assert!(reloaded.notify_download_outcomes());
    }

    #[test]
    fn system_theme_preference_defers_to_iced_system_theme() {
        let mut state = State::initial();

        assert_eq!(super::theme(&state), None);

        let _task = super::update(
            &mut state,
            Message::Toolbar(ToolbarMessage::ThemePreferenceSelected(
                ThemePreference::Light,
            )),
        );

        let theme = super::theme(&state).expect("explicit light theme");
        assert!(!theme.extended_palette().is_dark);
    }

    #[test]
    fn invalid_config_loads_defaults_with_feedback() {
        let path = temp_config_path("invalid-state");
        fs::write(&path, "endpoint=ftp://bad\n").expect("write invalid config");

        let state = State::load_from_path(path);

        assert_eq!(state.applied_endpoint(), "http://localhost:6800/jsonrpc");
        assert_eq!(
            state.settings_feedback().map(|feedback| feedback.message()),
            Some("Config was invalid; using defaults.")
        );
        assert_eq!(
            state.settings_feedback().map(|feedback| feedback.tone()),
            Some(FeedbackTone::Warning)
        );
    }

    #[test]
    fn secure_token_is_restored_without_writing_plaintext_config() {
        let path = temp_config_path("session-secret");
        let mut state = State::load_from_path(path.clone());

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::SecretChanged("super-secret".to_owned())),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        let contents = fs::read_to_string(&path).expect("config contents");
        let reloaded = State::load_from_path(path);

        assert!(!contents.contains("super-secret"));
        assert_eq!(reloaded.applied_auth_label(), "Token secret");
    }

    #[test]
    fn cancelling_settings_restores_draft_from_applied_settings() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::EndpointChanged(
                "http://aria2.local:6800/jsonrpc".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Cancel));

        assert_eq!(state.applied_endpoint(), "http://localhost:6800/jsonrpc");
        assert_eq!(state.draft_endpoint(), "http://localhost:6800/jsonrpc");
        assert!(!state.is_settings_open());
    }

    #[test]
    fn invalid_endpoint_validation_is_exposed_as_field_status() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::EndpointChanged(
                "ftp://localhost:6800/jsonrpc".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        assert_eq!(
            state.draft_endpoint_validation_message(),
            Some("Endpoint must start with http:// or https://.")
        );
        assert_eq!(state.settings_feedback(), None);
        assert!(state.is_settings_open());
        assert_eq!(state.applied_endpoint(), "http://localhost:6800/jsonrpc");
    }

    #[test]
    fn settings_can_choose_session_secret_without_displaying_secret() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::SecretChanged("super-secret".to_owned())),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        assert_eq!(state.applied_auth_label(), "Token secret");
        assert!(!state.status_text().contains("super-secret"));
    }

    #[test]
    fn empty_secret_disables_authentication() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::SecretChanged("super-secret".to_owned())),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::SecretChanged(String::new())),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        assert_eq!(state.applied_auth_label(), "No authentication");
    }

    #[test]
    fn connection_test_result_updates_visible_connection_state() {
        let mut state = State::initial();
        use_external_mode(&mut state);
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestRequested),
        );

        assert_eq!(state.connection_status(), ConnectionStatus::Testing);

        let result = Ok(ConnectionTest::new(VersionInfo::new("1.37.0", Vec::new())));
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestFinished {
                generation: 1,
                settings: Settings::default(),
                result,
            }),
        );

        assert_eq!(state.connection_status(), ConnectionStatus::Connected);
        assert_eq!(state.connected_version(), Some("1.37.0"));
        assert_eq!(state.refresh_state(), RefreshState::Refreshing);
        assert_eq!(
            state.settings_feedback().map(|feedback| feedback.message()),
            Some("Connection test succeeded.")
        );
        assert_eq!(
            state.settings_feedback().map(|feedback| feedback.tone()),
            Some(FeedbackTone::Success)
        );
    }

    #[test]
    fn initial_refresh_failure_keeps_successful_connection_state() {
        let mut state = State::initial();
        use_external_mode(&mut state);
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestRequested),
        );
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestFinished {
                generation: 1,
                settings: Settings::default(),
                result: Ok(ConnectionTest::new(VersionInfo::new("1.37.0", Vec::new()))),
            }),
        );

        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation: 1,
                result: Err(ClientError::Transport("connection refused".to_owned())),
            }),
        );

        assert_eq!(state.connection_status(), ConnectionStatus::Connected);
        assert_eq!(state.connected_version(), Some("1.37.0"));
        assert_eq!(state.refresh_state(), RefreshState::NeverRefreshed);
        assert_eq!(
            state.refresh_feedback(),
            Some("Connection failed. Check the endpoint and secret.")
        );
    }

    #[test]
    fn stale_connection_test_results_are_ignored() {
        let mut state = State::initial();
        use_external_mode(&mut state);
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestRequested),
        );
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestRequested),
        );

        let stale_result = Ok(ConnectionTest::new(VersionInfo::new("1.36.0", Vec::new())));
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestFinished {
                generation: 1,
                settings: Settings::default(),
                result: stale_result,
            }),
        );

        assert_eq!(state.connection_status(), ConnectionStatus::Testing);
        assert_eq!(state.connected_version(), None);

        let current_result = Err(ClientError::Transport("connection refused".to_owned()));
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestFinished {
                generation: 2,
                settings: Settings::default(),
                result: current_result,
            }),
        );

        assert_eq!(state.connection_status(), ConnectionStatus::Failed);
        assert_eq!(
            state.settings_feedback().map(|feedback| feedback.message()),
            Some("Connection failed. Check the endpoint and secret.")
        );
        assert_eq!(
            state.settings_feedback().map(|feedback| feedback.tone()),
            Some(FeedbackTone::Error)
        );
    }

    #[test]
    fn add_submit_exposes_uri_validation_as_field_status_before_rpc() {
        let mut state = State::initial();
        connect(&mut state);

        let _task = super::update(&mut state, Message::Add(AddMessage::Open));
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "ftp://example.test/file".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Add(AddMessage::Submit));

        assert_eq!(
            state.add_input_validation_message(),
            Some("Enter an http, https, or magnet link.")
        );
        assert_eq!(state.add_feedback(), None);
        assert!(!state.is_add_pending());

        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "magnet:?xt=urn:btih:abc".to_owned(),
            )),
        );

        assert!(state.is_add_ready());
        assert_eq!(state.add_input_validation_message(), None);
    }

    #[test]
    fn add_submit_success_keeps_dialog_recoverable_and_triggers_refresh_state() {
        let mut state = State::initial();
        connect(&mut state);

        let _task = super::update(&mut state, Message::Add(AddMessage::Open));
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "https://example.test/file".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Add(AddMessage::Submit));

        assert!(state.is_add_pending());

        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::SubmitFinished {
                generation: 1,
                result: Ok(Gid::new("new-gid").expect("valid gid")),
            }),
        );

        assert!(!state.is_add_pending());
        assert_eq!(state.add_input(), "");
        assert_eq!(
            state.add_feedback().map(|feedback| feedback.message()),
            Some("Download added.")
        );
        assert_eq!(
            state.add_feedback().map(|feedback| feedback.tone()),
            Some(FeedbackTone::Success)
        );
        assert_eq!(state.refresh_state(), RefreshState::Refreshing);
    }

    #[test]
    fn add_submit_error_is_visible_and_allows_retry() {
        let mut state = State::initial();
        connect(&mut state);

        let _task = super::update(&mut state, Message::Add(AddMessage::Open));
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "https://example.test/file".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Add(AddMessage::Submit));
        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::SubmitFinished {
                generation: 1,
                result: Err(ClientError::Rpc {
                    code: 1,
                    message: "bad uri".to_owned(),
                }),
            }),
        );

        assert!(!state.is_add_pending());
        assert_eq!(state.add_input(), "https://example.test/file");
        assert_eq!(
            state.add_feedback().map(|feedback| feedback.message()),
            Some("aria2 returned an RPC error.")
        );
        assert_eq!(
            state.add_feedback().map(|feedback| feedback.tone()),
            Some(FeedbackTone::Error)
        );
        assert!(state.is_add_ready());
    }

    #[test]
    fn row_action_enablement_follows_status_and_pending_state() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![
                download_item("active-gid", DownloadStatus::Active),
                download_item("paused-gid", DownloadStatus::Paused),
                download_item("complete-gid", DownloadStatus::Complete),
            ],
        );

        let rows = state.download_rows();
        let active_row = rows
            .iter()
            .find(|row| row.gid() == "active-gid")
            .expect("active row");
        let paused_row = rows
            .iter()
            .find(|row| row.gid() == "paused-gid")
            .expect("paused row");
        assert!(active_row.can_pause());
        assert!(!active_row.can_unpause());
        assert!(active_row.can_remove());
        assert!(paused_row.can_unpause());
        assert_eq!(state.filter_count(DownloadFilter::Complete), 1);
        assert!(state.can_purge_stopped());

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );

        let rows = state.download_rows();
        let active_row = rows
            .iter()
            .find(|row| row.gid() == "active-gid")
            .expect("active row");
        let paused_row = rows
            .iter()
            .find(|row| row.gid() == "paused-gid")
            .expect("paused row");
        assert!(active_row.pending());
        assert!(!paused_row.can_unpause());
    }

    #[test]
    fn pause_success_does_not_fetch_new_list_immediately() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(Gid::new("active-gid").expect("valid gid")),
                result: Ok(()),
            }),
        );

        assert_eq!(state.refresh_state(), RefreshState::Fresh);
        assert!(!state.download_rows()[0].pending());
        assert_eq!(state.begin_scheduled_downloads_refresh(), None);
    }

    #[test]
    fn remove_success_triggers_snapshot_refresh_without_retrying_command() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::ConfirmDestructiveActionsChanged(false)),
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Remove(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(Gid::new("active-gid").expect("valid gid")),
                result: Ok(()),
            }),
        );

        assert_eq!(state.refresh_state(), RefreshState::Refreshing);
        assert!(state.download_rows().is_empty());
    }

    #[test]
    fn pause_in_flight_ignores_pre_action_missing_refresh_and_shows_pausing() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );
        let (generation, _, _) = state.begin_downloads_refresh().expect("refresh");

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );
        let rows = state.download_rows();
        assert_eq!(
            rows[0].trailing(),
            &DownloadRowTrailing::Status("Pausing".to_owned())
        );

        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation,
                result: Ok(snapshot_with_items(Vec::new())),
            }),
        );

        let rows = state.download_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].trailing(),
            &DownloadRowTrailing::Status("Pausing".to_owned())
        );
    }

    #[test]
    fn pause_transition_survives_several_post_action_missing_refreshes() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );

        for _ in 0..5 {
            let (generation, _, _) = state.begin_downloads_refresh().expect("refresh");
            let _task = super::update(
                &mut state,
                Message::Downloads(DownloadsMessage::RefreshFinished {
                    generation,
                    result: Ok(snapshot_with_items(Vec::new())),
                }),
            );
        }

        let rows = state.download_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].trailing(),
            &DownloadRowTrailing::Status("Pausing".to_owned())
        );
    }

    #[test]
    fn pause_success_shows_paused_in_speed_slot_without_waiting_for_snapshot() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(Gid::new("active-gid").expect("valid gid")),
                result: Ok(()),
            }),
        );

        let rows = state.download_rows();
        assert_eq!(
            rows[0].trailing(),
            &DownloadRowTrailing::Status("Paused".to_owned())
        );
        assert!(rows[0].can_unpause());
    }

    #[test]
    fn stale_active_snapshot_does_not_clear_pause_success_transition() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(Gid::new("active-gid").expect("valid gid")),
                result: Ok(()),
            }),
        );

        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let rows = state.download_rows();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].trailing(),
            &DownloadRowTrailing::Status("Paused".to_owned())
        );
    }

    #[test]
    fn confirmed_paused_snapshot_clears_pause_transition() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(Gid::new("active-gid").expect("valid gid")),
                result: Ok(()),
            }),
        );
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Paused)],
        );
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        assert!(matches!(
            state.download_rows()[0].trailing(),
            DownloadRowTrailing::Speed { .. }
        ));
    }

    #[test]
    fn unpause_success_shows_resuming_in_speed_slot_without_waiting_for_snapshot() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("paused-gid", DownloadStatus::Paused)],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Unpause(
                Gid::new("paused-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(Gid::new("paused-gid").expect("valid gid")),
                result: Ok(()),
            }),
        );

        let rows = state.download_rows();
        assert_eq!(
            rows[0].trailing(),
            &DownloadRowTrailing::Status("Resuming".to_owned())
        );
        assert!(!rows[0].can_unpause());
    }

    #[test]
    fn stale_paused_snapshot_does_not_clear_resume_transition() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("paused-gid", DownloadStatus::Paused)],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Unpause(
                Gid::new("paused-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(Gid::new("paused-gid").expect("valid gid")),
                result: Ok(()),
            }),
        );
        apply_snapshot(
            &mut state,
            vec![download_item("paused-gid", DownloadStatus::Paused)],
        );

        let rows = state.download_rows();
        assert_eq!(
            rows[0].trailing(),
            &DownloadRowTrailing::Status("Resuming".to_owned())
        );
    }

    #[test]
    fn confirmed_active_snapshot_clears_resume_transition() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("paused-gid", DownloadStatus::Paused)],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Unpause(
                Gid::new("paused-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(Gid::new("paused-gid").expect("valid gid")),
                result: Ok(()),
            }),
        );
        apply_snapshot(
            &mut state,
            vec![download_item("paused-gid", DownloadStatus::Active)],
        );

        assert!(matches!(
            state.download_rows()[0].trailing(),
            DownloadRowTrailing::Speed { .. }
        ));
    }

    #[test]
    fn remove_success_hides_row_and_clears_selected_detail() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );
        let gid = Gid::new("active-gid").expect("valid gid");
        let _task = super::update(
            &mut state,
            Message::Selection(SelectionMessage::Select(gid.clone())),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::ConfirmDestructiveActionsChanged(false)),
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Remove(gid.clone())),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(gid),
                result: Ok(()),
            }),
        );

        assert!(state.download_rows().is_empty());
        assert_eq!(state.selected_download_detail(), None);
    }

    #[test]
    fn pause_transition_prunes_after_extended_missing_refresh_grace() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(Gid::new("active-gid").expect("valid gid")),
                result: Ok(()),
            }),
        );
        for _ in 0..6 {
            apply_snapshot(&mut state, Vec::new());
        }

        assert!(state.download_rows().is_empty());
    }

    #[test]
    fn row_action_error_attaches_to_download_and_does_not_refresh() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(Gid::new("active-gid").expect("valid gid")),
                result: Err(ClientError::Rpc {
                    code: 1,
                    message: "cannot pause".to_owned(),
                }),
            }),
        );

        assert_eq!(state.refresh_state(), RefreshState::Fresh);
        assert_eq!(
            state.download_rows()[0].error(),
            Some("aria2 returned an RPC error.")
        );
        assert!(matches!(
            state.download_rows()[0].trailing(),
            DownloadRowTrailing::Speed { .. }
        ));
    }

    #[test]
    fn purge_error_is_global_status_feedback() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("complete-gid", DownloadStatus::Complete)],
        );

        let _task = super::update(&mut state, Message::Action(ActionMessage::PurgeStopped));
        let _task = super::update(&mut state, Message::Action(ActionMessage::ConfirmPending));
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::PurgeStopped,
                result: Err(ClientError::Transport("connection refused".to_owned())),
            }),
        );

        assert_eq!(
            state.refresh_feedback(),
            Some("Connection failed. Check the endpoint and secret.")
        );
        assert_eq!(state.refresh_state(), RefreshState::Fresh);
    }

    #[test]
    fn remove_requires_confirmation_when_preference_enabled() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let gid = Gid::new("active-gid").expect("valid gid");
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Remove(gid.clone())),
        );

        assert_eq!(
            state.pending_action_confirmation(),
            Some(PendingActionConfirmation::Remove(gid))
        );
        assert!(!state.download_rows()[0].pending());

        let _task = super::update(&mut state, Message::Action(ActionMessage::ConfirmPending));

        assert_eq!(state.pending_action_confirmation(), None);
        assert!(state.download_rows()[0].pending());
    }

    #[test]
    fn purge_stopped_requires_confirmation_when_preference_enabled() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("complete-gid", DownloadStatus::Complete)],
        );

        let _task = super::update(&mut state, Message::Action(ActionMessage::PurgeStopped));

        assert_eq!(
            state.pending_action_confirmation(),
            Some(PendingActionConfirmation::PurgeStopped)
        );

        let _task = super::update(&mut state, Message::Action(ActionMessage::ConfirmPending));

        assert_eq!(state.pending_action_confirmation(), None);
        assert!(!state.can_purge_stopped());
    }

    #[test]
    fn cancel_destructive_confirmation_does_not_start_action() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Remove(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(&mut state, Message::Action(ActionMessage::CancelPending));

        assert_eq!(state.pending_action_confirmation(), None);
        assert!(!state.download_rows()[0].pending());
        assert_eq!(state.refresh_state(), RefreshState::Fresh);
    }

    #[test]
    fn disabled_destructive_confirmation_runs_action_immediately() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::ConfirmDestructiveActionsChanged(false)),
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Remove(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );

        assert_eq!(state.pending_action_confirmation(), None);
        assert!(state.download_rows()[0].pending());
    }

    #[test]
    fn action_in_flight_blocks_destructive_confirmation() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![
                download_item("first-gid", DownloadStatus::Active),
                download_item("second-gid", DownloadStatus::Active),
            ],
        );

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("first-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Remove(
                Gid::new("second-gid").expect("valid gid"),
            )),
        );

        assert_eq!(state.pending_action_confirmation(), None);
        assert_eq!(
            state
                .download_rows()
                .iter()
                .filter(|row| row.pending())
                .count(),
            1
        );
    }

    #[test]
    fn selecting_download_marks_row_and_builds_detail_view() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let _task = super::update(
            &mut state,
            Message::Selection(SelectionMessage::Select(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );

        assert_eq!(state.selected_gid().map(Gid::as_str), Some("active-gid"));
        assert!(state.download_rows()[0].selected());

        let detail = state.selected_download_detail().expect("selected detail");
        assert_eq!(detail.gid(), "active-gid");
        assert_eq!(detail.status(), "Active");
        assert_eq!(detail.progress(), "50% | 1.0 KiB / 2.0 KiB");
        assert_eq!(detail.speeds(), "Down 512 B/s | ETA 2s");
        assert_eq!(detail.file_tree()[0].label, "active-gid.bin");
        assert_eq!(
            detail.file_tree()[0].end.as_deref(),
            Some("1.0 KiB / 2.0 KiB")
        );
    }

    #[test]
    fn selecting_download_requests_selected_detail_refresh() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        let _task = super::update(
            &mut state,
            Message::Selection(SelectionMessage::Select(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );

        assert_eq!(state.refresh_state(), RefreshState::Refreshing);
    }

    #[test]
    fn selected_detail_merges_torrent_metadata_without_losing_summary() {
        let mut state = State::initial();
        connect(&mut state);
        let selected_gid = Gid::new("active-gid").expect("valid gid");
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );
        state.select_download(selected_gid.clone());
        let (generation, _, request) = state.begin_dirty_downloads_refresh().expect("refresh");
        assert_eq!(request.selected_gid(), Some(&selected_gid));

        let mut detail = DownloadDetail::new(download_item("active-gid", DownloadStatus::Active));
        detail.set_directory(Some("/downloads".to_owned()));
        detail.set_connections(4);
        detail.set_piece_length_bytes(262_144);
        detail.set_piece_count(32);
        detail.set_torrent(Some(TorrentDetail::new(
            Some("0123456789abcdef".to_owned()),
            true,
            12,
        )));
        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation,
                result: Ok(DownloadSnapshot::with_selected_detail(
                    GlobalStats::new(1_536, 512, 1, 0, 0),
                    Vec::new(),
                    Some(detail),
                )),
            }),
        );

        let detail = state.selected_download_detail().expect("selected detail");
        assert_eq!(detail.directory(), Some("/downloads"));
        assert!(detail.technical().contains(&"Connections 4".to_owned()));
        assert!(
            detail
                .torrent()
                .contains(&"Info hash 0123456789abcdef".to_owned())
        );
        assert!(detail.torrent().contains(&"Seeding yes".to_owned()));
    }

    #[test]
    fn download_rows_classify_curated_file_icons() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![
                download_item_with_path("movie-gid", DownloadStatus::Active, "/tmp/movie.mkv"),
                download_item_with_path("archive-gid", DownloadStatus::Active, "/tmp/data.zip"),
                download_item_with_path(
                    "torrent-gid",
                    DownloadStatus::Active,
                    "/tmp/linux.torrent",
                ),
            ],
        );

        let rows = state.download_rows();

        assert_eq!(rows[0].file_icon(), FileIcon::Video);
        assert_eq!(rows[1].file_icon(), FileIcon::Archive);
        assert_eq!(rows[2].file_icon(), FileIcon::Torrent);
    }

    #[test]
    fn download_rows_present_shared_folder_download_as_folder_card() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_folder_item(
                "folder-gid",
                "/downloads",
                [
                    "/downloads/Show Pack/info.txt",
                    "/downloads/Show Pack/poster.png",
                    "/downloads/Show Pack/video.mkv",
                ],
            )],
        );

        let rows = state.download_rows();

        assert_eq!(rows[0].name(), "Show Pack");
        assert_eq!(rows[0].file_icon(), FileIcon::Folder);
        assert_eq!(rows[0].metadata(), "3 files | GID folder-gid");
    }

    #[test]
    fn selected_folder_download_builds_file_tree() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_folder_item(
                "folder-gid",
                "/downloads",
                [
                    "/downloads/Show Pack/info.txt",
                    "/downloads/Show Pack/extras/poster.png",
                    "/downloads/Show Pack/video.mkv",
                ],
            )],
        );

        state.select_download(Gid::new("folder-gid").expect("valid gid"));

        let detail = state.selected_download_detail().expect("selected detail");
        let root = &detail.file_tree()[0];
        assert_eq!(root.id, "folder:Show Pack");
        assert_eq!(root.label, "Show Pack");
        assert!(
            state
                .selected_file_tree_state()
                .is_expanded("folder:Show Pack")
        );
        assert_eq!(root.children[0].label, "info.txt");
        assert_eq!(root.children[1].label, "extras");
        assert_eq!(root.children[1].children[0].label, "poster.png");
        assert_eq!(root.children[2].label, "video.mkv");
    }

    #[test]
    fn file_tree_selection_and_expansion_survive_same_download_refresh() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_folder_item(
                "folder-gid",
                "/downloads",
                [
                    "/downloads/Show Pack/info.txt",
                    "/downloads/Show Pack/extras/poster.png",
                ],
            )],
        );
        state.select_download(Gid::new("folder-gid").expect("valid gid"));

        state.update_file_tree(TreeMessage::Toggle("folder:Show Pack/extras".to_owned()));
        state.update_file_tree(TreeMessage::Select(
            "file:1:Show Pack/extras/poster.png".to_owned(),
        ));
        apply_snapshot(
            &mut state,
            vec![download_folder_item(
                "folder-gid",
                "/downloads",
                [
                    "/downloads/Show Pack/info.txt",
                    "/downloads/Show Pack/extras/poster.png",
                ],
            )],
        );

        assert!(
            state
                .selected_file_tree_state()
                .is_expanded("folder:Show Pack/extras")
        );
        assert!(
            state
                .selected_file_tree_state()
                .is_selected("file:1:Show Pack/extras/poster.png")
        );
    }

    #[test]
    fn file_tree_state_resets_when_selected_download_changes() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![
                download_folder_item(
                    "first-gid",
                    "/downloads",
                    [
                        "/downloads/First/info.txt",
                        "/downloads/First/extras/poster.png",
                    ],
                ),
                download_folder_item(
                    "second-gid",
                    "/downloads",
                    ["/downloads/Second/info.txt", "/downloads/Second/video.mkv"],
                ),
            ],
        );
        state.select_download(Gid::new("first-gid").expect("valid gid"));
        state.update_file_tree(TreeMessage::Toggle("folder:First/extras".to_owned()));
        state.update_file_tree(TreeMessage::Select(
            "file:1:First/extras/poster.png".to_owned(),
        ));

        state.select_download(Gid::new("second-gid").expect("valid gid"));

        assert!(
            !state
                .selected_file_tree_state()
                .is_expanded("folder:First/extras")
        );
        assert!(
            !state
                .selected_file_tree_state()
                .is_selected("file:1:First/extras/poster.png")
        );
        assert!(
            state
                .selected_file_tree_state()
                .is_expanded("folder:Second")
        );
    }

    #[test]
    fn download_rows_keep_file_icon_for_multi_file_without_shared_folder() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_folder_item(
                "loose-gid",
                "/downloads",
                ["/downloads/movie.mkv", "/downloads/subtitle.srt"],
            )],
        );

        let rows = state.download_rows();

        assert_eq!(rows[0].name(), "movie.mkv");
        assert_eq!(rows[0].file_icon(), FileIcon::Video);
        assert_eq!(rows[0].metadata(), "GID loose-gid");
    }

    #[test]
    fn download_rows_keep_file_icon_for_multi_file_in_different_folders() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_folder_item(
                "split-gid",
                "/downloads",
                ["/downloads/one/movie.mkv", "/downloads/two/subtitle.srt"],
            )],
        );

        let rows = state.download_rows();

        assert_eq!(rows[0].name(), "movie.mkv");
        assert_eq!(rows[0].file_icon(), FileIcon::Video);
        assert_eq!(rows[0].metadata(), "GID split-gid");
    }

    #[test]
    fn disappeared_selected_download_clears_selection() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );
        state.select_download(Gid::new("active-gid").expect("valid gid"));

        apply_snapshot(
            &mut state,
            vec![download_item("other-gid", DownloadStatus::Active)],
        );

        assert_eq!(state.selected_gid(), None);
        assert_eq!(state.selected_download_detail(), None);
    }

    #[test]
    fn detail_error_is_display_safe() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );
        let _task = super::update(
            &mut state,
            Message::Selection(SelectionMessage::Select(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );
        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Finished {
                generation: 1,
                target: ActionTarget::Download(Gid::new("active-gid").expect("valid gid")),
                result: Err(ClientError::Rpc {
                    code: 1,
                    message: "token:super-secret failed".to_owned(),
                }),
            }),
        );

        let detail = state.selected_download_detail().expect("selected detail");
        assert_eq!(detail.error(), Some("aria2 returned an RPC error."));
        assert!(!format!("{detail:?}").contains("super-secret"));
    }

    #[test]
    fn global_stats_are_unavailable_but_displayable_before_refresh() {
        let state = State::initial();

        assert_eq!(state.download_speed_text(), "0 B/s");
        assert_eq!(state.upload_speed_text(), "0 B/s");
        assert_eq!(state.counts_text(), "Active 0 | Waiting 0 | Stopped 0");
        assert_eq!(state.global_stats(), None);
    }

    #[test]
    fn download_snapshot_refresh_result_updates_visible_shell_labels_and_rows() {
        let mut state = State::initial();
        use_external_mode(&mut state);
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestRequested),
        );
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestFinished {
                generation: 1,
                settings: Settings::default(),
                result: Ok(ConnectionTest::new(VersionInfo::new("1.37.0", Vec::new()))),
            }),
        );

        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation: 1,
                result: Ok(snapshot_with_items(vec![download_item(
                    "active-gid",
                    DownloadStatus::Active,
                )])),
            }),
        );

        assert_eq!(
            state.global_stats(),
            Some(GlobalStats::new(1_536, 512, 2, 3, 4))
        );
        assert_eq!(state.download_speed_text(), "1.5 KiB/s");
        assert_eq!(state.upload_speed_text(), "512 B/s");
        assert_eq!(state.counts_text(), "Active 2 | Waiting 3 | Stopped 4");
        assert_eq!(state.download_items().len(), 1);
        let rows = state.download_rows();
        assert_eq!(rows[0].name(), "active-gid.bin");
        assert_eq!(
            rows[0].trailing(),
            &DownloadRowTrailing::Speed {
                download: "512 B/s".to_owned(),
                upload: "0 B/s".to_owned(),
                eta: "2s".to_owned(),
            }
        );
        assert_eq!(state.refresh_state(), RefreshState::Fresh);
    }

    #[test]
    fn download_row_speed_display_always_includes_upload_and_eta() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item_with_transfer_speeds(
                "active-gid",
                DownloadStatus::Active,
                512,
                128,
            )],
        );

        let rows = state.download_rows();

        assert_eq!(
            rows[0].trailing(),
            &DownloadRowTrailing::Speed {
                download: "512 B/s".to_owned(),
                upload: "128 B/s".to_owned(),
                eta: "2s".to_owned(),
            }
        );
    }

    #[test]
    fn non_active_rows_show_status_in_the_speed_slot() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![
                download_item("waiting-gid", DownloadStatus::Waiting),
                download_item("error-gid", DownloadStatus::Error),
            ],
        );

        let rows = state.download_rows();

        assert_eq!(
            rows[0].trailing(),
            &DownloadRowTrailing::Status("Waiting".to_owned())
        );

        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::FilterChanged(DownloadFilter::Complete)),
        );
        let rows = state.download_rows();

        assert_eq!(
            rows[0].trailing(),
            &DownloadRowTrailing::Status("Error".to_owned())
        );
    }

    #[test]
    fn refresh_requests_are_ignored_while_refreshing() {
        let mut state = State::initial();
        connect(&mut state);

        let (first_generation, _, _) = state.begin_downloads_refresh().expect("first refresh");
        assert_eq!(state.begin_downloads_refresh(), None);

        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation: first_generation,
                result: Ok(snapshot_with_items(vec![download_item(
                    "active-gid",
                    DownloadStatus::Active,
                )])),
            }),
        );

        assert_eq!(
            state.global_stats(),
            Some(GlobalStats::new(1_536, 512, 2, 3, 4))
        );
        assert_eq!(state.download_speed_text(), "1.5 KiB/s");
        assert_eq!(state.download_items().len(), 1);
    }

    #[test]
    fn refresh_errors_preserve_last_good_snapshot_and_mark_stale() {
        let mut state = State::initial();
        connect(&mut state);

        let (generation, _, _) = state.begin_downloads_refresh().expect("refresh");
        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation,
                result: Ok(snapshot_with_items(vec![download_item(
                    "active-gid",
                    DownloadStatus::Active,
                )])),
            }),
        );

        let (generation, _, _) = state.begin_downloads_refresh().expect("refresh");
        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation,
                result: Err(ClientError::Transport(
                    "token:super-secret refused".to_owned(),
                )),
            }),
        );

        assert_eq!(
            state.global_stats(),
            Some(GlobalStats::new(1_536, 512, 2, 3, 4))
        );
        assert_eq!(state.download_items().len(), 1);
        assert_eq!(state.refresh_state(), RefreshState::Stale);
        assert_eq!(
            state.refresh_feedback(),
            Some("Connection failed. Check the endpoint and secret.")
        );
        assert!(!format!("{state:?}").contains("super-secret"));
    }

    #[test]
    fn active_filter_returns_waiting_paused_and_active_rows_in_visible_order() {
        let mut state = State::initial();
        connect(&mut state);

        let (generation, _, _) = state.begin_downloads_refresh().expect("refresh");
        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation,
                result: Ok(snapshot_with_items(vec![
                    download_item("active-gid", DownloadStatus::Active),
                    download_item("waiting-gid", DownloadStatus::Waiting),
                    download_item("paused-gid", DownloadStatus::Paused),
                    download_item("complete-gid", DownloadStatus::Complete),
                    download_item("error-gid", DownloadStatus::Error),
                ])),
            }),
        );

        let rows = state.download_rows();

        assert_eq!(
            rows.iter().map(|row| row.gid()).collect::<Vec<_>>(),
            vec!["waiting-gid", "paused-gid", "active-gid"]
        );
        assert_eq!(state.filter_count(DownloadFilter::All), 5);
        assert_eq!(state.filter_count(DownloadFilter::Active), 3);
        assert_eq!(state.filter_count(DownloadFilter::Complete), 2);
    }

    #[test]
    fn complete_filter_returns_error_rows_before_complete_rows() {
        let mut state = State::initial();
        connect(&mut state);

        apply_snapshot(
            &mut state,
            vec![
                download_item("complete-gid", DownloadStatus::Complete),
                download_item("error-gid", DownloadStatus::Error),
                download_item("active-gid", DownloadStatus::Active),
            ],
        );

        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::FilterChanged(DownloadFilter::Complete)),
        );

        let rows = state.download_rows();

        assert_eq!(
            rows.iter().map(|row| row.gid()).collect::<Vec<_>>(),
            vec!["error-gid", "complete-gid"]
        );
    }

    #[test]
    fn exact_download_status_filters_remain_available_internally() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![
                download_item("waiting-gid", DownloadStatus::Waiting),
                download_item("paused-gid", DownloadStatus::Paused),
                download_item("error-gid", DownloadStatus::Error),
            ],
        );

        assert_eq!(state.filter_count(DownloadFilter::Waiting), 1);
        assert_eq!(state.filter_count(DownloadFilter::Paused), 1);
        assert_eq!(state.filter_count(DownloadFilter::Error), 1);
    }

    #[test]
    fn partial_active_refresh_preserves_stopped_history_rows() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![
                download_item("active-gid", DownloadStatus::Active),
                download_item("done-gid", DownloadStatus::Complete),
            ],
        );

        state.invalidate_refresh(RefreshInvalidation::Active);
        let (generation, _, request) = state
            .begin_scheduled_downloads_refresh()
            .expect("dirty active refresh");
        assert!(request.include_active());
        assert!(!request.include_stopped());

        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation,
                result: Ok(snapshot_with_items(vec![download_item(
                    "active-gid",
                    DownloadStatus::Active,
                )])),
            }),
        );

        assert_eq!(state.filter_count(DownloadFilter::Complete), 1);
    }

    #[test]
    fn download_start_notification_dirties_active_and_waiting_sections() {
        let mut state = State::initial();
        connect(&mut state);

        state.invalidate_refresh(RefreshInvalidation::Aria2Notification(
            Aria2Notification::DownloadStart(Gid::new("active-gid").expect("valid gid")),
        ));
        let (_, _, request) = state
            .begin_dirty_downloads_refresh()
            .expect("notification starts dirty refresh");

        assert!(request.include_active());
        assert!(request.include_waiting());
        assert!(!request.include_stopped());
    }

    #[test]
    fn websocket_connected_event_updates_compact_status_text() {
        let mut state = State::initial();
        connect(&mut state);

        let _task = super::update(
            &mut state,
            Message::WebSocket(WebSocketMessage::Event(WebSocketEvent::Connected)),
        );

        assert!(state.status_text().contains("WebSocket live"));
    }

    #[test]
    fn websocket_notification_enters_dirty_refresh_path() {
        let mut state = State::initial();
        connect(&mut state);

        let _task = super::update(
            &mut state,
            Message::WebSocket(WebSocketMessage::Event(WebSocketEvent::Notification(
                Aria2Notification::DownloadStart(Gid::new("active-gid").expect("valid gid")),
            ))),
        );

        assert_eq!(state.refresh_state(), RefreshState::Refreshing);
    }

    #[test]
    fn complete_notification_refreshes_all_sections_and_selected_detail() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        state.select_download(Gid::new("active-gid").expect("valid gid"));
        state.invalidate_refresh(RefreshInvalidation::Aria2Notification(
            Aria2Notification::DownloadComplete(Gid::new("active-gid").expect("valid gid")),
        ));
        let (_, _, request) = state
            .begin_dirty_downloads_refresh()
            .expect("notification starts dirty refresh");

        assert!(request.include_active());
        assert!(request.include_waiting());
        assert!(request.include_stopped());
        assert_eq!(request.selected_gid().map(Gid::as_str), Some("active-gid"));
    }

    #[test]
    fn notification_preference_records_complete_and_error_transitions_once() {
        let mut state = State::initial();
        connect(&mut state);
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::NotifyDownloadOutcomesChanged(true)),
        );
        apply_snapshot(
            &mut state,
            vec![
                download_item("complete-gid", DownloadStatus::Active),
                download_item("error-gid", DownloadStatus::Active),
            ],
        );

        apply_snapshot(
            &mut state,
            vec![
                download_item("complete-gid", DownloadStatus::Complete),
                download_item("error-gid", DownloadStatus::Error),
            ],
        );
        apply_snapshot(
            &mut state,
            vec![
                download_item("complete-gid", DownloadStatus::Complete),
                download_item("error-gid", DownloadStatus::Error),
            ],
        );

        assert_eq!(state.notification_intents().len(), 2);
        assert_eq!(
            state.notification_intents()[0].gid().as_str(),
            "complete-gid"
        );
        assert_eq!(
            state.notification_intents()[0].outcome(),
            NotificationOutcome::Complete
        );
        assert_eq!(state.notification_intents()[1].gid().as_str(), "error-gid");
        assert_eq!(
            state.notification_intents()[1].outcome(),
            NotificationOutcome::Failed
        );
    }

    #[test]
    fn disabled_notification_preference_does_not_record_transitions() {
        let mut state = State::initial();
        connect(&mut state);
        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Active)],
        );

        apply_snapshot(
            &mut state,
            vec![download_item("active-gid", DownloadStatus::Complete)],
        );

        assert!(state.notification_intents().is_empty());
    }

    #[test]
    fn unknown_notification_without_gid_does_not_start_dirty_refresh() {
        let mut state = State::initial();
        connect(&mut state);

        assert!(
            !state.invalidate_refresh(RefreshInvalidation::Aria2Notification(
                Aria2Notification::Unknown {
                    method: "aria2.onFutureEvent".to_owned(),
                    gid: None,
                },
            ))
        );
    }

    #[test]
    fn scheduled_refresh_waits_when_idle_downloads_are_current() {
        let mut state = State::initial();
        connect(&mut state);

        assert_eq!(state.begin_scheduled_downloads_refresh(), None);
    }

    #[test]
    fn empty_and_loading_download_states_have_display_text() {
        let mut state = State::initial();

        assert_eq!(
            state.downloads_empty_text(),
            Some("No downloads found.".to_owned())
        );

        connect(&mut state);
        let _refresh = state.begin_downloads_refresh().expect("refresh");

        assert_eq!(
            state.downloads_empty_text(),
            Some("Loading downloads...".to_owned())
        );
    }

    fn connect(state: &mut State) {
        use_external_mode(state);
        let _task = super::update(state, Message::Connection(ConnectionMessage::TestRequested));
        let _task = super::update(
            state,
            Message::Connection(ConnectionMessage::TestFinished {
                generation: 1,
                settings: Settings::default(),
                result: Ok(ConnectionTest::new(VersionInfo::new("1.37.0", Vec::new()))),
            }),
        );
        let _task = super::update(
            state,
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation: 1,
                result: Ok(snapshot_with_items(Vec::new())),
            }),
        );
    }

    fn use_external_mode(state: &mut State) {
        let _task = super::update(
            state,
            Message::Settings(SettingsMessage::DaemonModeChanged(DaemonMode::External)),
        );
        let _task = super::update(state, Message::Settings(SettingsMessage::Save));
    }

    fn snapshot_with_items(items: Vec<DownloadItem>) -> DownloadSnapshot {
        DownloadSnapshot::new(GlobalStats::new(1_536, 512, 2, 3, 4), items)
    }

    fn apply_snapshot(state: &mut State, items: Vec<DownloadItem>) {
        let (generation, _, _) = state.begin_downloads_refresh().expect("refresh");
        let _task = super::update(
            state,
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation,
                result: Ok(snapshot_with_items(items)),
            }),
        );
    }

    fn download_item(gid: &str, status: DownloadStatus) -> DownloadItem {
        download_item_with_path(gid, status, format!("/tmp/{gid}.bin"))
    }

    fn download_item_with_transfer_speeds(
        gid: &str,
        status: DownloadStatus,
        download_speed: u64,
        upload_speed: u64,
    ) -> DownloadItem {
        DownloadItem::new(
            Gid::new(gid).expect("valid gid"),
            status,
            2048,
            1024,
            download_speed,
            upload_speed,
            vec![DownloadFile::new(
                format!("/tmp/{gid}.bin"),
                2048,
                1024,
                true,
            )],
        )
    }

    fn download_item_with_path(
        gid: &str,
        status: DownloadStatus,
        path: impl Into<String>,
    ) -> DownloadItem {
        DownloadItem::new(
            Gid::new(gid).expect("valid gid"),
            status,
            2048,
            1024,
            512,
            0,
            vec![DownloadFile::new(path, 2048, 1024, true)],
        )
    }

    fn download_folder_item<const N: usize>(
        gid: &str,
        directory: &str,
        paths: [&str; N],
    ) -> DownloadItem {
        let mut item = DownloadItem::new(
            Gid::new(gid).expect("valid gid"),
            DownloadStatus::Active,
            4096,
            2048,
            512,
            0,
            paths
                .into_iter()
                .map(|path| DownloadFile::new(path, 1024, 512, true))
                .collect(),
        );
        item.set_directory(Some(directory.to_owned()));
        item
    }

    fn temp_config_path(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("cottid-app-{name}-{unique}"));
        fs::create_dir_all(&dir).expect("temp dir");
        dir.join("config")
    }
}
