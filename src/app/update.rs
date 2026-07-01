use iced::Task;

use super::{Message, SettingsMessage, State, ToolbarMessage};

pub fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Toolbar(message) => update_toolbar(state, message),
        Message::Settings(message) => update_settings(state, message),
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
