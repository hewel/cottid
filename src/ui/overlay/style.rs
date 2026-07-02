use iced::widget::container;
use iced::{Background, Border, Color, Theme};

use crate::ui::tokens::{TOKENS, mode_from_theme};

pub(crate) fn tooltip_surface(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(23, 23, 23))),
        text_color: Some(Color::from_rgb8(250, 250, 250)),
        border: Border {
            radius: TOKENS.radius.element.into(),
            color: Color::TRANSPARENT,
            width: TOKENS.border_width.hairline,
        },
        shadow: TOKENS.shadow.low.light,
        ..container::Style::default()
    }
}

#[allow(
    dead_code,
    reason = "retained reusable popover API after removing sidebar demo"
)]
pub(crate) fn popover_surface(theme: &Theme) -> container::Style {
    let mode = mode_from_theme(theme);

    container::Style {
        background: Some(Background::Color(
            TOKENS.colors.background_popover.get(mode),
        )),
        text_color: Some(TOKENS.colors.text_primary.get(mode)),
        border: Border {
            radius: TOKENS.radius.container.into(),
            color: TOKENS.colors.border.get(mode),
            width: TOKENS.border_width.regular,
        },
        shadow: TOKENS.shadow.medium.get(mode),
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use iced::{Background, Color, Theme};

    #[test]
    fn tooltip_surface_uses_inverted_dark_surface() {
        let style = super::tooltip_surface(&Theme::Light);

        assert_eq!(
            style.background,
            Some(Background::Color(Color::from_rgb8(23, 23, 23)))
        );
    }

    #[test]
    fn popover_surface_uses_theme_background() {
        let style = super::popover_surface(&Theme::Light);

        assert_eq!(
            style.background,
            Some(Background::Color(Color::from_rgb8(255, 255, 255)))
        );
    }
}
