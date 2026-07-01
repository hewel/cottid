use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::app::{ConnectionStatus, Message, SettingsMessage, State, ToolbarMessage};
use crate::config::RpcAuthDraft;

pub fn view(state: &State) -> Element<'_, Message> {
    let toolbar = row![
        text("Cottid").size(24),
        text(state.applied_endpoint()),
        text(state.applied_auth_label()),
        text(connection_label(state.connection_status())),
        button("Settings").on_press(Message::Toolbar(ToolbarMessage::OpenSettings)),
    ]
    .align_y(Alignment::Center)
    .spacing(16)
    .width(Length::Fill);

    let main_content = column![
        text("Offline").size(20),
        text("Configure an aria2 RPC endpoint to get started."),
    ]
    .spacing(8)
    .width(Length::Fill);

    let status = row![
        text(state.status_text()),
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

    container(shell)
        .width(Length::Fill)
        .height(Length::Fill)
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

    container(
        column![
            fields,
            row![
                button("Test Connection"),
                button("Save").on_press(Message::Settings(SettingsMessage::Save)),
                button("Cancel").on_press(Message::Settings(SettingsMessage::Cancel)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(16),
    )
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
    }
}
