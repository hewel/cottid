use iced::widget::{button, column, container, row, text, text_input};
use iced::{Alignment, Element, Length};

use crate::app::{
    AddMessage, ConnectionMessage, ConnectionStatus, DownloadsMessage, Message, SettingsMessage,
    State, ToolbarMessage,
};
use crate::config::RpcAuthDraft;
use crate::ui::icons::{Icon, icon};
use crate::ui::theme;

pub fn view(state: &State) -> Element<'_, Message> {
    let sidebar_width = if state.is_compact_layout() {
        96.0
    } else {
        260.0
    };
    let sidebar = sidebar(state, state.is_compact_layout()).width(Length::Fixed(sidebar_width));
    let downloads = crate::ui::downloads::view(state);

    let main = row![sidebar, downloads]
        .spacing(18)
        .width(Length::Fill)
        .height(Length::Fill);

    let mut shell = column![main, status_strip(state)]
        .spacing(14)
        .padding(18)
        .width(Length::Fill)
        .height(Length::Fill);

    if state.is_settings_open() {
        shell = shell.push(settings_modal(state));
    }

    if state.is_add_open() {
        shell = shell.push(add_modal(state));
    }

    container(shell)
        .style(theme::app_background)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn sidebar(state: &State, compact: bool) -> container::Container<'_, Message> {
    let title = if compact {
        column![
            text("C").size(28),
            text(connection_label(state.connection_status())).size(11)
        ]
        .spacing(2)
        .align_x(Alignment::Center)
    } else {
        column![
            text("Cottid").size(26),
            text(state.applied_endpoint())
                .size(12)
                .color(theme::TEXT_MUTED),
            text(connection_label(state.connection_status())).size(12),
        ]
        .spacing(4)
    };

    let add = nav_button(Icon::Add, if compact { "" } else { "Add" }, true, || {
        Message::Add(AddMessage::Open)
    })
    .style(theme::primary_button);
    let refresh = nav_button(
        Icon::Refresh,
        if compact { "" } else { "Refresh" },
        true,
        || Message::Downloads(DownloadsMessage::RefreshRequested),
    );
    let settings = nav_button(
        Icon::Settings,
        if compact { "" } else { "Settings" },
        true,
        || Message::Toolbar(ToolbarMessage::OpenSettings),
    );

    let purge = nav_button(
        Icon::Purge,
        if compact { "" } else { "Purge" },
        state.can_purge_stopped(),
        || Message::Action(crate::app::ActionMessage::PurgeStopped),
    );

    let mut filters = column![].spacing(6);
    for filter in crate::app::DownloadFilter::ALL {
        let label = if compact {
            filter.label().chars().next().unwrap_or('?').to_string()
        } else {
            format!("{} {}", filter.label(), state.filter_count(filter))
        };
        let selected = filter == state.selected_filter();
        filters = filters.push(filter_button(label, selected, move || {
            Message::Downloads(DownloadsMessage::FilterChanged(filter))
        }));
    }

    let mut content = column![
        title,
        column![add, refresh, purge, settings].spacing(8),
        text(if compact { "Views" } else { "Library" })
            .size(12)
            .color(theme::TEXT_MUTED),
        filters,
    ]
    .spacing(18)
    .width(Length::Fill);

    if !compact {
        content = content.push(
            container(
                column![
                    text("Connection").size(12).color(theme::TEXT_MUTED),
                    text(state.status_text()).size(13),
                    text(state.applied_auth_label())
                        .size(12)
                        .color(theme::TEXT_MUTED),
                ]
                .spacing(4),
            )
            .style(theme::muted_surface)
            .padding(12)
            .width(Length::Fill),
        );
    }

    container(content)
        .style(theme::sidebar)
        .padding(14)
        .height(Length::Fill)
}

fn nav_button(
    icon_kind: Icon,
    label: &'static str,
    enabled: bool,
    message: impl FnOnce() -> Message,
) -> button::Button<'static, Message> {
    let content = if label.is_empty() {
        row![icon(icon_kind, 18, theme::TEXT)].align_y(Alignment::Center)
    } else {
        row![icon(icon_kind, 18, theme::TEXT), text(label).size(14)]
            .spacing(8)
            .align_y(Alignment::Center)
    };
    let button = button(content)
        .padding([10, 12])
        .width(Length::Fill)
        .style(theme::subtle_button);

    if enabled {
        button.on_press(message())
    } else {
        button
    }
}

fn filter_button(
    label: String,
    selected: bool,
    message: impl FnOnce() -> Message,
) -> Element<'static, Message> {
    let button = button(text(label).size(14))
        .padding([8, 10])
        .width(Length::Fill)
        .style(if selected {
            theme::primary_button
        } else {
            theme::subtle_button
        });

    button.on_press(message()).into()
}

fn status_strip(state: &State) -> Element<'_, Message> {
    let feedback = state.refresh_feedback().unwrap_or("");
    container(
        row![
            text(state.status_text()).size(12),
            text(format!("Down {}", state.download_speed_text())).size(12),
            text(format!("Up {}", state.upload_speed_text())).size(12),
            text(state.refresh_state_text()).size(12),
            text(state.counts_text()).size(12),
            text(feedback).size(12).color(theme::RED),
        ]
        .spacing(16)
        .align_y(Alignment::Center),
    )
    .style(theme::status_strip)
    .padding([8, 12])
    .width(Length::Fill)
    .into()
}

fn add_modal(state: &State) -> Element<'_, Message> {
    let input = text_input(
        "https://example.com/file.iso or magnet:?",
        state.add_input(),
    )
    .on_input(|value| Message::Add(AddMessage::InputChanged(value)))
    .padding(10);

    let submit = if state.is_add_ready() {
        button(text(if state.is_add_pending() {
            "Adding"
        } else {
            "Add download"
        }))
        .style(theme::primary_button)
        .on_press(Message::Add(AddMessage::Submit))
    } else {
        button(text(if state.is_add_pending() {
            "Adding"
        } else {
            "Add download"
        }))
        .style(theme::primary_button)
    };

    container(
        column![
            text("Add Download").size(20),
            input,
            text(
                state
                    .add_feedback()
                    .unwrap_or("Enter one URI or magnet link.")
            )
            .size(12)
            .color(theme::TEXT_MUTED),
            row![
                submit,
                button("Cancel")
                    .style(theme::subtle_button)
                    .on_press(Message::Add(AddMessage::Cancel)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(10),
    )
    .style(theme::surface)
    .padding(18)
    .width(Length::Fill)
    .into()
}

fn settings_modal(state: &State) -> Element<'_, Message> {
    let endpoint = text_input("http://localhost:6800/jsonrpc", state.draft_endpoint())
        .on_input(|value| Message::Settings(SettingsMessage::EndpointChanged(value)))
        .padding(10);

    let polling = text_input("2", &state.draft_polling_interval_seconds().to_string())
        .on_input(|value| Message::Settings(SettingsMessage::PollingIntervalChanged(value)))
        .padding(10);

    let secret = text_input("Session token", state.draft_secret())
        .on_input(|value| Message::Settings(SettingsMessage::SecretChanged(value)))
        .padding(10);

    let auth_row = row![
        auth_button(
            "No authentication",
            RpcAuthDraft::NoSecret,
            state.draft_auth()
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
        text("RPC endpoint").size(12).color(theme::TEXT_MUTED),
        endpoint,
        text("Authentication").size(12).color(theme::TEXT_MUTED),
        auth_row,
    ]
    .spacing(8);

    if matches!(state.draft_auth(), RpcAuthDraft::SessionSecret) {
        fields = fields
            .push(text("Secret").size(12).color(theme::TEXT_MUTED))
            .push(secret);
    }

    fields = fields
        .push(text("Polling interval").size(12).color(theme::TEXT_MUTED))
        .push(polling)
        .push(
            text(
                state
                    .settings_feedback()
                    .unwrap_or("Settings are not persisted yet."),
            )
            .size(12)
            .color(theme::TEXT_MUTED),
        );

    let mut actions = row![
        button("Test Connection")
            .style(theme::subtle_button)
            .on_press(Message::Connection(ConnectionMessage::TestRequested)),
        button("Save")
            .style(theme::primary_button)
            .on_press(Message::Settings(SettingsMessage::Save)),
        button("Cancel")
            .style(theme::subtle_button)
            .on_press(Message::Settings(SettingsMessage::Cancel)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    if state.is_plaintext_fallback_pending() {
        actions = actions
            .push(
                button("Save Plaintext")
                    .style(theme::subtle_button)
                    .on_press(Message::Settings(SettingsMessage::SavePlaintextFallback)),
            )
            .push(
                button("Session Only")
                    .style(theme::subtle_button)
                    .on_press(Message::Settings(SettingsMessage::KeepSecretSessionOnly)),
            );
    }

    container(column![fields, actions].spacing(16))
        .style(theme::surface)
        .padding(18)
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
        .style(theme::subtle_button)
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
