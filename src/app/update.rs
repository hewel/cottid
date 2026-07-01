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
        SettingsMessage::Close => {
            state.close_settings();
            Task::none()
        }
    }
}
