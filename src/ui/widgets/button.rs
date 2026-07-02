use iced::widget::button;
use iced::{Background, Border, Color, Theme};

use crate::ui::tokens::{TOKENS, mode_from_theme};
use crate::ui::variants::ButtonVariant;

pub(crate) fn style(
    theme: &Theme,
    variant: ButtonVariant,
    status: button::Status,
) -> button::Style {
    let mode = mode_from_theme(theme);

    if matches!(status, button::Status::Disabled) {
        return button_style(
            Some(TOKENS.colors.background_muted.get(mode)),
            TOKENS.colors.text_disabled.get(mode),
            TOKENS.colors.border.get(mode),
            TOKENS.radius.element,
        );
    }

    match variant {
        ButtonVariant::Primary => {
            let background = match status {
                button::Status::Hovered => TOKENS.colors.accent_hover.get(mode),
                button::Status::Pressed => TOKENS.colors.accent_pressed.get(mode),
                button::Status::Active | button::Status::Disabled => TOKENS.colors.accent.get(mode),
            };

            button_style(
                Some(background),
                TOKENS.colors.on_accent.get(mode),
                background,
                TOKENS.radius.element,
            )
        }
        ButtonVariant::Secondary => {
            let background = match status {
                button::Status::Hovered => TOKENS.colors.badge_neutral_background.get(mode),
                button::Status::Pressed => TOKENS.colors.background_pressed.get(mode),
                button::Status::Active | button::Status::Disabled => {
                    TOKENS.colors.background_muted.get(mode)
                }
            };

            button_style(
                Some(background),
                TOKENS.colors.text_primary.get(mode),
                TOKENS.colors.border.get(mode),
                TOKENS.radius.element,
            )
        }
        ButtonVariant::Destructive => {
            let background = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    TOKENS.colors.error_muted.get(mode)
                }
                button::Status::Active | button::Status::Disabled => {
                    TOKENS.colors.background_muted.get(mode)
                }
            };

            button_style(
                Some(background),
                TOKENS.colors.error.get(mode),
                TOKENS.colors.error.get(mode),
                TOKENS.radius.element,
            )
        }
        ButtonVariant::Ghost => {
            let background = match status {
                button::Status::Hovered => Some(TOKENS.colors.background_hover.get(mode)),
                button::Status::Pressed => Some(TOKENS.colors.background_pressed.get(mode)),
                button::Status::Active | button::Status::Disabled => None,
            };

            button_style(
                background,
                TOKENS.colors.text_primary.get(mode),
                Color::TRANSPARENT,
                TOKENS.radius.element,
            )
        }
    }
}

pub(crate) fn selected(theme: &Theme, status: button::Status) -> button::Style {
    let mode = mode_from_theme(theme);
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => {
            TOKENS.colors.badge_neutral_background.get(mode)
        }
        button::Status::Active | button::Status::Disabled => TOKENS.colors.accent_muted.get(mode),
    };

    button_style(
        Some(background),
        TOKENS.colors.text_primary.get(mode),
        TOKENS.colors.accent.get(mode),
        TOKENS.radius.element,
    )
}

fn button_style(
    background: Option<Color>,
    text_color: Color,
    border_color: Color,
    radius: f32,
) -> button::Style {
    button::Style {
        background: background.map(Background::Color),
        text_color,
        border: Border {
            radius: radius.into(),
            color: border_color,
            width: TOKENS.border_width.regular,
        },
        ..button::Style::default()
    }
}

#[cfg(test)]
mod tests {
    use iced::{Background, Color};

    use crate::ui::variants::ButtonVariant;

    #[test]
    fn style_uses_disabled_text_when_button_is_disabled() {
        let style = super::style(
            &iced::Theme::Light,
            ButtonVariant::Primary,
            iced::widget::button::Status::Disabled,
        );

        assert_eq!(style.text_color, Color::from_rgb8(163, 163, 163));
    }

    #[test]
    fn style_uses_neutral_accent_for_primary_button() {
        let style = super::style(
            &iced::Theme::Light,
            ButtonVariant::Primary,
            iced::widget::button::Status::Active,
        );

        assert_eq!(
            style.background,
            Some(Background::Color(Color::from_rgb8(38, 38, 38)))
        );
    }
}
