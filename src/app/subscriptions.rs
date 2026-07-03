use iced::keyboard::{Key, Modifiers, key};
use iced::{Event, Subscription, event};

use futures_util::SinkExt;
use std::pin::Pin;
use std::time::Duration;

use super::{
    AddMessage, DownloadsMessage, Message, SelectionMessage, SettingsMessage, ToolbarMessage,
    WebSocketMessage,
};

pub fn subscription(state: &super::State) -> Subscription<Message> {
    let keyboard = event::listen_with(app_event);

    if state.is_connected() {
        let mut subscriptions = vec![
            keyboard,
            iced::time::every(Duration::from_secs(
                u64::from(state.polling_interval_seconds()).max(1),
            ))
            .map(|_| Message::Downloads(DownloadsMessage::RefreshTick)),
        ];

        if let Some(endpoint) = state.websocket_subscription_endpoint() {
            subscriptions.push(Subscription::run_with(endpoint, websocket_notifications));
        }

        return Subscription::batch(subscriptions);
    }

    keyboard
}

fn websocket_notifications(
    endpoint: &String,
) -> Pin<Box<dyn iced::futures::Stream<Item = Message> + Send>> {
    let endpoint = endpoint.clone();
    Box::pin(iced::stream::channel(100, async move |output| {
        crate::aria2::websocket::listen_notifications(endpoint, |event| {
            let mut output = output.clone();
            async move {
                output
                    .send(Message::WebSocket(WebSocketMessage::Event(event)))
                    .await
                    .is_ok()
            }
        })
        .await;
    }))
}

fn app_event(event: Event, status: event::Status, _window: iced::window::Id) -> Option<Message> {
    if matches!(status, event::Status::Captured) {
        return None;
    }

    if let Event::Window(iced::window::Event::Resized(size)) = event {
        return Some(Message::WindowResized {
            width: size.width.round() as u32,
            height: size.height.round() as u32,
        });
    }

    let Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) = event else {
        return None;
    };

    match key {
        Key::Named(key::Named::Escape) => Some(Message::ModalCancel),
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

    use super::app_event;
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
            Some(Message::ModalCancel)
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

    #[test]
    fn ignores_captured_keyboard_events() {
        let message = app_event(
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                modified_key: keyboard::Key::Unidentified,
                physical_key: keyboard::key::Physical::Unidentified(
                    keyboard::key::NativeCode::Unidentified,
                ),
                location: keyboard::Location::Standard,
                modifiers: keyboard::Modifiers::empty(),
                text: None,
                repeat: false,
            }),
            iced::event::Status::Captured,
            iced::window::Id::unique(),
        );

        assert_eq!(message, None);
    }

    fn shortcut(key: keyboard::Key, modifiers: keyboard::Modifiers) -> Option<Message> {
        app_event(
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
