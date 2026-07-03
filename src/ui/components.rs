use iced::widget::{
    button, column, container, mouse_area, opaque, row, scrollable, space, stack, text,
};
use iced::{Alignment, Color, Element, Length, Theme};

use crate::app::{FeedbackTone, FormFeedback, Message};
use crate::ui::icons::{Icon, icon};
use crate::ui::overlay::style as overlay_style;
use crate::ui::theme;
use crate::ui::tokens::TOKENS;
use crate::ui::variants::{BadgeVariant, ButtonVariant, FeedbackVariant, SurfaceVariant};
use crate::ui::widgets;

pub(crate) fn surface<'a>(
    content: impl Into<Element<'a, Message>>,
    variant: SurfaceVariant,
) -> container::Container<'a, Message> {
    container(content).style(move |theme| theme::surface_variant(theme, variant))
}

pub(crate) fn app_surface<'a>(
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    surface(content, SurfaceVariant::App)
}

pub(crate) fn sidebar_surface<'a>(
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    surface(content, SurfaceVariant::Sidebar)
}

pub(crate) fn card_surface<'a>(
    content: impl Into<Element<'a, Message>>,
    selected: bool,
) -> container::Container<'a, Message> {
    let variant = if selected {
        SurfaceVariant::SelectedCard
    } else {
        SurfaceVariant::Card
    };

    surface(content, variant)
}

pub(crate) fn muted_panel<'a>(
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    surface(content, SurfaceVariant::Muted)
}

pub(crate) fn modal_surface<'a>(
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    surface(content, SurfaceVariant::Modal)
}

pub(crate) fn modal_layer<'a>(
    content: impl Into<Element<'a, Message>>,
    max_width: f32,
    max_height: f32,
) -> Element<'a, Message> {
    let scrim: Element<'a, Message> = mouse_area(
        surface(space::vertical().width(Length::Fill), SurfaceVariant::Scrim)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .on_press(Message::ModalCancel)
    .into();

    let modal_content = scrollable(content).width(Length::Fill);
    let modal_card = opaque(
        container(modal_content)
            .width(Length::Fill)
            .max_width(max_width)
            .max_height(max_height),
    );
    let modal = container(modal_card).padding(24).center(Length::Fill);

    stack(vec![scrim, modal.into()])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub(crate) fn search_surface<'a>(
    content: impl Into<Element<'a, Message>>,
) -> container::Container<'a, Message> {
    surface(content, SurfaceVariant::Search)
}

pub(crate) fn icon_button(icon_kind: Icon, message: Message) -> button::Button<'static, Message> {
    button(icon(icon_kind, 18.0, theme::text_color))
        .padding(10)
        .style(theme::icon_button)
        .on_press(message)
}

pub(crate) fn action_button(
    icon_kind: Icon,
    message: Message,
    danger: bool,
) -> Element<'static, Message> {
    let color = if danger {
        theme::danger_color
    } else {
        theme::text_color
    };
    let style = if danger {
        theme::danger_button
    } else {
        theme::icon_button
    };

    button(icon(icon_kind, 16.0, color))
        .style(style)
        .padding(10)
        .on_press(message)
        .into()
}

pub(crate) fn text_button(
    label: impl Into<String>,
    variant: ButtonVariant,
) -> button::Button<'static, Message> {
    button(text(label.into()))
        .style(move |theme, status| theme::button_variant(theme, status, variant))
}

pub(crate) fn toggle_button<'a>(
    content: impl Into<Element<'a, Message>>,
    selected: bool,
) -> button::Button<'a, Message> {
    let variant = if selected {
        ButtonVariant::Secondary
    } else {
        ButtonVariant::Ghost
    };

    button(content).style(move |theme, status| theme::button_variant(theme, status, variant))
}

pub(crate) fn toggle_text_button(
    label: impl Into<String>,
    selected: bool,
) -> button::Button<'static, Message> {
    toggle_button(text(label.into()), selected)
}

pub(crate) fn badge(label: impl Into<String>, variant: BadgeVariant) -> Element<'static, Message> {
    container(text(label.into()).size(crate::ui::tokens::TOKENS.typography.caption))
        .padding([crate::ui::tokens::S1 / 2.0, crate::ui::tokens::S2])
        .style(move |theme| widgets::badge::style(theme, variant))
        .into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransferSpeedTone {
    Default,
    Tooltip,
}

pub(crate) fn transfer_speed_summary(
    download_speed: impl Into<String>,
    upload_speed: impl Into<String>,
    eta: Option<String>,
    tone: TransferSpeedTone,
) -> Element<'static, Message> {
    transfer_speed_summary_content(download_speed, upload_speed, eta, tone).into()
}

pub(crate) fn transfer_speed_summary_end_aligned(
    download_speed: impl Into<String>,
    upload_speed: impl Into<String>,
    eta: Option<String>,
    tone: TransferSpeedTone,
) -> Element<'static, Message> {
    row![
        space::horizontal(),
        transfer_speed_summary_content(download_speed, upload_speed, eta, tone),
    ]
    .width(Length::Fill)
    .align_y(Alignment::Center)
    .into()
}

fn transfer_speed_summary_content(
    download_speed: impl Into<String>,
    upload_speed: impl Into<String>,
    eta: Option<String>,
    tone: TransferSpeedTone,
) -> iced::widget::Row<'static, Message> {
    let mut content = row![
        transfer_speed_item(Icon::ArrowDown, download_speed.into(), tone),
        transfer_speed_item(Icon::ArrowUp, upload_speed.into(), tone),
    ]
    .spacing(6)
    .align_y(Alignment::Center);

    if let Some(eta) = eta {
        content = content.push(transfer_speed_item(Icon::HourglassMedium, eta, tone));
    }

    content
}

fn transfer_speed_item(
    icon_kind: Icon,
    label: String,
    tone: TransferSpeedTone,
) -> Element<'static, Message> {
    let color = transfer_speed_color(tone);
    let label = text(label).size(TOKENS.typography.caption);
    let label = if matches!(tone, TransferSpeedTone::Default) {
        label.style(theme::muted_text)
    } else {
        label
    };

    row![icon(icon_kind, 12.0, color), label,]
        .spacing(4)
        .align_y(Alignment::Center)
        .into()
}

fn transfer_speed_color(tone: TransferSpeedTone) -> fn(&Theme) -> Color {
    match tone {
        TransferSpeedTone::Default => theme::muted_color,
        TransferSpeedTone::Tooltip => overlay_style::tooltip_foreground,
    }
}

pub(crate) fn form_feedback_banner(feedback: &FormFeedback) -> Element<'static, Message> {
    feedback_banner(feedback.tone(), feedback.message())
}

pub(crate) fn feedback_banner(tone: FeedbackTone, message: &str) -> Element<'static, Message> {
    let (icon_kind, variant) = feedback_variant(tone);
    let color = match variant {
        FeedbackVariant::Info => theme::feedback_info_color,
        FeedbackVariant::Success => theme::feedback_success_color,
        FeedbackVariant::Warning => theme::feedback_warning_color,
        FeedbackVariant::Error => theme::feedback_error_color,
    };

    surface(
        row![
            icon(icon_kind, 16.0, color),
            text(message.to_owned()).size(12),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        SurfaceVariant::Feedback(variant),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .into()
}

pub(crate) fn stat_row(label: &'static str, value: &str) -> Element<'static, Message> {
    muted_panel(
        row![
            text(label)
                .size(12)
                .style(theme::muted_text)
                .width(Length::Fill),
            text(value.to_owned()).size(13),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    )
    .padding(10)
    .width(Length::Fill)
    .into()
}

pub(crate) fn section(title: &'static str, rows: &[String]) -> Element<'static, Message> {
    let mut rows_content = column![].spacing(6);

    for row in rows {
        rows_content = rows_content.push(text(row.to_owned()).size(12).style(theme::muted_text));
    }

    section_element(title, rows_content.into())
}

pub(crate) fn section_element(
    title: &'static str,
    body: Element<'static, Message>,
) -> Element<'static, Message> {
    let content = column![text(title).size(13), body].spacing(6);

    muted_panel(content).padding(10).width(Length::Fill).into()
}

fn feedback_variant(tone: FeedbackTone) -> (Icon, FeedbackVariant) {
    match tone {
        FeedbackTone::Info => (Icon::Info, FeedbackVariant::Info),
        FeedbackTone::Success => (Icon::CheckCircle, FeedbackVariant::Success),
        FeedbackTone::Warning => (Icon::Error, FeedbackVariant::Warning),
        FeedbackTone::Error => (Icon::XCircle, FeedbackVariant::Error),
    }
}
