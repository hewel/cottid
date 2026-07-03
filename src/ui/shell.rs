use iced::widget::{button, column, container, row, space, stack, text};
use iced::{Alignment, Element, Length};

use crate::app::{
    AddMessage, ConnectionMessage, ConnectionStatus, DownloadsMessage, Message, SettingsMessage,
    State, TextInputFocusTarget, ToolbarMessage,
};
use crate::config::{RpcAuthDraft, ThemePreference};
use crate::ui::components as ui;
use crate::ui::icons::{Icon, icon};
use crate::ui::overlay::{
    Alignment as OverlayAlignment, Placement, PopoverId, PopoverOptions, TooltipOptions,
    app_popover, app_tooltip, app_tooltip_element,
};
use crate::ui::theme;
use crate::ui::variants::{BadgeVariant, ButtonVariant};
use crate::ui::widgets::field::{
    FieldOptions, FieldStatus, FieldStatusKind, FieldStatusVariant, Requiredness, text_field,
};

const CONNECTION_DETAIL_POPOVER: PopoverId = PopoverId(1);

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

    let shell = column![main]
        .padding(10)
        .width(Length::Fill)
        .height(Length::Fill);

    let base = ui::app_surface(shell)
        .width(Length::Fill)
        .height(Length::Fill);

    let mut layers = vec![base.into()];

    if state.is_settings_open() {
        layers.push(ui::modal_layer(
            settings_modal(state),
            state.modal_max_width(640.0),
            state.modal_max_height(),
        ));
    } else if state.is_add_open() {
        layers.push(ui::modal_layer(
            add_modal(state),
            state.modal_max_width(400.0),
            state.modal_max_height(),
        ));
    }

    stack(layers)
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
                ui::muted_panel(text("C").size(16)).padding([6, 9]),
                text("Cottid").size(24),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(4)
    };

    let mut filters = column![].spacing(6);
    for filter in crate::app::DownloadFilter::VISIBLE {
        let selected = filter == state.selected_filter();
        filters = filters.push(filter_button(
            filter_icon(filter),
            if compact { None } else { Some(filter.label()) },
            state.filter_count(filter),
            selected,
            move || Message::Downloads(DownloadsMessage::FilterChanged(filter)),
        ));
    }

    let main_content = column![
        title,
        text(if compact { "State" } else { "Download state" })
            .size(12)
            .style(theme::muted_text),
        filters,
    ]
    .spacing(24)
    .width(Length::Fill);

    let bottom_content = column![
        theme_switcher(state),
        row![connection_detail_popover(state)].width(Length::Fill),
        row![settings_icon_button()].width(Length::Fill),
    ]
    .spacing(10)
    .width(Length::Fill);

    let content = column![main_content, space::vertical(), bottom_content]
        .spacing(16)
        .width(Length::Fill)
        .height(Length::Fill);

    ui::sidebar_surface(content)
        .padding(18)
        .height(Length::Fill)
}

fn theme_switcher(state: &State) -> Element<'static, Message> {
    let selected = state.theme_preference();

    app_tooltip(
        compact_theme_cycle_button(selected),
        format!(
            "Theme: {}. Switch to {}",
            selected.label(),
            selected.next().label()
        ),
        TooltipOptions::default(),
    )
}

fn compact_theme_cycle_button(selected: ThemePreference) -> button::Button<'static, Message> {
    theme_icon_control(theme_icon(selected))
}

fn theme_icon_control(icon_kind: Icon) -> button::Button<'static, Message> {
    ui::icon_button(
        icon_kind,
        Message::Toolbar(ToolbarMessage::CycleThemePreference),
    )
}

fn theme_icon(preference: ThemePreference) -> Icon {
    match preference {
        ThemePreference::System => Icon::SystemTheme,
        ThemePreference::Light => Icon::Sun,
        ThemePreference::Dark => Icon::Moon,
    }
}

fn connection_detail_content(state: &State) -> Element<'static, Message> {
    column![
        text("Connection").size(12).style(theme::muted_text),
        text(state.status_text()).size(13),
        text(state.applied_auth_label().to_owned())
            .size(12)
            .style(theme::muted_text),
        text(state.applied_endpoint().to_owned())
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
    .spacing(4)
    .width(Length::Fill)
    .into()
}

fn connection_detail_popover(state: &State) -> Element<'_, Message> {
    let is_open = state.is_popover_open(CONNECTION_DETAIL_POPOVER);
    let trigger = app_tooltip_element(
        ui::icon_button(Icon::Cpu, Message::TogglePopover(CONNECTION_DETAIL_POPOVER)),
        ui::transfer_speed_summary(
            state.download_speed_text(),
            state.upload_speed_text(),
            None,
            ui::TransferSpeedTone::Tooltip,
        ),
        TooltipOptions {
            enabled: !is_open,
            ..TooltipOptions::default()
        },
    );

    app_popover(
        CONNECTION_DETAIL_POPOVER,
        trigger,
        connection_detail_content(state),
        is_open,
        PopoverOptions {
            placement: Placement::Above,
            alignment: OverlayAlignment::Start,
            width: Some(240.0),
            ..PopoverOptions::default()
        },
        Message::ClosePopover,
    )
}

fn settings_icon_button() -> Element<'static, Message> {
    app_tooltip(
        ui::icon_button(
            Icon::Settings,
            Message::Toolbar(ToolbarMessage::OpenSettings),
        ),
        "Connection and app settings",
        TooltipOptions::default(),
    )
}

fn filter_button(
    icon_kind: Icon,
    label: Option<&'static str>,
    count: usize,
    selected: bool,
    message: impl FnOnce() -> Message,
) -> Element<'static, Message> {
    let content = if let Some(label) = label {
        row![
            icon(icon_kind, 18.0, theme::text_color),
            text(label).size(14).width(Length::Fill),
            ui::badge(count.to_string(), BadgeVariant::Neutral),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    } else {
        row![icon(icon_kind, 18.0, theme::text_color)]
            .align_y(Alignment::Center)
            .width(Length::Fill)
    };
    let button = ui::toggle_button(content, selected)
        .padding([9, 12])
        .width(Length::Fill);

    button.on_press(message()).into()
}

fn filter_icon(filter: crate::app::DownloadFilter) -> Icon {
    match filter {
        crate::app::DownloadFilter::Active => Icon::SpinnerGap,
        crate::app::DownloadFilter::Complete => Icon::CheckCircle,
        crate::app::DownloadFilter::All
        | crate::app::DownloadFilter::Waiting
        | crate::app::DownloadFilter::Paused
        | crate::app::DownloadFilter::Error => unreachable!(),
    }
}

fn add_modal(state: &State) -> Element<'_, Message> {
    let input = text_field(
        FieldOptions {
            description: Some("HTTP, HTTPS, or magnet link."),
            requiredness: Requiredness::Required,
            is_disabled: state.is_add_pending(),
            label_action: Some(Message::FocusTextInput(TextInputFocusTarget::AddUri)),
            status: state.add_input_validation_message().map(field_error),
            ..FieldOptions::new("Download URI")
        },
        "https://example.com/file.iso or magnet:?",
        state.add_input(),
        Some(TextInputFocusTarget::AddUri.id_value()),
        |value| Message::Add(AddMessage::InputChanged(value)),
    );

    let submit = if state.is_add_ready() {
        ui::text_button(
            if state.is_add_pending() {
                "Adding"
            } else {
                "Add download"
            },
            ButtonVariant::Primary,
        )
        .on_press(Message::Add(AddMessage::Submit))
    } else {
        ui::text_button(
            if state.is_add_pending() {
                "Adding"
            } else {
                "Add download"
            },
            ButtonVariant::Primary,
        )
    };

    let mut content = column![text("Add Download").size(20), input,].spacing(10);

    if let Some(feedback) = state.add_feedback() {
        content = content.push(ui::form_feedback_banner(feedback));
    }

    content = content.push(
        row![
            submit,
            ui::text_button("Cancel", ButtonVariant::Secondary)
                .on_press(Message::Add(AddMessage::Cancel)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    );

    ui::modal_surface(content)
        .padding(18)
        .width(Length::Fill)
        .into()
}

fn settings_modal(state: &State) -> Element<'_, Message> {
    let is_testing = matches!(state.connection_status(), ConnectionStatus::Testing);
    let endpoint = text_field(
        FieldOptions {
            description: Some("aria2 JSON-RPC endpoint."),
            requiredness: Requiredness::Required,
            is_disabled: is_testing,
            label_tooltip: Some(
                "Use the endpoint exposed by an already-running aria2c RPC server.",
            ),
            label_action: Some(Message::FocusTextInput(
                TextInputFocusTarget::SettingsEndpoint,
            )),
            status: state.draft_endpoint_validation_message().map(field_error),
            ..FieldOptions::new("RPC endpoint")
        },
        "http://localhost:6800/jsonrpc",
        state.draft_endpoint(),
        Some(TextInputFocusTarget::SettingsEndpoint.id_value()),
        |value| Message::Settings(SettingsMessage::EndpointChanged(value)),
    );

    let polling_value = state.draft_polling_interval_seconds().to_string();
    let polling = text_field(
        FieldOptions {
            description: Some("Seconds between scheduled refreshes."),
            requiredness: Requiredness::Required,
            is_disabled: is_testing,
            label_action: Some(Message::FocusTextInput(
                TextInputFocusTarget::SettingsPollingInterval,
            )),
            ..FieldOptions::new("Polling interval")
        },
        "2",
        &polling_value,
        Some(TextInputFocusTarget::SettingsPollingInterval.id_value()),
        |value| Message::Settings(SettingsMessage::PollingIntervalChanged(value)),
    );

    let secret = text_field(
        FieldOptions {
            description: Some("aria2 token secret for this endpoint."),
            requiredness: Requiredness::Required,
            is_disabled: is_testing,
            label_action: Some(Message::FocusTextInput(
                TextInputFocusTarget::SettingsSecret,
            )),
            status: state.draft_secret_validation_message().map(field_error),
            ..FieldOptions::new("Secret")
        },
        "Session token",
        state.draft_secret(),
        Some(TextInputFocusTarget::SettingsSecret.id_value()),
        |value| Message::Settings(SettingsMessage::SecretChanged(value)),
    );

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
        endpoint,
        text("Authentication").size(12).style(theme::muted_text),
        auth_row,
        text("Theme").size(12).style(theme::muted_text),
        theme_row(state.draft_theme_preference()),
    ]
    .spacing(8);

    if matches!(state.draft_auth(), RpcAuthDraft::SessionSecret) {
        fields = fields.push(secret);
    }

    if let Some(feedback) = state.settings_feedback() {
        fields = fields.push(ui::form_feedback_banner(feedback));
    }

    fields = fields.push(polling);

    let mut actions = row![
        ui::text_button("Test Connection", ButtonVariant::Secondary)
            .on_press(Message::Connection(ConnectionMessage::TestRequested)),
        ui::text_button("Save", ButtonVariant::Primary)
            .on_press(Message::Settings(SettingsMessage::Save)),
        ui::text_button("Cancel", ButtonVariant::Secondary)
            .on_press(Message::Settings(SettingsMessage::Cancel)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    if state.is_plaintext_fallback_pending() {
        actions = actions
            .push(
                ui::text_button("Save Plaintext", ButtonVariant::Secondary)
                    .on_press(Message::Settings(SettingsMessage::SavePlaintextFallback)),
            )
            .push(
                ui::text_button("Session Only", ButtonVariant::Secondary)
                    .on_press(Message::Settings(SettingsMessage::KeepSecretSessionOnly)),
            );
    }

    ui::modal_surface(column![fields, actions].spacing(16))
        .padding(18)
        .width(Length::Fill)
        .into()
}

fn auth_button(
    label: &'static str,
    auth: RpcAuthDraft,
    selected: RpcAuthDraft,
) -> Element<'static, Message> {
    ui::toggle_text_button(label, auth == selected)
        .on_press(Message::Settings(SettingsMessage::AuthChanged(auth)))
        .into()
}

fn theme_row(selected: ThemePreference) -> Element<'static, Message> {
    let mut row = row![].spacing(8);

    for preference in ThemePreference::ALL {
        row = row.push(settings_theme_button(preference, selected));
    }

    row.into()
}

fn settings_theme_button(
    preference: ThemePreference,
    selected: ThemePreference,
) -> button::Button<'static, Message> {
    let label = preference.label();
    ui::toggle_text_button(label, preference == selected).on_press(Message::Settings(
        SettingsMessage::ThemePreferenceChanged(preference),
    ))
}

fn field_error(message: &'static str) -> FieldStatus<'static> {
    FieldStatus {
        kind: FieldStatusKind::Error,
        message,
        variant: FieldStatusVariant::Attached,
    }
}

fn connection_label(status: ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::Offline => "Offline",
        ConnectionStatus::Testing => "Testing...",
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Failed => "Connection failed",
    }
}
