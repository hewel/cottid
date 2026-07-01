use iced::Task;

use super::{ConnectionMessage, Message, SettingsMessage, State, StatsMessage, ToolbarMessage};

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Connection(message) => update_connection(state, message),
        Message::Stats(message) => update_stats(state, message),
        Message::Toolbar(message) => update_toolbar(state, message),
        Message::Settings(message) => update_settings(state, message),
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
            if state.finish_connection_test(generation, result) {
                let stats_generation = state.begin_stats_refresh();
                return Task::perform(
                    async move { crate::aria2::client::fetch_global_stats(settings) },
                    move |result| {
                        Message::Stats(StatsMessage::RefreshFinished {
                            generation: stats_generation,
                            result,
                        })
                    },
                );
            }

            Task::none()
        }
    }
}

fn update_stats(state: &mut State, message: StatsMessage) -> Task<Message> {
    match message {
        StatsMessage::RefreshFinished { generation, result } => {
            state.finish_stats_refresh(generation, result);
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
