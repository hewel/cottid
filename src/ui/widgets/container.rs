use iced::widget::container;
use iced::{Background, Border, Color, Shadow, Theme};

use crate::ui::tokens::{TOKENS, mode_from_theme};

pub(crate) fn surface(theme: &Theme) -> container::Style {
    let mode = mode_from_theme(theme);
    container_style(
        TOKENS.colors.background_surface.get(mode),
        TOKENS.colors.text_primary.get(mode),
        bordered(TOKENS.radius.container, TOKENS.colors.border.get(mode)),
        TOKENS.shadow.none,
    )
}

pub(crate) fn card(theme: &Theme) -> container::Style {
    let mode = mode_from_theme(theme);
    container_style(
        TOKENS.colors.background_card.get(mode),
        TOKENS.colors.text_primary.get(mode),
        bordered(TOKENS.radius.container, TOKENS.colors.border.get(mode)),
        TOKENS.shadow.none,
    )
}

pub(crate) fn selected_card(theme: &Theme) -> container::Style {
    let mode = mode_from_theme(theme);
    container_style(
        TOKENS.colors.background_card.get(mode),
        TOKENS.colors.text_primary.get(mode),
        bordered(TOKENS.radius.container, TOKENS.colors.accent.get(mode)),
        TOKENS.shadow.none,
    )
}

pub(crate) fn modal(theme: &Theme) -> container::Style {
    let mode = mode_from_theme(theme);
    container_style(
        TOKENS.colors.background_surface.get(mode),
        TOKENS.colors.text_primary.get(mode),
        Border {
            radius: TOKENS.radius.container.into(),
            color: Color::TRANSPARENT,
            width: TOKENS.border_width.hairline,
        },
        TOKENS.shadow.high.get(mode),
    )
}

pub(crate) fn scrim(theme: &Theme) -> container::Style {
    let mode = mode_from_theme(theme);
    container_style(
        TOKENS.colors.background_scrim.get(mode),
        TOKENS.colors.text_primary.get(mode),
        Border::default(),
        TOKENS.shadow.none,
    )
}

pub(crate) fn muted(theme: &Theme) -> container::Style {
    let mode = mode_from_theme(theme);
    container_style(
        TOKENS.colors.background_muted.get(mode),
        TOKENS.colors.text_secondary.get(mode),
        bordered(TOKENS.radius.container, TOKENS.colors.border.get(mode)),
        TOKENS.shadow.none,
    )
}

pub(crate) fn search(theme: &Theme) -> container::Style {
    let mode = mode_from_theme(theme);
    container_style(
        TOKENS.colors.background_muted.get(mode),
        TOKENS.colors.text_secondary.get(mode),
        bordered(TOKENS.radius.element, TOKENS.colors.border.get(mode)),
        TOKENS.shadow.none,
    )
}

pub(crate) fn app(theme: &Theme) -> container::Style {
    let mode = mode_from_theme(theme);
    container_style(
        TOKENS.colors.background_body.get(mode),
        TOKENS.colors.text_primary.get(mode),
        Border::default(),
        TOKENS.shadow.none,
    )
}

pub(crate) fn feedback(theme: &Theme, background: Color, border_color: Color) -> container::Style {
    let mode = mode_from_theme(theme);
    container_style(
        background,
        TOKENS.colors.text_primary.get(mode),
        bordered(TOKENS.radius.element, border_color),
        TOKENS.shadow.none,
    )
}

fn container_style(
    background: Color,
    text_color: Color,
    border: Border,
    shadow: Shadow,
) -> container::Style {
    container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(text_color),
        border,
        shadow,
        ..container::Style::default()
    }
}

fn bordered(radius: f32, color: Color) -> Border {
    Border {
        radius: radius.into(),
        color,
        width: TOKENS.border_width.regular,
    }
}
