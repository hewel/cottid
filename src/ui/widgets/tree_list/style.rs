use iced::widget::{button, container, text};
use iced::{Background, Border, Color, Theme};

use crate::ui::color;
use crate::ui::tokens::{TOKENS, mode_from_theme};

use super::types::DensityMetrics;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TreeListStyle {
    pub(crate) text_primary: Color,
    pub(crate) text_secondary: Color,
    pub(crate) text_disabled: Color,
    pub(crate) row_selected_background: Color,
    pub(crate) row_hover_background: Color,
    pub(crate) row_pressed_background: Color,
    pub(crate) chevron_color: Color,
    pub(crate) row_radius: f32,
}

pub(crate) fn tree_list_style(theme: &Theme, _metrics: DensityMetrics) -> TreeListStyle {
    let mode = mode_from_theme(theme);

    TreeListStyle {
        text_primary: TOKENS.colors.text_primary.get(mode),
        text_secondary: TOKENS.colors.text_secondary.get(mode),
        text_disabled: TOKENS.colors.text_disabled.get(mode),
        row_selected_background: TOKENS.colors.accent_muted.get(mode),
        row_hover_background: TOKENS.colors.background_hover.get(mode),
        row_pressed_background: TOKENS.colors.background_pressed.get(mode),
        chevron_color: TOKENS.colors.text_secondary.get(mode),
        row_radius: TOKENS.radius.inner,
    }
}

pub(crate) fn tree_row_style(
    theme: &Theme,
    status: button::Status,
    selected: bool,
    disabled: bool,
    metrics: DensityMetrics,
) -> button::Style {
    let style = tree_list_style(theme, metrics);
    let base = if selected {
        style.row_selected_background
    } else {
        Color::TRANSPARENT
    };
    let background = match status {
        button::Status::Hovered if !disabled && selected => {
            let mode = mode_from_theme(theme);
            color::overlay(base, TOKENS.interaction_overlay.get(mode).hover)
        }
        button::Status::Hovered if !disabled => style.row_hover_background,
        button::Status::Pressed if !disabled && selected => {
            let mode = mode_from_theme(theme);
            color::overlay(base, TOKENS.interaction_overlay.get(mode).pressed)
        }
        button::Status::Pressed if !disabled => style.row_pressed_background,
        button::Status::Active | button::Status::Disabled | button::Status::Hovered => base,
        button::Status::Pressed => base,
    };

    button::Style {
        background: background_color(background),
        text_color: text_color_from_style(style, disabled),
        border: Border {
            radius: style.row_radius.into(),
            color: Color::TRANSPARENT,
            width: 0.0,
        },
        ..button::Style::default()
    }
}

pub(crate) fn row_container_style(
    theme: &Theme,
    selected: bool,
    metrics: DensityMetrics,
) -> container::Style {
    let style = tree_list_style(theme, metrics);
    let background = if selected {
        style.row_selected_background
    } else {
        Color::TRANSPARENT
    };

    container::Style {
        background: background_color(background),
        text_color: Some(style.text_primary),
        border: Border {
            radius: style.row_radius.into(),
            color: Color::TRANSPARENT,
            width: 0.0,
        },
        ..container::Style::default()
    }
}

pub(crate) fn primary_text(theme: &Theme, disabled: bool) -> text::Style {
    let style = tree_list_style(theme, DensityMetrics::default());
    text::Style {
        color: Some(text_color_from_style(style, disabled)),
    }
}

pub(crate) fn secondary_text(theme: &Theme, disabled: bool) -> text::Style {
    let style = tree_list_style(theme, DensityMetrics::default());
    text::Style {
        color: Some(if disabled {
            style.text_disabled
        } else {
            style.text_secondary
        }),
    }
}

pub(crate) fn chevron_icon_color(theme: &Theme) -> Color {
    tree_list_style(theme, DensityMetrics::default()).chevron_color
}

pub(crate) fn disabled_chevron_icon_color(theme: &Theme) -> Color {
    tree_list_style(theme, DensityMetrics::default()).text_disabled
}

fn text_color_from_style(style: TreeListStyle, disabled: bool) -> Color {
    if disabled {
        style.text_disabled
    } else {
        style.text_primary
    }
}

fn background_color(background: Color) -> Option<Background> {
    if background.a <= f32::EPSILON {
        None
    } else {
        Some(Background::Color(background))
    }
}

#[cfg(test)]
mod tests {
    use iced::widget::button;
    use iced::{Background, Color};

    #[test]
    fn tree_row_style_uses_selected_background() {
        let style = super::tree_row_style(
            &iced::Theme::Light,
            button::Status::Active,
            true,
            false,
            crate::ui::widgets::tree_list::TreeDensity::Balanced.metrics(),
        );

        assert!(matches!(style.background, Some(Background::Color(_))));
    }

    #[test]
    fn primary_text_uses_disabled_color_when_disabled() {
        let style = super::primary_text(&iced::Theme::Light, true);

        assert_eq!(style.color, Some(Color::from_rgb8(163, 163, 163)));
    }
}
