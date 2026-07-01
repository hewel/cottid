mod message;
mod scheduler;
mod state;
mod subscriptions;
mod update;

use iced::{Element, Task};

pub use message::{
    ActionMessage, ActionTarget, AddMessage, ConnectionMessage, DownloadsMessage, Message,
    RefreshInvalidation, SelectionMessage, SettingsMessage, ToolbarMessage,
};
pub use state::{
    ConnectionStatus, DownloadDetailView, DownloadFilter, DownloadRowView, FileIcon, RefreshState,
    State,
};

pub fn run() -> iced::Result {
    iced::application(boot, update, view)
        .title("Cottid")
        .subscription(subscription)
        .run()
}

fn boot() -> (State, Task<Message>) {
    boot_with(State::load())
}

fn boot_with(mut state: State) -> (State, Task<Message>) {
    let task = update::start_connection_test(&mut state);
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::aria2::client::ConnectionTest;
    use crate::aria2::domain::{
        DownloadDetail, DownloadFile, DownloadItem, DownloadSnapshot, DownloadStatus, Gid,
        GlobalStats, TorrentDetail, VersionInfo,
    };
    use crate::aria2::errors::ClientError;
    use crate::aria2::notifications::Aria2Notification;
    use crate::config::{RpcAuthDraft, Settings};

    use super::{
        ActionMessage, ActionTarget, AddMessage, ConnectionMessage, ConnectionStatus,
        DownloadFilter, DownloadsMessage, FileIcon, Message, RefreshInvalidation, RefreshState,
        SelectionMessage, SettingsMessage, State, ToolbarMessage,
    };

    #[test]
    fn starts_offline_and_settings_ready() {
        let state = State::initial();

        assert_eq!(state.connection_status(), ConnectionStatus::Offline);
        assert!(state.is_settings_ready());
    }

    #[test]
    fn boot_starts_connection_test_from_loaded_config() {
        let path = temp_config_path("boot-auto-connect");

        let (state, _task) = super::boot_from_path(path);

        assert_eq!(state.connection_status(), ConnectionStatus::Testing);
    }

    #[test]
    fn boot_connection_success_triggers_initial_refresh() {
        let path = temp_config_path("boot-refresh");
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
    fn subscription_hook_is_available_before_connection() {
        let state = State::initial();

        let _subscription = super::subscription(&state);
    }

    #[test]
    fn window_resize_updates_compact_layout_state() {
        let mut state = State::initial();

        let _task = super::update(&mut state, Message::WindowResized(720));

        assert!(state.is_compact_layout());

        let _task = super::update(&mut state, Message::WindowResized(1200));

        assert!(!state.is_compact_layout());
    }

    #[test]
    fn view_builds_from_normalized_app_state() {
        let state = State::initial();

        let _element = super::view(&state);
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
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        let reloaded = State::load_from_path(path);

        assert_eq!(
            reloaded.applied_endpoint(),
            "http://aria2.local:6800/jsonrpc"
        );
        assert_eq!(reloaded.draft_polling_interval_seconds(), 7);
        assert_eq!(reloaded.applied_auth_label(), "No authentication");
    }

    #[test]
    fn successful_settings_connection_test_applies_and_saves_settings() {
        let path = temp_config_path("test-connection-save");
        let mut state = State::load_from_path(path.clone());
        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));

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
            state.settings_feedback(),
            Some("Connection test succeeded and settings saved.")
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
            Message::Downloads(DownloadsMessage::FilterChanged(DownloadFilter::Paused)),
        );

        let reloaded = State::load_from_path(path);

        assert_eq!(reloaded.selected_filter(), DownloadFilter::Paused);
    }

    #[test]
    fn invalid_config_loads_defaults_with_feedback() {
        let path = temp_config_path("invalid-state");
        fs::write(&path, "endpoint=ftp://bad\n").expect("write invalid config");

        let state = State::load_from_path(path);

        assert_eq!(state.applied_endpoint(), "http://localhost:6800/jsonrpc");
        assert_eq!(
            state.settings_feedback(),
            Some("Config was invalid; using defaults.")
        );
    }

    #[test]
    fn secure_token_is_restored_without_writing_plaintext_config() {
        let path = temp_config_path("session-secret");
        let mut state = State::load_from_path(path.clone());

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::AuthChanged(RpcAuthDraft::SessionSecret)),
        );
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
    fn invalid_endpoint_feedback_stays_in_settings_state() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::EndpointChanged(
                "ftp://localhost:6800/jsonrpc".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        assert_eq!(
            state.settings_feedback(),
            Some("Endpoint must start with http:// or https://.")
        );
        assert!(state.is_settings_open());
        assert_eq!(state.applied_endpoint(), "http://localhost:6800/jsonrpc");
    }

    #[test]
    fn settings_can_choose_session_secret_without_displaying_secret() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::AuthChanged(RpcAuthDraft::SessionSecret)),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::SecretChanged("super-secret".to_owned())),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        assert_eq!(state.applied_auth_label(), "Token secret");
        assert!(!state.status_text().contains("super-secret"));
    }

    #[test]
    fn connection_test_result_updates_visible_connection_state() {
        let mut state = State::initial();
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
            state.settings_feedback(),
            Some("Connection test succeeded.")
        );
    }

    #[test]
    fn initial_refresh_failure_keeps_successful_connection_state() {
        let mut state = State::initial();
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
            state.settings_feedback(),
            Some("Connection failed. Check the endpoint and secret.")
        );
    }

    #[test]
    fn add_submit_validates_one_uri_or_magnet_before_rpc() {
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
            state.add_feedback(),
            Some("Enter an http, https, or magnet link.")
        );
        assert!(!state.is_add_pending());

        let _task = super::update(
            &mut state,
            Message::Add(AddMessage::InputChanged(
                "magnet:?xt=urn:btih:abc".to_owned(),
            )),
        );

        assert!(state.is_add_ready());
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
        assert_eq!(state.add_feedback(), Some("Download added."));
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
        assert_eq!(state.add_feedback(), Some("aria2 returned an RPC error."));
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
        assert!(rows[0].can_pause());
        assert!(!rows[0].can_unpause());
        assert!(rows[0].can_remove());
        assert!(!rows[2].can_remove());
        assert!(state.can_purge_stopped());

        let _task = super::update(
            &mut state,
            Message::Action(ActionMessage::Pause(
                Gid::new("active-gid").expect("valid gid"),
            )),
        );

        let rows = state.download_rows();
        assert!(rows[0].pending());
        assert!(!rows[1].can_unpause());
    }

    #[test]
    fn action_success_triggers_snapshot_refresh_without_retrying_command() {
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

        assert_eq!(state.refresh_state(), RefreshState::Refreshing);
        assert!(!state.download_rows()[0].pending());
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
        assert_eq!(detail.files()[0], "/tmp/active-gid.bin | 1.0 KiB / 2.0 KiB");
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
        assert_eq!(state.download_rows()[0].name(), "active-gid.bin");
        assert_eq!(state.refresh_state(), RefreshState::Fresh);
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
    fn filter_state_returns_only_matching_download_rows() {
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
                    download_item("error-gid", DownloadStatus::Error),
                ])),
            }),
        );

        let _task = super::update(
            &mut state,
            Message::Downloads(DownloadsMessage::FilterChanged(DownloadFilter::Error)),
        );

        assert_eq!(state.download_rows().len(), 1);
        assert_eq!(state.download_rows()[0].gid(), "error-gid");
        assert_eq!(state.filter_count(DownloadFilter::All), 3);
        assert_eq!(state.filter_count(DownloadFilter::Active), 1);
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
