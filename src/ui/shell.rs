use iced::widget::{button, column, container, row, space, stack, text};
use iced::{Alignment, Element, Length};

use crate::app::{
    ActionMessage, AddMessage, ConnectionMessage, ConnectionStatus, DownloadsMessage, Message,
    SettingsMessage, State, ToolbarMessage,
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

    let bottom_content = column![
        row![connection_detail_popover(state), settings_icon_button()]
            .spacing(8)
            .width(Length::Fill)
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
            ui::badge(count.to_string(), BadgeVariant::Neutral),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
    };
    let button = ui::toggle_button(content, selected)
        .padding([9, 12])
        .width(Length::Fill);

    button.on_press(message()).into()
}

fn add_modal(state: &State) -> Element<'_, Message> {
    let input = ui::form_input(
        "https://example.com/file.iso or magnet:?",
        state.add_input(),
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

    let add_feedback = ui::feedback_or_info(state.add_feedback(), "Enter one URI or magnet link.");

    ui::modal_surface(
        column![
            text("Add Download").size(20),
            input,
            add_feedback,
            row![
                submit,
                ui::text_button("Cancel", ButtonVariant::Secondary)
                    .on_press(Message::Add(AddMessage::Cancel)),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        ]
        .spacing(10),
    )
    .padding(18)
    .width(Length::Fill)
    .into()
}

fn settings_modal(state: &State) -> Element<'_, Message> {
    let endpoint = ui::form_input(
        "http://localhost:6800/jsonrpc",
        state.draft_endpoint(),
        |value| Message::Settings(SettingsMessage::EndpointChanged(value)),
    );

    let polling_value = state.draft_polling_interval_seconds().to_string();
    let polling = ui::form_input("2", &polling_value, |value| {
        Message::Settings(SettingsMessage::PollingIntervalChanged(value))
    });

    let secret = ui::form_input("Session token", state.draft_secret(), |value| {
        Message::Settings(SettingsMessage::SecretChanged(value))
    });

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

    let settings_feedback =
        ui::feedback_or_info(state.settings_feedback(), "Settings are not persisted yet.");

    fields = fields
        .push(text("Polling interval").size(12).style(theme::muted_text))
        .push(polling)
        .push(settings_feedback);

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
        row = row.push(theme_button(preference, selected));
    }

    row.into()
}

fn theme_button(
    preference: ThemePreference,
    selected: ThemePreference,
) -> button::Button<'static, Message> {
    let label = preference.label();
    ui::toggle_text_button(label, preference == selected).on_press(Message::Settings(
        SettingsMessage::ThemePreferenceChanged(preference),
    ))
}

fn connection_label(status: ConnectionStatus) -> &'static str {
    match status {
        ConnectionStatus::Offline => "Offline",
        ConnectionStatus::Testing => "Testing...",
        ConnectionStatus::Connected => "Connected",
        ConnectionStatus::Failed => "Connection failed",
    }
}
