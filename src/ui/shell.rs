use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::app::{
    AddMessage, ConnectionMessage, ConnectionStatus, Message, SettingsMessage, State,
    ToolbarMessage,
};
use crate::config::RpcAuthDraft;

pub fn view(state: &State) -> Element<'_, Message> {
    let toolbar = row![
        text("Cottid").size(24),
        text(state.applied_endpoint()),
        text(state.applied_auth_label()),
        text(connection_label(state.connection_status())),
        text(format!("Down {}", state.download_speed_text())),
        text(format!("Up {}", state.upload_speed_text())),
        text(state.refresh_state_text()),
        button("Add").on_press(Message::Add(AddMessage::Open)),
        button("Settings").on_press(Message::Toolbar(ToolbarMessage::OpenSettings)),
    ]
    .align_y(Alignment::Center)
    .spacing(16)
    .width(Length::Fill);

    let main_content = crate::ui::downloads::view(state);

    let status = row![
        text(state.status_text()),
        text(state.refresh_state_text()),
        text(state.counts_text()),
        text(state.refresh_feedback().unwrap_or("")),
        text(if state.is_settings_ready() {
            "Settings ready"
        } else {
            "Settings incomplete"
        }),
    ]
    .spacing(16)
    .width(Length::Fill);

    let mut shell = column![toolbar, main_content, status]
        .spacing(24)
        .padding(24)
        .width(Length::Fill)
        .height(Length::Fill);

    if state.is_settings_open() {
        shell = shell.push(settings_modal(state));
    }

    if state.is_add_open() {
        shell = shell.push(add_modal(state));
    }

    container(shell)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn add_modal(state: &State) -> Element<'_, Message> {
    let input = text_input(
        "https://example.com/file.iso or magnet:?",
        state.add_input(),
    )
    .on_input(|value| Message::Add(AddMessage::InputChanged(value)))
    .padding(8);

    let submit = if state.is_add_ready() {
        button(if state.is_add_pending() {
            "Adding"
        } else {
            "Add"
        })
        .on_press(Message::Add(AddMessage::Submit))
    } else {
        button(if state.is_add_pending() {
            "Adding"
        } else {
            "Add"
        })
    };

    container(
        column![
            text("Add Download").size(20),
            text("URI or magnet"),
            input,
            text(
                state
                    .add_feedback()
                    .unwrap_or("Enter one URI or magnet link.")
            ),
            row![
                submit,
                button("Cancel").on_press(Message::Add(AddMessage::Cancel)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(8),
    )
    .padding(16)
    .width(Length::Fill)
    .into()
}

fn settings_modal(state: &State) -> Element<'_, Message> {
    let endpoint = text_input("http://localhost:6800/jsonrpc", state.draft_endpoint())
        .on_input(|value| Message::Settings(SettingsMessage::EndpointChanged(value)))
        .padding(8);

    let polling = text_input("2", &state.draft_polling_interval_seconds().to_string())
        .on_input(|value| Message::Settings(SettingsMessage::PollingIntervalChanged(value)))
        .padding(8);

    let secret = text_input("Session token", state.draft_secret())
        .on_input(|value| Message::Settings(SettingsMessage::SecretChanged(value)))
        .padding(8);

    let auth_row = row![
        auth_button(
            "No authentication",
            RpcAuthDraft::NoSecret,
            state.draft_auth(),
        ),
        auth_button(
            "Token secret",
            RpcAuthDraft::SessionSecret,
            state.draft_auth()
        ),
    ]
    .spacing(8);

    let mut fields = column![
        text("Connection Settings").size(20),
        text("RPC endpoint"),
        endpoint,
        text("Authentication"),
        auth_row,
    ]
    .spacing(8);

    if matches!(state.draft_auth(), RpcAuthDraft::SessionSecret) {
        fields = fields.push(text("Secret")).push(secret);
    }

    fields = fields
        .push(text("Polling interval"))
        .push(polling)
        .push(text(
            state
                .settings_feedback()
                .unwrap_or("Settings are not persisted yet."),
        ));

    let mut actions = row![
        button("Test Connection").on_press(Message::Connection(ConnectionMessage::TestRequested)),
        button("Save").on_press(Message::Settings(SettingsMessage::Save)),
        button("Cancel").on_press(Message::Settings(SettingsMessage::Cancel)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    if state.is_plaintext_fallback_pending() {
        actions = actions
            .push(
                button("Save Plaintext")
                    .on_press(Message::Settings(SettingsMessage::SavePlaintextFallback)),
            )
            .push(
                button("Session Only")
                    .on_press(Message::Settings(SettingsMessage::KeepSecretSessionOnly)),
            );
    }

    container(column![fields, actions].spacing(16))
        .padding(16)
        .width(Length::Fill)
        .into()
}

fn auth_button(
    label: &'static str,
    auth: RpcAuthDraft,
    selected: RpcAuthDraft,
) -> Element<'static, Message> {
    let label = if auth == selected {
        format!("{label} selected")
    } else {
        label.to_owned()
    };

    button(text(label))
        .on_press(Message::Settings(SettingsMessage::AuthChanged(auth)))
        .into()
}

fn connection_label(status: ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::Offline => "Offline",
        ConnectionStatus::Testing => "Testing...",
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Failed => "Connection failed",
    }
}
