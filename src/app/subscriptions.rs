use iced::Subscription;

use super::Message;

pub fn subscription(_state: &super::State) -> Subscription<Message> {
    Subscription::none()
}
