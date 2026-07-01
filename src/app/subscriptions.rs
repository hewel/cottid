use iced::keyboard::{Key, Modifiers, key};
use iced::{Event, Subscription, event};

use std::time::Duration;

use super::{
    AddMessage, DownloadsMessage, Message, SelectionMessage, SettingsMessage, ToolbarMessage,
};

pub fn subscription(state: &super::State) -> Subscription<Message> {
    let keyboard = event::listen_with(keyboard_shortcut);

    if state.is_connected() {
        return Subscription::batch([
            keyboard,
            iced::time::every(Duration::from_secs(
                u64::from(state.polling_interval_seconds()).max(1),
            ))
            .map(|_| Message::Downloads(DownloadsMessage::RefreshRequested)),
        ]);
    }

    keyboard
}

fn keyboard_shortcut(
    event: Event,
    _status: event::Status,
    _window: iced::window::Id,
) -> Option<Message> {
    let Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) = event else {
        return None;
    };

    match key {
        Key::Named(key::Named::Escape) => Some(Message::Add(AddMessage::Cancel)),
        Key::Character(value)
            if modifiers.contains(Modifiers::CTRL) && value.eq_ignore_ascii_case("n") =>
        {
            Some(Message::Add(AddMessage::Open))
        }
        Key::Character(value) if modifiers.contains(Modifiers::CTRL) && value == "," => {
            Some(Message::Toolbar(ToolbarMessage::OpenSettings))
        }
        Key::Character(value) if value.eq_ignore_ascii_case("r") => {
            Some(Message::Downloads(DownloadsMessage::RefreshRequested))
        }
        Key::Named(key::Named::Backspace) => Some(Message::Selection(SelectionMessage::Clear)),
        Key::Named(key::Named::Enter) => Some(Message::Add(AddMessage::Submit)),
        Key::Character(value)
            if modifiers.contains(Modifiers::CTRL) && value.eq_ignore_ascii_case("s") =>
        {
            Some(Message::Settings(SettingsMessage::Save))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use iced::keyboard;

    use super::keyboard_shortcut;
    use crate::app::{
        AddMessage, DownloadsMessage, Message, SelectionMessage, SettingsMessage, ToolbarMessage,
    };

    #[test]
    fn maps_common_keyboard_shortcuts_to_messages() {
        assert_eq!(
            shortcut(
                keyboard::Key::Named(keyboard::key::Named::Escape),
                keyboard::Modifiers::empty()
            ),
            Some(Message::Add(AddMessage::Cancel))
        );
        assert_eq!(
            shortcut(
                keyboard::Key::Character("n".into()),
                keyboard::Modifiers::CTRL
            ),
            Some(Message::Add(AddMessage::Open))
        );
        assert_eq!(
            shortcut(
                keyboard::Key::Character(",".into()),
                keyboard::Modifiers::CTRL
            ),
            Some(Message::Toolbar(ToolbarMessage::OpenSettings))
        );
        assert_eq!(
            shortcut(
                keyboard::Key::Character("r".into()),
                keyboard::Modifiers::empty()
            ),
            Some(Message::Downloads(DownloadsMessage::RefreshRequested))
        );
        assert_eq!(
            shortcut(
                keyboard::Key::Named(keyboard::key::Named::Backspace),
                keyboard::Modifiers::empty()
            ),
            Some(Message::Selection(SelectionMessage::Clear))
        );
        assert_eq!(
            shortcut(
                keyboard::Key::Named(keyboard::key::Named::Enter),
                keyboard::Modifiers::empty()
            ),
            Some(Message::Add(AddMessage::Submit))
        );
        assert_eq!(
            shortcut(
                keyboard::Key::Character("s".into()),
                keyboard::Modifiers::CTRL
            ),
            Some(Message::Settings(SettingsMessage::Save))
        );
    }

    fn shortcut(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
        keyboard_shortcut(
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key,
                modified_key: keyboard::Key::Unidentified,
                physical_key: keyboard::key::Physical::Unidentified(
                    keyboard::key::NativeCode::Unidentified,
                ),
                location: keyboard::Location::Standard,
                modifiers,
                text: None,
                repeat: false,
            }),
            iced::event::Status::Ignored,
            iced::window::Id::unique(),
        )
    }
}
