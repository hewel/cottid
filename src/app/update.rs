use iced::Task;
use iced::widget::operation;

use super::state::RunningAction;
use super::{
    ActionMessage, AddMessage, ConnectionMessage, DaemonMessage, DownloadsMessage, Message,
    SelectionMessage, SettingsMessage, State, ToolbarMessage, WebSocketMessage,
};

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Add(message) => update_add(state, message),
        Message::Action(message) => update_action(state, message),
        Message::Connection(message) => update_connection(state, message),
        Message::Daemon(message) => update_daemon(state, message),
        Message::Downloads(message) => update_downloads(state, message),
        Message::ModalCancel => {
            state.cancel_active_modal();
            Task::none()
        }
        Message::TogglePopover(id) => {
            state.toggle_popover(id);
            Task::none()
        }
        Message::ClosePopover => {
            state.close_popover();
            Task::none()
        }
        Message::Selection(message) => update_selection(state, message),
        Message::Tree(message) => {
            state.update_file_tree(message);
            Task::none()
        }
        Message::Toolbar(message) => update_toolbar(state, message),
        Message::Settings(message) => update_settings(state, message),
        Message::WebSocket(message) => update_websocket(state, message),
        Message::FocusTextInput(target) => operation::focus(target.id()),
        Message::WindowResized { width, height } => {
            state.set_viewport_size(width, height);
            Task::none()
        }
    }
}

pub fn start_boot_connection(state: &mut State) -> Task<Message> {
    if matches!(state.daemon_mode(), crate::config::DaemonMode::Managed) {
        start_managed_daemon(state)
    } else {
        start_connection_test(state)
    }
}

pub fn start_managed_daemon(state: &mut State) -> Task<Message> {
    let Some((generation, config)) = state.begin_managed_daemon_start() else {
        return Task::none();
    };

    start_managed_daemon_task(generation, config)
}

fn start_managed_daemon_task(
    generation: u64,
    config: crate::daemon::ManagedDaemonConfig,
) -> Task<Message> {
    Task::perform(
        async move { crate::daemon::start_managed_daemon(config).await },
        move |result| Message::Daemon(DaemonMessage::StartFinished { generation, result }),
    )
}

fn update_daemon(state: &mut State, message: DaemonMessage) -> Task<Message> {
    match message {
        DaemonMessage::MonitorTick => {
            let Some((generation, config)) = state.poll_managed_daemon_exit() else {
                return Task::none();
            };

            start_managed_daemon_task(generation, config)
        }
        DaemonMessage::ChildExited { generation } => {
            let Some((generation, config)) = state.handle_managed_daemon_exit(generation) else {
                return Task::none();
            };

            start_managed_daemon_task(generation, config)
        }
        DaemonMessage::StartFinished { generation, result } => {
            let Some(settings) = state.finish_managed_daemon_start(generation, result) else {
                return Task::none();
            };

            start_connected_tasks(state, settings)
        }
    }
}

fn update_websocket(state: &mut State, message: WebSocketMessage) -> Task<Message> {
    match message {
        WebSocketMessage::Event(event) => {
            let Some(invalidation) = state.apply_websocket_event(event) else {
                return Task::none();
            };
            if !state.invalidate_refresh(invalidation) {
                return Task::none();
            }
            start_dirty_refresh(state)
        }
    }
}

pub fn start_connection_test(state: &mut State) -> Task<Message> {
    let Some((generation, settings)) = state.begin_connection_test() else {
        return Task::none();
    };

    let settings_for_test = settings.clone();

    Task::perform(
        async move { crate::aria2::client::test_connection(settings_for_test).await },
        move |result| {
            Message::Connection(ConnectionMessage::TestFinished {
                generation,
                settings,
                result,
            })
        },
    )
}

fn update_selection(state: &mut State, message: SelectionMessage) -> Task<Message> {
    match message {
        SelectionMessage::Select(gid) => {
            if state.select_download(gid) {
                return start_dirty_refresh(state);
            }

            Task::none()
        }
        SelectionMessage::Clear => {
            state.clear_selection();
            Task::none()
        }
    }
}

fn update_action(state: &mut State, message: ActionMessage) -> Task<Message> {
    match message {
        ActionMessage::ConfirmPending => {
            let Some((generation, settings, action)) = state.confirm_pending_action() else {
                return Task::none();
            };
            run_action_task(generation, settings, action)
        }
        ActionMessage::CancelPending => {
            state.cancel_pending_action();
            Task::none()
        }
        ActionMessage::Pause(_)
        | ActionMessage::Unpause(_)
        | ActionMessage::Remove(_)
        | ActionMessage::PurgeStopped
            if state.request_action_confirmation(message.clone()) =>
        {
            Task::none()
        }
        ActionMessage::Pause(_)
        | ActionMessage::Unpause(_)
        | ActionMessage::Remove(_)
        | ActionMessage::PurgeStopped => {
            let Some((generation, settings, action)) = state.begin_action(message) else {
                return Task::none();
            };
            run_action_task(generation, settings, action)
        }
        ActionMessage::Finished {
            generation,
            target,
            result,
        } => {
            if state.finish_action(generation, target, result) {
                return start_dirty_refresh(state);
            }

            Task::none()
        }
    }
}

fn run_action_task(
    generation: u64,
    settings: crate::config::Settings,
    action: RunningAction,
) -> Task<Message> {
    let target = action.target();

    Task::perform(
        async move {
            match action {
                RunningAction::Pause(gid) => {
                    crate::aria2::client::pause(settings, gid).await.map(|_| ())
                }
                RunningAction::Unpause(gid) => crate::aria2::client::unpause(settings, gid)
                    .await
                    .map(|_| ()),
                RunningAction::Remove(gid) => crate::aria2::client::remove(settings, gid)
                    .await
                    .map(|_| ()),
                RunningAction::PurgeStopped => crate::aria2::client::purge_stopped(settings).await,
            }
        },
        move |result| {
            Message::Action(ActionMessage::Finished {
                generation,
                target,
                result,
            })
        },
    )
}

fn update_add(state: &mut State, message: AddMessage) -> Task<Message> {
    match message {
        AddMessage::Open => {
            state.open_add_dialog();
            Task::none()
        }
        AddMessage::Cancel => {
            state.cancel_add_dialog();
            Task::none()
        }
        AddMessage::InputChanged(input) => {
            state.set_add_input(input);
            Task::none()
        }
        AddMessage::OutputFilenameChanged(output_filename) => {
            state.set_add_output_filename(output_filename);
            Task::none()
        }
        AddMessage::MaxDownloadLimitChanged(limit) => {
            state.set_add_max_download_limit(limit);
            Task::none()
        }
        AddMessage::MaxUploadLimitChanged(limit) => {
            state.set_add_max_upload_limit(limit);
            Task::none()
        }
        AddMessage::Submit => {
            let Some((generation, settings, uri, options)) = state.begin_add_uri() else {
                return Task::none();
            };

            Task::perform(
                async move { crate::aria2::client::add_uri(settings, uri, options).await },
                move |result| Message::Add(AddMessage::SubmitFinished { generation, result }),
            )
        }
        AddMessage::SubmitFinished { generation, result } => {
            if state.finish_add_uri(generation, result) {
                return start_dirty_refresh(state);
            }

            Task::none()
        }
    }
}

fn update_connection(state: &mut State, message: ConnectionMessage) -> Task<Message> {
    match message {
        ConnectionMessage::TestRequested => start_connection_test(state),
        ConnectionMessage::TestFinished {
            generation,
            settings,
            result,
        } => {
            if state.finish_connection_test(generation, settings.clone(), result) {
                return start_connected_tasks(state, settings);
            }

            Task::none()
        }
    }
}

fn start_connected_tasks(state: &mut State, settings: crate::config::Settings) -> Task<Message> {
    let Some((refresh_generation, refresh_settings, refresh_request)) =
        state.begin_downloads_refresh()
    else {
        return Task::none();
    };

    let refresh_task = Task::perform(
        async move {
            crate::aria2::client::fetch_download_snapshot_with_request(
                refresh_settings,
                refresh_request,
            )
            .await
        },
        move |result| {
            Message::Downloads(DownloadsMessage::RefreshFinished {
                generation: refresh_generation,
                result,
            })
        },
    );
    let websocket_settings = settings.clone();
    let (runtime_generation, runtime_settings) = state.begin_runtime_global_options_fetch(settings);
    let runtime_task = Task::perform(
        async move {
            let result =
                crate::aria2::client::get_runtime_global_options(runtime_settings.clone()).await;
            (runtime_settings, result)
        },
        move |(settings, result)| {
            Message::Settings(SettingsMessage::RuntimeGlobalOptionsFetched {
                generation: runtime_generation,
                settings,
                result,
            })
        },
    );
    let websocket_task = if websocket_settings.websocket_enabled() {
        Task::perform(
            async move { crate::aria2::client::test_websocket_notifications(websocket_settings).await },
            |result| {
                Message::WebSocket(WebSocketMessage::Event(match result {
                    Ok(()) => crate::aria2::websocket::WebSocketEvent::Connected,
                    Err(_) => crate::aria2::websocket::WebSocketEvent::Degraded,
                }))
            },
        )
    } else {
        Task::none()
    };

    Task::batch([refresh_task, runtime_task, websocket_task])
}

fn update_downloads(state: &mut State, message: DownloadsMessage) -> Task<Message> {
    match message {
        DownloadsMessage::RefreshTick => {
            let Some((generation, settings, request)) = state.begin_scheduled_downloads_refresh()
            else {
                return Task::none();
            };

            Task::perform(
                async move {
                    crate::aria2::client::fetch_download_snapshot_with_request(settings, request)
                        .await
                },
                move |result| {
                    Message::Downloads(DownloadsMessage::RefreshFinished { generation, result })
                },
            )
        }
        DownloadsMessage::RefreshRequested => {
            let Some((generation, settings, request)) = state.begin_downloads_refresh() else {
                return Task::none();
            };

            Task::perform(
                async move {
                    crate::aria2::client::fetch_download_snapshot_with_request(settings, request)
                        .await
                },
                move |result| {
                    Message::Downloads(DownloadsMessage::RefreshFinished { generation, result })
                },
            )
        }
        DownloadsMessage::Invalidated(invalidation) => {
            if !state.invalidate_refresh(invalidation) {
                return Task::none();
            }
            start_dirty_refresh(state)
        }
        DownloadsMessage::FilterChanged(filter) => {
            state.set_download_filter(filter);
            Task::none()
        }
        DownloadsMessage::RefreshFinished { generation, result } => {
            state.finish_downloads_refresh(generation, result);
            Task::none()
        }
    }
}

fn start_dirty_refresh(state: &mut State) -> Task<Message> {
    let Some((generation, settings, request)) = state.begin_dirty_downloads_refresh() else {
        return Task::none();
    };

    Task::perform(
        async move {
            crate::aria2::client::fetch_download_snapshot_with_request(settings, request).await
        },
        move |result| Message::Downloads(DownloadsMessage::RefreshFinished { generation, result }),
    )
}

fn update_toolbar(state: &mut State, message: ToolbarMessage) -> Task<Message> {
    match message {
        ToolbarMessage::OpenSettings => {
            state.open_settings();
            Task::none()
        }
        ToolbarMessage::ThemePreferenceSelected(theme_preference) => {
            state.set_theme_preference(theme_preference);
            Task::none()
        }
        ToolbarMessage::CycleThemePreference => {
            state.cycle_theme_preference();
            Task::none()
        }
    }
}

fn update_settings(state: &mut State, message: SettingsMessage) -> Task<Message> {
    match message {
        SettingsMessage::Cancel => {
            state.cancel_settings();
            Task::none()
        }
        SettingsMessage::Save => {
            let Some((generation, settings, options)) = state.save_settings() else {
                return Task::none();
            };

            Task::perform(
                async move {
                    let result = crate::aria2::client::change_runtime_global_options(
                        settings.clone(),
                        options,
                    )
                    .await;
                    (settings, result)
                },
                move |(settings, result)| {
                    Message::Settings(SettingsMessage::RuntimeGlobalOptionsSaved {
                        generation,
                        settings,
                        result,
                    })
                },
            )
        }
        SettingsMessage::SavePlaintextFallback => {
            state.save_plaintext_fallback();
            Task::none()
        }
        SettingsMessage::KeepSecretSessionOnly => {
            state.keep_secret_session_only();
            Task::none()
        }
        SettingsMessage::DaemonModeChanged(daemon_mode) => {
            state.set_draft_daemon_mode(daemon_mode);
            Task::none()
        }
        SettingsMessage::EndpointChanged(endpoint) => {
            state.set_draft_endpoint(endpoint);
            Task::none()
        }
        SettingsMessage::SecretChanged(secret) => {
            state.set_draft_secret(secret);
            Task::none()
        }
        SettingsMessage::PollingIntervalChanged(value) => {
            state.set_draft_polling_interval(value);
            Task::none()
        }
        SettingsMessage::WebSocketEnabledChanged(enabled) => {
            state.set_draft_websocket_enabled(enabled);
            Task::none()
        }
        SettingsMessage::NewDownloadDirectoryChanged(directory) => {
            state.set_draft_new_download_directory(directory);
            Task::none()
        }
        SettingsMessage::NewDownloadOutputFilenameChanged(output_filename) => {
            state.set_draft_new_download_output_filename(output_filename);
            Task::none()
        }
        SettingsMessage::NewDownloadMaxDownloadLimitChanged(limit) => {
            state.set_draft_new_download_max_download_limit(limit);
            Task::none()
        }
        SettingsMessage::NewDownloadMaxUploadLimitChanged(limit) => {
            state.set_draft_new_download_max_upload_limit(limit);
            Task::none()
        }
        SettingsMessage::RuntimeMaxConcurrentDownloadsChanged(value) => {
            state.set_draft_runtime_max_concurrent_downloads(value);
            Task::none()
        }
        SettingsMessage::RuntimeMaxOverallDownloadLimitChanged(value) => {
            state.set_draft_runtime_max_overall_download_limit(value);
            Task::none()
        }
        SettingsMessage::RuntimeMaxOverallUploadLimitChanged(value) => {
            state.set_draft_runtime_max_overall_upload_limit(value);
            Task::none()
        }
        SettingsMessage::RuntimeGlobalOptionsFetched {
            generation,
            settings,
            result,
        } => {
            state.apply_runtime_global_options(generation, settings, result);
            Task::none()
        }
        SettingsMessage::RuntimeGlobalOptionsSaved {
            generation,
            settings,
            result,
        } => {
            state.finish_runtime_global_options_save(generation, settings, result);
            Task::none()
        }
        SettingsMessage::ConfirmDestructiveActionsChanged(enabled) => {
            state.set_confirm_destructive_actions(enabled);
            Task::none()
        }
        SettingsMessage::NotifyDownloadOutcomesChanged(enabled) => {
            state.set_notify_download_outcomes(enabled);
            Task::none()
        }
    }
}
