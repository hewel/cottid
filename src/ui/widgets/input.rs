use iced::widget::text_input;
use iced::{Background, Border, Theme};

use crate::ui::tokens::{TOKENS, mode_from_theme};

pub(crate) fn style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let mode = mode_from_theme(theme);
    let border_color = match status {
        text_input::Status::Focused { .. } => TOKENS.colors.accent.get(mode),
        text_input::Status::Hovered => TOKENS.colors.border_emphasized.get(mode),
        text_input::Status::Disabled | text_input::Status::Active => TOKENS.colors.border.get(mode),
    };
    let background = match status {
        text_input::Status::Hovered | text_input::Status::Focused { is_hovered: true } => {
            TOKENS.colors.background_surface.get(mode)
        }
        text_input::Status::Disabled => TOKENS.colors.background_muted.get(mode),
        text_input::Status::Focused { is_hovered: false } | text_input::Status::Active => {
            TOKENS.colors.background_surface.get(mode)
        }
    };

    text_input::Style {
        background: Background::Color(background),
        border: Border {
            radius: TOKENS.radius.element.into(),
            color: border_color,
            width: TOKENS.border_width.regular,
        },
        icon: TOKENS.colors.text_secondary.get(mode),
        placeholder: TOKENS.colors.text_secondary.get(mode),
        value: if matches!(status, text_input::Status::Disabled) {
            TOKENS.colors.text_disabled.get(mode)
        } else {
            TOKENS.colors.text_primary.get(mode)
        },
        selection: TOKENS.colors.info_muted.get(mode),
    }
}

#[cfg(test)]
mod tests {
    use iced::Color;
    use iced::widget::text_input;

    #[test]
    fn style_uses_accent_border_when_input_is_focused() {
        let style = super::style(
            &iced::Theme::Light,
            text_input::Status::Focused { is_hovered: false },
        );

        assert_eq!(style.border.color, Color::from_rgb8(38, 38, 38));
    }
}
