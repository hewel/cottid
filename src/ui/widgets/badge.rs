use iced::widget::container;
use iced::{Background, Border, Color, Theme};

use crate::ui::tokens::{TOKENS, mode_from_theme};
use crate::ui::variants::BadgeVariant;

pub(crate) fn style(theme: &Theme, variant: BadgeVariant) -> container::Style {
    let mode = mode_from_theme(theme);
    let (background, text_color) = match variant {
        BadgeVariant::Neutral => (
            TOKENS.colors.badge_neutral_background.get(mode),
            TOKENS.colors.badge_neutral_text.get(mode),
        ),
        BadgeVariant::Success => (
            TOKENS.colors.success.get(mode),
            TOKENS.colors.on_success.get(mode),
        ),
        BadgeVariant::Warning => (
            Color::from_rgb8(255, 206, 47),
            TOKENS.colors.on_warning.get(mode),
        ),
        BadgeVariant::Error => (
            TOKENS.colors.error.get(mode),
            TOKENS.colors.on_error.get(mode),
        ),
        BadgeVariant::Blue => (
            TOKENS.colors.badge_blue_background.get(mode),
            TOKENS.colors.badge_blue_text.get(mode),
        ),
        BadgeVariant::Green => (
            TOKENS.colors.badge_green_background.get(mode),
            TOKENS.colors.badge_green_text.get(mode),
        ),
        BadgeVariant::Red => (
            TOKENS.colors.badge_red_background.get(mode),
            TOKENS.colors.badge_red_text.get(mode),
        ),
    };

    container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(text_color),
        border: Border {
            radius: TOKENS.radius.full.into(),
            color: Color::TRANSPARENT,
            width: TOKENS.border_width.hairline,
        },
        ..container::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use iced::{Background, Color};

    use crate::ui::variants::BadgeVariant;

    #[test]
    fn style_uses_filled_success_background_for_semantic_success_badge() {
        let style = super::style(&iced::Theme::Light, BadgeVariant::Success);

        assert_eq!(
            style.background,
            Some(Background::Color(Color::from_rgb8(0, 112, 4)))
        );
    }

    #[test]
    fn style_uses_tinted_background_for_categorical_blue_badge() {
        let style = super::style(&iced::Theme::Light, BadgeVariant::Blue);

        assert_eq!(
            style.background,
            Some(Background::Color(Color::from_rgb8(196, 221, 251)))
        );
    }
}
