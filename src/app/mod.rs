mod message;
mod state;
mod subscriptions;
mod update;

use iced::{Element, Task};

pub use message::{ConnectionMessage, Message, SettingsMessage, ToolbarMessage};
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
    use crate::aria2::client::ConnectionTest;
    use crate::aria2::domain::VersionInfo;
    use crate::aria2::errors::ClientError;
    use crate::config::RpcAuthDraft;

    use super::{
        ConnectionMessage, ConnectionStatus, Message, SettingsMessage, State, ToolbarMessage,
    };

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
    fn settings_message_cancels_settings() {
        let mut state = State::initial();
        let _task = super::update(&mut state, Message::Toolbar(ToolbarMessage::OpenSettings));

        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Cancel));

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

    #[test]
    fn settings_draft_does_not_change_applied_settings_until_saved() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::EndpointChanged(
                "http://aria2.local:6800/jsonrpc".to_owned(),
            )),
        );

        assert_eq!(state.applied_endpoint(), "http://localhost:6800/jsonrpc");
        assert_eq!(state.draft_endpoint(), "http://aria2.local:6800/jsonrpc");

        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        assert_eq!(state.applied_endpoint(), "http://aria2.local:6800/jsonrpc");
        assert_eq!(state.draft_endpoint(), "http://aria2.local:6800/jsonrpc");
    }

    #[test]
    fn cancelling_settings_restores_draft_from_applied_settings() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::EndpointChanged(
                "http://aria2.local:6800/jsonrpc".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Cancel));

        assert_eq!(state.applied_endpoint(), "http://localhost:6800/jsonrpc");
        assert_eq!(state.draft_endpoint(), "http://localhost:6800/jsonrpc");
        assert!(!state.is_settings_open());
    }

    #[test]
    fn invalid_endpoint_feedback_stays_in_settings_state() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::EndpointChanged(
                "ftp://localhost:6800/jsonrpc".to_owned(),
            )),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        assert_eq!(
            state.settings_feedback(),
            Some("Endpoint must start with http:// or https://.")
        );
        assert!(state.is_settings_open());
        assert_eq!(state.applied_endpoint(), "http://localhost:6800/jsonrpc");
    }

    #[test]
    fn settings_can_choose_session_secret_without_displaying_secret() {
        let mut state = State::initial();

        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::AuthChanged(RpcAuthDraft::SessionSecret)),
        );
        let _task = super::update(
            &mut state,
            Message::Settings(SettingsMessage::SecretChanged("super-secret".to_owned())),
        );
        let _task = super::update(&mut state, Message::Settings(SettingsMessage::Save));

        assert_eq!(state.applied_auth_label(), "Token secret");
        assert!(!state.status_text().contains("super-secret"));
    }

    #[test]
    fn connection_test_result_updates_visible_connection_state() {
        let mut state = State::initial();
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestRequested),
        );

        assert_eq!(state.connection_status(), ConnectionStatus::Testing);

        let result = Ok(ConnectionTest::new(VersionInfo::new("1.37.0", Vec::new())));
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestFinished {
                generation: 1,
                result,
            }),
        );

        assert_eq!(state.connection_status(), ConnectionStatus::Connected);
        assert_eq!(state.connected_version(), Some("1.37.0"));
        assert_eq!(
            state.settings_feedback(),
            Some("Connection test succeeded.")
        );
    }

    #[test]
    fn stale_connection_test_results_are_ignored() {
        let mut state = State::initial();
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestRequested),
        );
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestRequested),
        );

        let stale_result = Ok(ConnectionTest::new(VersionInfo::new("1.36.0", Vec::new())));
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestFinished {
                generation: 1,
                result: stale_result,
            }),
        );

        assert_eq!(state.connection_status(), ConnectionStatus::Testing);
        assert_eq!(state.connected_version(), None);

        let current_result = Err(ClientError::Transport("connection refused".to_owned()));
        let _task = super::update(
            &mut state,
            Message::Connection(ConnectionMessage::TestFinished {
                generation: 2,
                result: current_result,
            }),
        );

        assert_eq!(state.connection_status(), ConnectionStatus::Failed);
        assert_eq!(
            state.settings_feedback(),
            Some("Connection test failed. Check the endpoint and secret.")
        );
    }
}
