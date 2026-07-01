mod message;
mod state;
mod subscriptions;
mod update;

use iced::{Element, Task};

pub use message::{Message, SettingsMessage, ToolbarMessage};
pub use state::{ConnectionStatus, State};

pub fn run() -> iced::Result {
    iced::application(State::initial, update, view)
        .title("Cottid")
        .subscription(subscription)
        .run()
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
    use super::{ConnectionStatus, Message, SettingsMessage, State, ToolbarMessage};

    #[test]
    fn starts_offline_and_settings_ready() {
        let state = State::initial();

        assert_eq!(state.connection_status(), ConnectionStatus::Offline);
        assert!(state.is_settings_ready());
    }

    #[test]
    fn toolbar_message_opens_settings_without_changing_connection() {
        let mut state = State::initial();

        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));

        assert!(state.is_settings_open());
        assert_eq!(state.connection_status(), ConnectionStatus::Offline);
    }

    #[test]
    fn settings_message_closes_settings() {
        let mut state = State::initial();
        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));

        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Close));

        assert!(!state.is_settings_open());
    }

    #[test]
    fn subscription_hook_is_available_before_polling_exists() {
        let state = State::initial();

        let _subscription = super::subscription(&state);
    }

    #[test]
    fn view_builds_from_normalized_app_state() {
        let state = State::initial();

        let _element = super::view(&state);
    }
}
