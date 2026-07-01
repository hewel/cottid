use iced::Subscription;

use std::time::Duration;

use super::{DownloadsMessage, Message};

pub fn subscription(state: &super::State) -> Subscription<Message> {
    if !state.is_connected() {
        return Subscription::none();
    }

    iced::time::every(Duration::from_secs(
        u64::from(state.polling_interval_seconds()).max(1),
    ))
    .map(|_| Message::Downloads(DownloadsMessage::RefreshRequested))
}
