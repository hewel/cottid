use iced::widget::{button, column, container, row, space, text, text_input};
use iced::{Alignment, Element, Length};

use crate::app::{
    ActionMessage, AddMessage, ConnectionMessage, ConnectionStatus, DownloadsMessage, FeedbackTone,
    FormFeedback, Message, SettingsMessage, State, ToolbarMessage,
};
use crate::config::{RpcAuthDraft, ThemePreference};
use crate::ui::icons::{Icon, icon};
use crate::ui::theme;

pub fn view(state: &State) -> Element<'_, Message> {
    let sidebar_width = if state.is_compact_layout() {
        92.0
    } else {
        248.0
    };
    let sidebar = sidebar(state, state.is_compact_layout()).width(Length::Fixed(sidebar_width));
    let downloads = crate::ui::downloads::view(state);

    let main = row![sidebar, downloads]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill);

    let mut shell = column![main]
        .padding(10)
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
            row![
                container(text("C").size(16))
                    .style(theme::muted_surface)
                    .padding([6, 9]),
                text("Cottid").size(24),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(4)
    };

    let purge = nav_button(
        Icon::Purge,
        if compact { "" } else { "Clear results" },
        state.can_purge_stopped(),
        || Message::Action(ActionMessage::PurgeStopped),
    );

    let mut filters = column![].spacing(6);
    for filter in crate::app::DownloadFilter::ALL
        .into_iter()
        .filter(|filter| *filter != crate::app::DownloadFilter::All)
    {
        let label = if compact {
            filter.label().chars().next().unwrap_or('?').to_string()
        } else {
            filter.label().to_owned()
        };
        let selected = filter == state.selected_filter();
        filters = filters.push(filter_button(
            label,
            state.filter_count(filter),
            selected,
            move || Message::Downloads(DownloadsMessage::FilterChanged(filter)),
        ));
    }

    let main_content = column![
        title,
        column![purge].spacing(8),
        text(if compact { "State" } else { "Download state" })
            .size(12)
            .style(theme::muted_text),
        filters,
    ]
    .spacing(24)
    .width(Length::Fill);

    let mut bottom_content = column![].spacing(10).width(Length::Fill);

    if !compact {
        bottom_content = bottom_content.push(connection_status_card(state));
    }

    bottom_content = bottom_content.push(row![settings_icon_button()].width(Length::Fill));

    let content = column![main_content, space::vertical(), bottom_content]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill);

    container(content)
        .style(theme::sidebar)
        .padding(18)
        .height(Length::Fill)
}

fn nav_button(
    icon_kind: Icon,
    label: &'static str,
    enabled: bool,
    message: impl FnOnce() -> Message,
) -> button::Button<'static, Message> {
    let content = if label.is_empty() {
        row![icon(icon_kind, 18, theme::text_color)].align_y(Alignment::Center)
    } else {
        row![icon(icon_kind, 18, theme::text_color), text(label).size(14)]
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

fn connection_status_card(state: &State) -> Element<'_, Message> {
    container(
        column![
            text("Connection").size(12).style(theme::muted_text),
            text(state.status_text()).size(13),
            text(state.applied_auth_label())
                .size(12)
                .style(theme::muted_text),
            text(state.applied_endpoint())
                .size(12)
                .style(theme::muted_text),
            text(state.counts_text()).size(12).style(theme::muted_text),
            text(format!("Down {}", state.download_speed_text()))
                .size(12)
                .style(theme::muted_text),
            text(format!("Up {}", state.upload_speed_text()))
                .size(12)
                .style(theme::muted_text),
        ]
        .spacing(4),
    )
    .style(theme::muted_surface)
    .padding(12)
    .width(Length::Fill)
    .into()
}

fn settings_icon_button() -> button::Button<'static, Message> {
    button(icon(Icon::Settings, 18, theme::text_color))
        .padding(10)
        .style(theme::subtle_button)
        .on_press(Message::Toolbar(ToolbarMessage::OpenSettings))
}

fn filter_button(
    label: String,
    count: usize,
    selected: bool,
    message: impl FnOnce() -> Message,
) -> Element<'static, Message> {
    let content = if label.len() == 1 {
        row![text(label).size(14)].align_y(Alignment::Center)
    } else {
        row![
            text(label).size(14).width(Length::Fill),
            text(count.to_string()).size(13).style(theme::muted_text),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    };
    let button = button(content)
        .padding([9, 12])
        .width(Length::Fill)
        .style(if selected {
            theme::selected_button
        } else {
            theme::subtle_button
        });

    button.on_press(message()).into()
}

fn add_modal(state: &State) -> Element<'_, Message> {
    let input = text_input(
        "https://example.com/file.iso or magnet:?",
        state.add_input(),
    )
    .on_input(|value| Message::Add(AddMessage::InputChanged(value)))
    .padding(10)
    .style(theme::form_text_input);

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

    let add_feedback = state
        .add_feedback()
        .map(form_feedback_banner)
        .unwrap_or_else(|| feedback_banner(FeedbackTone::Info, "Enter one URI or magnet link."));

    container(
        column![
            text("Add Download").size(20),
            input,
            add_feedback,
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
        .padding(10)
        .style(theme::form_text_input);

    let polling = text_input("2", &state.draft_polling_interval_seconds().to_string())
        .on_input(|value| Message::Settings(SettingsMessage::PollingIntervalChanged(value)))
        .padding(10)
        .style(theme::form_text_input);

    let secret = text_input("Session token", state.draft_secret())
        .on_input(|value| Message::Settings(SettingsMessage::SecretChanged(value)))
        .padding(10)
        .style(theme::form_text_input);

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
        text("RPC endpoint").size(12).style(theme::muted_text),
        endpoint,
        text("Authentication").size(12).style(theme::muted_text),
        auth_row,
        text("Theme").size(12).style(theme::muted_text),
        theme_row(state.draft_theme_preference()),
    ]
    .spacing(8);

    if matches!(state.draft_auth(), RpcAuthDraft::SessionSecret) {
        fields = fields
            .push(text("Secret").size(12).style(theme::muted_text))
            .push(secret);
    }

    let settings_feedback = state
        .settings_feedback()
        .map(form_feedback_banner)
        .unwrap_or_else(|| feedback_banner(FeedbackTone::Info, "Settings are not persisted yet."));

    fields = fields
        .push(text("Polling interval").size(12).style(theme::muted_text))
        .push(polling)
        .push(settings_feedback);

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
        .style(if auth == selected {
            theme::selected_button
        } else {
            theme::subtle_button
        })
        .on_press(Message::Settings(SettingsMessage::AuthChanged(auth)))
        .into()
}

fn form_feedback_banner(feedback: &FormFeedback) -> Element<'static, Message> {
    feedback_banner(feedback.tone(), feedback.message())
}

fn feedback_banner(tone: FeedbackTone, message: &str) -> Element<'static, Message> {
    let (icon_kind, color, surface) = match tone {
        FeedbackTone::Info => (
            Icon::Info,
            theme::feedback_info_color as fn(&iced::Theme) -> iced::Color,
            theme::feedback_info_surface as fn(&iced::Theme) -> container::Style,
        ),
        FeedbackTone::Success => (
            Icon::CheckCircle,
            theme::feedback_success_color as fn(&iced::Theme) -> iced::Color,
            theme::feedback_success_surface as fn(&iced::Theme) -> container::Style,
        ),
        FeedbackTone::Warning => (
            Icon::Error,
            theme::feedback_warning_color as fn(&iced::Theme) -> iced::Color,
            theme::feedback_warning_surface as fn(&iced::Theme) -> container::Style,
        ),
        FeedbackTone::Error => (
            Icon::XCircle,
            theme::feedback_error_color as fn(&iced::Theme) -> iced::Color,
            theme::feedback_error_surface as fn(&iced::Theme) -> container::Style,
        ),
    };

    container(
        row![
            icon(icon_kind, 16, color),
            text(message.to_owned()).size(12),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .style(surface)
    .padding([10, 12])
    .width(Length::Fill)
    .into()
}

fn theme_row(selected: ThemePreference) -> Element<'static, Message> {
    let mut row = row![].spacing(8);

    for preference in ThemePreference::ALL {
        row = row.push(theme_button(preference, selected));
    }

    row.into()
}

fn theme_button(
    preference: ThemePreference,
    selected: ThemePreference,
) -> button::Button<'static, Message> {
    let label = if preference == selected {
        format!("{} selected", preference.label())
    } else {
        preference.label().to_owned()
    };

    button(text(label))
        .style(if preference == selected {
            theme::selected_button
        } else {
            theme::subtle_button
        })
        .on_press(Message::Settings(SettingsMessage::ThemePreferenceChanged(
            preference,
        )))
}

fn connection_label(status: ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::Offline => "Offline",
        ConnectionStatus::Testing => "Testing...",
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Failed => "Connection failed",
    }
}
