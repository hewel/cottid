use iced::Task;

use super::state::RunningAction;
use super::{
    ActionMessage, AddMessage, ConnectionMessage, DownloadsMessage, Message, SelectionMessage,
    SettingsMessage, State, ToolbarMessage,
};

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Add(message) => update_add(state, message),
        Message::Action(message) => update_action(state, message),
        Message::Connection(message) => update_connection(state, message),
        Message::Downloads(message) => update_downloads(state, message),
        Message::Selection(message) => update_selection(state, message),
        Message::Toolbar(message) => update_toolbar(state, message),
        Message::Settings(message) => update_settings(state, message),
    }
}

fn update_selection(state: &mut State, message: SelectionMessage) -> Task<Message> {
    match message {
        SelectionMessage::Select(gid) => {
            state.select_download(gid);
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
        ActionMessage::Pause(_)
        | ActionMessage::Unpause(_)
        | ActionMessage::Remove(_)
        | ActionMessage::PurgeStopped => {
            let Some((generation, settings, action)) = state.begin_action(message) else {
                return Task::none();
            };
            let target = action.target();

            Task::perform(
                async move {
                    match action {
                        RunningAction::Pause(gid) => {
                            crate::aria2::client::pause(settings, gid).map(|_| ())
                        }
                        RunningAction::Unpause(gid) => {
                            crate::aria2::client::unpause(settings, gid).map(|_| ())
                        }
                        RunningAction::Remove(gid) => {
                            crate::aria2::client::remove(settings, gid).map(|_| ())
                        }
                        RunningAction::PurgeStopped => {
                            crate::aria2::client::purge_stopped(settings)
                        }
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
        ActionMessage::Finished {
            generation,
            target,
            result,
        } => {
            if state.finish_action(generation, target, result) {
                let Some((refresh_generation, refresh_settings)) = state.begin_downloads_refresh()
                else {
                    return Task::none();
                };

                return Task::perform(
                    async move { crate::aria2::client::fetch_download_snapshot(refresh_settings) },
                    move |result| {
                        Message::Downloads(DownloadsMessage::RefreshFinished {
                            generation: refresh_generation,
                            result,
                        })
                    },
                );
            }

            Task::none()
        }
    }
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
        AddMessage::Submit => {
            let Some((generation, settings, uri)) = state.begin_add_uri() else {
                return Task::none();
            };

            Task::perform(
                async move { crate::aria2::client::add_uri(settings, uri) },
                move |result| Message::Add(AddMessage::SubmitFinished { generation, result }),
            )
        }
        AddMessage::SubmitFinished { generation, result } => {
            if state.finish_add_uri(generation, result) {
                let Some((refresh_generation, refresh_settings)) = state.begin_downloads_refresh()
                else {
                    return Task::none();
                };

                return Task::perform(
                    async move { crate::aria2::client::fetch_download_snapshot(refresh_settings) },
                    move |result| {
                        Message::Downloads(DownloadsMessage::RefreshFinished {
                            generation: refresh_generation,
                            result,
                        })
                    },
                );
            }

            Task::none()
        }
    }
}

fn update_connection(state: &mut State, message: ConnectionMessage) -> Task<Message> {
    match message {
        ConnectionMessage::TestRequested => {
            let Some((generation, settings)) = state.begin_connection_test() else {
                return Task::none();
            };

            let settings_for_test = settings.clone();

            Task::perform(
                async move { crate::aria2::client::test_connection(settings_for_test) },
                move |result| {
                    Message::Connection(ConnectionMessage::TestFinished {
                        generation,
                        settings,
                        result,
                    })
                },
            )
        }
        ConnectionMessage::TestFinished {
            generation,
            settings,
            result,
        } => {
            if state.finish_connection_test(generation, settings, result) {
                let Some((refresh_generation, refresh_settings)) = state.begin_downloads_refresh()
                else {
                    return Task::none();
                };

                return Task::perform(
                    async move { crate::aria2::client::fetch_download_snapshot(refresh_settings) },
                    move |result| {
                        Message::Downloads(DownloadsMessage::RefreshFinished {
                            generation: refresh_generation,
                            result,
                        })
                    },
                );
            }

            Task::none()
        }
    }
}

fn update_downloads(state: &mut State, message: DownloadsMessage) -> Task<Message> {
    match message {
        DownloadsMessage::RefreshRequested => {
            let Some((generation, settings)) = state.begin_downloads_refresh() else {
                return Task::none();
            };

            Task::perform(
                async move { crate::aria2::client::fetch_download_snapshot(settings) },
                move |result| {
                    Message::Downloads(DownloadsMessage::RefreshFinished { generation, result })
                },
            )
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

fn update_toolbar(state: &mut State, message: ToolbarMessage) -> Task<Message> {
    match message {
        ToolbarMessage::OpenSettings => {
            state.open_settings();
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
            state.save_settings();
            Task::none()
        }
        SettingsMessage::EndpointChanged(endpoint) => {
            state.set_draft_endpoint(endpoint);
            Task::none()
        }
        SettingsMessage::AuthChanged(auth) => {
            state.set_draft_auth(auth);
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
    }
}
