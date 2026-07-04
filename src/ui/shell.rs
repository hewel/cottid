use iced::widget::{button, checkbox, column, container, row, space, stack, text};
use iced::{Alignment, Element, Length};

use crate::app::{
    AddMessage, ConnectionMessage, ConnectionStatus, DownloadsMessage, Message,
    PendingActionConfirmation, SettingsMessage, State, TextInputFocusTarget, ToolbarMessage,
};
use crate::config::{DaemonMode, ThemePreference};
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

    if state.pending_action_confirmation().is_some() {
        layers.push(ui::modal_layer(
            action_confirmation_modal(state),
            state.modal_max_width(420.0),
            state.modal_max_height(),
        ));
    } else if state.is_settings_open() {
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

fn action_confirmation_modal(state: &State) -> Element<'_, Message> {
    let Some(confirmation) = state.pending_action_confirmation() else {
        return space::vertical().into();
    };
    let (title, body, confirm_label) = match confirmation {
        PendingActionConfirmation::Remove(_) => (
            "Remove Download",
            "Remove this download from aria2?",
            "Remove",
        ),
        PendingActionConfirmation::PurgeStopped => (
            "Purge Stopped Results",
            "Purge completed and failed results from aria2?",
            "Purge",
        ),
    };

    let content = column![
        text(title).size(20),
        text(body).size(13).style(theme::muted_text),
        row![
            ui::text_button(confirm_label, ButtonVariant::Destructive)
                .on_press(Message::Action(crate::app::ActionMessage::ConfirmPending)),
            ui::text_button("Cancel", ButtonVariant::Secondary)
                .on_press(Message::Action(crate::app::ActionMessage::CancelPending)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(12);

    content.into()
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
    let mut content = column![
        text("Connection").size(12).style(theme::muted_text),
        text(state.daemon_status_text()).size(13),
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
    .spacing(4);

    if let Some(error) = state.daemon_error_text() {
        content = content.push(text(error).size(12).style(theme::danger_text));
    }

    if let Some(warning) = state.daemon_shutdown_warning_text() {
        content = content.push(text(warning.to_owned()).size(12).style(theme::muted_text));
    }

    if let Some(log_path) = state.managed_daemon_log_path_text() {
        content = content.push(text(log_path).size(12).style(theme::muted_text));
    }

    content.width(Length::Fill).into()
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

    let output_filename = text_field(
        FieldOptions {
            description: Some("Optional output filename for this download."),
            requiredness: Requiredness::Optional,
            is_disabled: state.is_add_pending(),
            status: state
                .add_output_filename_validation_message()
                .map(field_error),
            ..FieldOptions::new("Output filename")
        },
        "file.iso",
        state.add_output_filename(),
        None,
        |value| Message::Add(AddMessage::OutputFilenameChanged(value)),
    );

    let max_download_limit = text_field(
        FieldOptions {
            description: Some(
                "Optional per-task download limit in bytes per second. 0 is unlimited.",
            ),
            requiredness: Requiredness::Optional,
            is_disabled: state.is_add_pending(),
            status: state
                .add_max_download_limit_validation_message()
                .map(field_error),
            ..FieldOptions::new("Download limit")
        },
        "0",
        state.add_max_download_limit(),
        None,
        |value| Message::Add(AddMessage::MaxDownloadLimitChanged(value)),
    );

    let max_upload_limit = text_field(
        FieldOptions {
            description: Some(
                "Optional per-task upload limit in bytes per second. 0 is unlimited.",
            ),
            requiredness: Requiredness::Optional,
            is_disabled: state.is_add_pending(),
            status: state
                .add_max_upload_limit_validation_message()
                .map(field_error),
            ..FieldOptions::new("Upload limit")
        },
        "0",
        state.add_max_upload_limit(),
        None,
        |value| Message::Add(AddMessage::MaxUploadLimitChanged(value)),
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

    let mut content = column![
        text("Add Download").size(20),
        input,
        output_filename,
        max_download_limit,
        max_upload_limit,
    ]
    .spacing(10);

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

    content.into()
}

fn settings_modal(state: &State) -> Element<'_, Message> {
    let is_testing = matches!(state.connection_status(), ConnectionStatus::Testing);
    let mode = daemon_mode_selector(state);
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

    let new_download_directory = text_field(
        FieldOptions {
            description: Some("Daemon-local directory used for new downloads."),
            requiredness: Requiredness::Optional,
            is_disabled: is_testing,
            label_action: Some(Message::FocusTextInput(
                TextInputFocusTarget::SettingsNewDownloadDirectory,
            )),
            ..FieldOptions::new("New download directory")
        },
        "/downloads",
        state.draft_new_download_directory(),
        Some(TextInputFocusTarget::SettingsNewDownloadDirectory.id_value()),
        |value| Message::Settings(SettingsMessage::NewDownloadDirectoryChanged(value)),
    );

    let new_download_output = text_field(
        FieldOptions {
            description: Some("Default output filename for new downloads."),
            requiredness: Requiredness::Optional,
            is_disabled: is_testing,
            label_action: Some(Message::FocusTextInput(
                TextInputFocusTarget::SettingsNewDownloadOutput,
            )),
            status: state
                .draft_new_download_output_filename_validation_message()
                .map(field_error),
            ..FieldOptions::new("Default output filename")
        },
        "file.iso",
        state.draft_new_download_output_filename(),
        Some(TextInputFocusTarget::SettingsNewDownloadOutput.id_value()),
        |value| Message::Settings(SettingsMessage::NewDownloadOutputFilenameChanged(value)),
    );

    let new_download_download_limit = text_field(
        FieldOptions {
            description: Some(
                "Default per-task download limit in bytes per second. 0 is unlimited.",
            ),
            requiredness: Requiredness::Optional,
            is_disabled: is_testing,
            label_action: Some(Message::FocusTextInput(
                TextInputFocusTarget::SettingsNewDownloadDownloadLimit,
            )),
            status: state
                .draft_new_download_max_download_limit_validation_message()
                .map(field_error),
            ..FieldOptions::new("Default download limit")
        },
        "0",
        state.draft_new_download_max_download_limit(),
        Some(TextInputFocusTarget::SettingsNewDownloadDownloadLimit.id_value()),
        |value| Message::Settings(SettingsMessage::NewDownloadMaxDownloadLimitChanged(value)),
    );

    let new_download_upload_limit = text_field(
        FieldOptions {
            description: Some("Default per-task upload limit in bytes per second. 0 is unlimited."),
            requiredness: Requiredness::Optional,
            is_disabled: is_testing,
            label_action: Some(Message::FocusTextInput(
                TextInputFocusTarget::SettingsNewDownloadUploadLimit,
            )),
            status: state
                .draft_new_download_max_upload_limit_validation_message()
                .map(field_error),
            ..FieldOptions::new("Default upload limit")
        },
        "0",
        state.draft_new_download_max_upload_limit(),
        Some(TextInputFocusTarget::SettingsNewDownloadUploadLimit.id_value()),
        |value| Message::Settings(SettingsMessage::NewDownloadMaxUploadLimitChanged(value)),
    );

    let runtime_max_concurrent = text_field(
        FieldOptions {
            description: Some("Maximum active downloads on the connected daemon."),
            requiredness: Requiredness::Optional,
            is_disabled: is_testing,
            label_action: Some(Message::FocusTextInput(
                TextInputFocusTarget::SettingsRuntimeMaxConcurrent,
            )),
            status: state
                .draft_runtime_max_concurrent_downloads_validation_message()
                .map(field_error),
            ..FieldOptions::new("Max concurrent downloads")
        },
        "5",
        state.draft_runtime_max_concurrent_downloads(),
        Some(TextInputFocusTarget::SettingsRuntimeMaxConcurrent.id_value()),
        |value| Message::Settings(SettingsMessage::RuntimeMaxConcurrentDownloadsChanged(value)),
    );

    let runtime_download_limit = text_field(
        FieldOptions {
            description: Some("Global download limit in bytes per second. 0 is unlimited."),
            requiredness: Requiredness::Optional,
            is_disabled: is_testing,
            label_action: Some(Message::FocusTextInput(
                TextInputFocusTarget::SettingsRuntimeDownloadLimit,
            )),
            status: state
                .draft_runtime_max_overall_download_limit_validation_message()
                .map(field_error),
            ..FieldOptions::new("Global download limit")
        },
        "0",
        state.draft_runtime_max_overall_download_limit(),
        Some(TextInputFocusTarget::SettingsRuntimeDownloadLimit.id_value()),
        |value| {
            Message::Settings(SettingsMessage::RuntimeMaxOverallDownloadLimitChanged(
                value,
            ))
        },
    );

    let runtime_upload_limit = text_field(
        FieldOptions {
            description: Some("Global upload limit in bytes per second. 0 is unlimited."),
            requiredness: Requiredness::Optional,
            is_disabled: is_testing,
            label_action: Some(Message::FocusTextInput(
                TextInputFocusTarget::SettingsRuntimeUploadLimit,
            )),
            status: state
                .draft_runtime_max_overall_upload_limit_validation_message()
                .map(field_error),
            ..FieldOptions::new("Global upload limit")
        },
        "0",
        state.draft_runtime_max_overall_upload_limit(),
        Some(TextInputFocusTarget::SettingsRuntimeUploadLimit.id_value()),
        |value| Message::Settings(SettingsMessage::RuntimeMaxOverallUploadLimitChanged(value)),
    );

    let secret = text_field(
        FieldOptions {
            description: Some("Optional aria2 token secret for this endpoint."),
            requiredness: Requiredness::Optional,
            is_disabled: is_testing,
            label_action: Some(Message::FocusTextInput(
                TextInputFocusTarget::SettingsSecret,
            )),
            ..FieldOptions::new("Secret")
        },
        "Session token",
        state.draft_secret(),
        Some(TextInputFocusTarget::SettingsSecret.id_value()),
        |value| Message::Settings(SettingsMessage::SecretChanged(value)),
    );

    let mut fields = column![text("Connection Settings").size(20), mode,].spacing(8);
    if state.is_draft_external_daemon_mode() {
        fields = fields.push(endpoint).push(secret);
    }

    if let Some(feedback) = state.settings_feedback() {
        fields = fields.push(ui::form_feedback_banner(feedback));
    }

    fields = fields
        .push(polling)
        .push(
            checkbox(state.draft_websocket_enabled())
                .label("Use WebSocket for live updates and download actions")
                .on_toggle(|enabled| {
                    Message::Settings(SettingsMessage::WebSocketEnabledChanged(enabled))
                })
                .size(16),
        )
        .push(new_download_directory)
        .push(new_download_output)
        .push(new_download_download_limit)
        .push(new_download_upload_limit)
        .push(runtime_max_concurrent)
        .push(runtime_download_limit)
        .push(runtime_upload_limit)
        .push(
            checkbox(state.confirm_destructive_actions())
                .label("Confirm remove and purge actions")
                .on_toggle(|enabled| {
                    Message::Settings(SettingsMessage::ConfirmDestructiveActionsChanged(enabled))
                })
                .size(16),
        )
        .push(
            checkbox(state.notify_download_outcomes())
                .label("Track completed and failed download notification intents")
                .on_toggle(|enabled| {
                    Message::Settings(SettingsMessage::NotifyDownloadOutcomesChanged(enabled))
                })
                .size(16),
        );

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

    column![fields, actions].spacing(16).into()
}

fn daemon_mode_selector(state: &State) -> Element<'static, Message> {
    row![
        ui::toggle_button(
            text("Managed local").size(13),
            state.draft_daemon_mode() == DaemonMode::Managed
        )
        .padding([8, 10])
        .on_press(Message::Settings(SettingsMessage::DaemonModeChanged(
            DaemonMode::Managed,
        ))),
        ui::toggle_button(
            text("External").size(13),
            state.draft_daemon_mode() == DaemonMode::External
        )
        .padding([8, 10])
        .on_press(Message::Settings(SettingsMessage::DaemonModeChanged(
            DaemonMode::External,
        ))),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
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
