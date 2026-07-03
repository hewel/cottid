use std::time::Duration;

use iced::widget::{column, container, mouse_area, row, text, text_input, tooltip};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use crate::ui::icons::{Icon, icon};
use crate::ui::overlay::style as overlay_style;
use crate::ui::tokens::{TOKENS, mode_from_theme};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Requiredness {
    None,
    Required,
    Optional,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldStatusKind {
    Warning,
    Error,
    Success,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldStatusVariant {
    Attached,
    Detached,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FieldStatus<'a> {
    pub(crate) kind: FieldStatusKind,
    pub(crate) message: &'a str,
    pub(crate) variant: FieldStatusVariant,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct FieldOptions<'a, Message = ()> {
    pub(crate) label: &'a str,
    pub(crate) description: Option<&'a str>,
    pub(crate) requiredness: Requiredness,
    pub(crate) is_disabled: bool,
    pub(crate) is_label_hidden: bool,
    pub(crate) label_tooltip: Option<&'a str>,
    pub(crate) label_action: Option<Message>,
    pub(crate) status: Option<FieldStatus<'a>>,
    pub(crate) width: Length,
}

impl<'a, Message> FieldOptions<'a, Message> {
    pub(crate) fn new(label: &'a str) -> Self {
        Self {
            label,
            description: None,
            requiredness: Requiredness::None,
            is_disabled: false,
            is_label_hidden: false,
            label_tooltip: None,
            label_action: None,
            status: None,
            width: Length::Fill,
        }
    }
}

pub(crate) fn field<'a, Message: Clone + 'a>(
    options: FieldOptions<'a, Message>,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut content = column![].spacing(TOKENS.spacing.s1).width(options.width);

    if !options.is_label_hidden {
        content = content.push(field_label(
            options.label,
            options.requiredness,
            options.is_disabled,
            options.label_tooltip,
            options.label_action,
        ));
    }

    let control = control.into();

    if let Some(status) = options.status {
        let status_box = field_status_box(status.kind, status.message, options.is_disabled);

        if matches!(status.variant, FieldStatusVariant::Attached) {
            content = content.push(column![control, status_box].spacing(0).width(Length::Fill));
        } else {
            content = content.push(control).push(status_box);
        }
    } else {
        content = content.push(control);
    }

    content.into()
}

pub(crate) fn text_field<'a, Message: Clone + 'a>(
    options: FieldOptions<'a, Message>,
    placeholder: &'a str,
    value: &str,
    input_id: Option<&'static str>,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let validation = options.status.map(|status| status.kind);
    let is_disabled = options.is_disabled;
    let mut input = text_input(placeholder, value)
        .on_input_maybe(if is_disabled { None } else { Some(on_input) })
        .padding(10)
        .style(move |theme, status| text_input_style(theme, status, validation));

    if let Some(input_id) = input_id {
        input = input.id(input_id);
    }

    field(options, input)
}

pub(crate) fn text_input_style(
    theme: &Theme,
    status: text_input::Status,
    validation: Option<FieldStatusKind>,
) -> text_input::Style {
    let mode = mode_from_theme(theme);
    let border_color = validation
        .map(|kind| status_foreground_color(mode, kind))
        .unwrap_or_else(|| match status {
            text_input::Status::Focused { .. } => TOKENS.colors.accent.get(mode),
            text_input::Status::Hovered => TOKENS.colors.border_emphasized.get(mode),
            text_input::Status::Disabled | text_input::Status::Active => {
                TOKENS.colors.border.get(mode)
            }
        });
    let background = match status {
        text_input::Status::Disabled => TOKENS.colors.background_muted.get(mode),
        text_input::Status::Hovered
        | text_input::Status::Focused { .. }
        | text_input::Status::Active => TOKENS.colors.background_surface.get(mode),
    };

    text_input::Style {
        background: Background::Color(background),
        border: Border {
            radius: TOKENS.radius.element.into(),
            color: if matches!(status, text_input::Status::Disabled) {
                TOKENS.colors.border.get(mode)
            } else {
                border_color
            },
            width: TOKENS.border_width.regular,
        },
        icon: TOKENS.colors.text_secondary.get(mode),
        placeholder: TOKENS.colors.text_secondary.get(mode),
        value: if matches!(status, text_input::Status::Disabled) {
            TOKENS.colors.text_disabled.get(mode)
        } else {
            TOKENS.colors.text_primary.get(mode)
        },
        selection: TOKENS.colors.accent_muted.get(mode),
    }
}

fn field_label<'a, Message: Clone + 'a>(
    label: &'a str,
    requiredness: Requiredness,
    is_disabled: bool,
    label_tooltip: Option<&'a str>,
    label_action: Option<Message>,
) -> Element<'a, Message> {
    let mut label_row = row![
        text(label)
            .size(TOKENS.typography.label)
            .style(move |theme| field_text_style(theme, is_disabled))
    ]
    .spacing(TOKENS.spacing.s1)
    .align_y(Alignment::Center);

    if let Some(marker) = requiredness_marker(requiredness) {
        label_row = label_row.push(
            text(marker)
                .size(TOKENS.typography.caption)
                .style(requiredness_marker_style),
        );
    }

    if let Some(tooltip_text) = label_tooltip {
        label_row = label_row.push(label_tooltip_icon(tooltip_text, is_disabled));
    }

    if let Some(action) = label_action.filter(|_| !is_disabled) {
        return mouse_area(label_row).on_press(action).into();
    }

    label_row.into()
}

fn label_tooltip_icon<'a, Message: 'a>(
    tooltip_text: &'a str,
    is_disabled: bool,
) -> Element<'a, Message> {
    let trigger = container(icon(Icon::Info, 16.0, move |theme| {
        field_text_color(theme, is_disabled)
    }))
    .padding([0, 4]);

    tooltip(
        trigger,
        container(text(tooltip_text).size(TOKENS.typography.caption)).max_width(260),
        tooltip::Position::Right,
    )
    .delay(Duration::from_millis(200))
    .gap(6.0)
    .padding(8.0)
    .snap_within_viewport(true)
    .style(overlay_style::tooltip_surface)
    .into()
}

fn field_status_box<'a, Message: 'a>(
    kind: FieldStatusKind,
    message: &'a str,
    is_disabled: bool,
) -> Element<'a, Message> {
    // Astryx Field uses CSS selectors, focus-within, margins, and layered
    // shadows. iced exposes explicit widget composition and one widget style at
    // a time, so attached status is an approximation rather than border merging.
    container(
        text(message)
            .size(TOKENS.typography.caption)
            .style(move |theme| status_text_style(theme, kind, is_disabled)),
    )
    .padding([TOKENS.spacing.s1, 0.0])
    .width(Length::Fill)
    .into()
}

fn field_text_style(theme: &Theme, is_disabled: bool) -> text::Style {
    text::Style {
        color: Some(field_text_color(theme, is_disabled)),
    }
}

fn field_text_color(theme: &Theme, is_disabled: bool) -> Color {
    let mode = mode_from_theme(theme);
    if is_disabled {
        TOKENS.colors.text_disabled.get(mode)
    } else {
        TOKENS.colors.text_secondary.get(mode)
    }
}

fn requiredness_marker_style(theme: &Theme) -> text::Style {
    let mode = mode_from_theme(theme);
    text::Style {
        color: Some(TOKENS.colors.error_muted.get(mode)),
    }
}

fn status_text_style(theme: &Theme, kind: FieldStatusKind, is_disabled: bool) -> text::Style {
    let mode = mode_from_theme(theme);
    text::Style {
        color: Some(if is_disabled {
            TOKENS.colors.text_disabled.get(mode)
        } else {
            status_foreground_color(mode, kind)
        }),
    }
}

fn status_foreground_color(mode: crate::ui::tokens::Mode, kind: FieldStatusKind) -> Color {
    match kind {
        FieldStatusKind::Warning => TOKENS.colors.warning.get(mode),
        FieldStatusKind::Error => TOKENS.colors.error.get(mode),
        FieldStatusKind::Success => TOKENS.colors.success.get(mode),
    }
}

fn requiredness_marker(requiredness: Requiredness) -> Option<&'static str> {
    match requiredness {
        Requiredness::Required => Some("*"),
        Requiredness::Optional | Requiredness::None => None,
    }
}

#[cfg(test)]
mod tests {
    use iced::Color;
    use iced::widget::text_input;

    use super::{
        FieldOptions, FieldStatusKind, Requiredness, requiredness_marker,
        requiredness_marker_style, text_input_style,
    };

    #[test]
    fn field_options_new_defaults_to_visible_enabled_fill_width_field() {
        let options: FieldOptions<'_, ()> = FieldOptions::new("RPC URL");

        assert_eq!(options.label, "RPC URL");
        assert_eq!(options.description, None);
        assert_eq!(options.requiredness, Requiredness::None);
        assert!(!options.is_disabled);
        assert!(!options.is_label_hidden);
        assert_eq!(options.label_tooltip, None);
        assert_eq!(options.label_action, None);
        assert_eq!(options.status, None);
        assert_eq!(options.width, iced::Length::Fill);
    }

    #[test]
    fn requiredness_marker_returns_asterisk_for_required_fields() {
        assert_eq!(requiredness_marker(Requiredness::Required), Some("*"));
    }

    #[test]
    fn requiredness_marker_returns_no_marker_for_optional_fields() {
        assert_eq!(requiredness_marker(Requiredness::Optional), None);
    }

    #[test]
    fn requiredness_marker_returns_no_marker_for_unmarked_fields() {
        assert_eq!(requiredness_marker(Requiredness::None), None);
    }

    #[test]
    fn requiredness_marker_style_uses_muted_error_color() {
        let style = requiredness_marker_style(&iced::Theme::Light);

        assert_eq!(style.color, Some(Color::from_rgb8(250, 206, 203)));
    }

    #[test]
    fn text_input_style_uses_error_border_for_validation_error() {
        let style = text_input_style(
            &iced::Theme::Light,
            text_input::Status::Focused { is_hovered: false },
            Some(FieldStatusKind::Error),
        );

        assert_eq!(style.border.color, Color::from_rgb8(165, 12, 37));
    }

    #[test]
    fn text_input_style_uses_warning_border_for_validation_warning() {
        let style = text_input_style(
            &iced::Theme::Light,
            text_input::Status::Hovered,
            Some(FieldStatusKind::Warning),
        );

        assert_eq!(style.border.color, Color::from_rgb8(116, 91, 0));
    }

    #[test]
    fn text_input_style_uses_success_border_for_validation_success() {
        let style = text_input_style(
            &iced::Theme::Light,
            text_input::Status::Active,
            Some(FieldStatusKind::Success),
        );

        assert_eq!(style.border.color, Color::from_rgb8(0, 112, 4));
    }

    #[test]
    fn text_input_style_keeps_disabled_border_muted_when_validation_exists() {
        let style = text_input_style(
            &iced::Theme::Light,
            text_input::Status::Disabled,
            Some(FieldStatusKind::Error),
        );

        assert_eq!(style.border.color, Color::from_rgb8(235, 235, 235));
        assert_eq!(style.value, Color::from_rgb8(163, 163, 163));
    }
}
