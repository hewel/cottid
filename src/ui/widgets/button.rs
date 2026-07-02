use iced::widget::button;
use iced::{Background, Border, Color, Theme};

use crate::ui::color;
use crate::ui::tokens::{InteractionOverlay, Mode, TOKENS, mode_from_theme};
use crate::ui::variants::ButtonVariant;

pub(crate) fn style(
    theme: &Theme,
    variant: ButtonVariant,
    status: button::Status,
) -> button::Style {
    let mode = mode_from_theme(theme);
    let base_background = base_background(mode, variant);
    let text_color = if matches!(status, button::Status::Disabled) {
        TOKENS.colors.text_disabled.get(mode)
    } else {
        text_color(mode, variant)
    };
    let background = interactive_background(
        base_background,
        status,
        TOKENS.interaction_overlay.get(mode),
    );

    button_style(background, text_color, TOKENS.radius.element)
}

fn base_background(mode: Mode, variant: ButtonVariant) -> Color {
    match variant {
        ButtonVariant::Primary => TOKENS.colors.accent.get(mode),
        ButtonVariant::Secondary => TOKENS.colors.background_muted.get(mode),
        ButtonVariant::Destructive => TOKENS.colors.error_muted.get(mode),
        ButtonVariant::Ghost => Color::TRANSPARENT,
    }
}

fn text_color(mode: Mode, variant: ButtonVariant) -> Color {
    match variant {
        ButtonVariant::Primary => TOKENS.colors.on_accent.get(mode),
        ButtonVariant::Secondary | ButtonVariant::Ghost => TOKENS.colors.text_primary.get(mode),
        ButtonVariant::Destructive => TOKENS.colors.error.get(mode),
    }
}

fn interactive_background(
    base: Color,
    status: button::Status,
    overlay: InteractionOverlay,
) -> Color {
    // CONTEXT: Astryx Web layers an interaction background over the base color.
    // iced has one background slot, so we pre-blend the overlay into one color.
    match status {
        button::Status::Active => base,
        button::Status::Hovered => overlay_background(base, overlay.hover),
        button::Status::Pressed => overlay_background(base, overlay.pressed),
        button::Status::Disabled => scale_alpha(base, 0.5),
    }
}

fn overlay_background(base: Color, overlay: Color) -> Color {
    if base.a <= f32::EPSILON {
        overlay
    } else {
        color::overlay(base, overlay)
    }
}

fn scale_alpha(color: Color, factor: f32) -> Color {
    Color {
        a: color.a * factor,
        ..color
    }
}

fn button_style(background: Color, text_color: Color, radius: f32) -> button::Style {
    button::Style {
        background: background_color(background),
        text_color,
        border: Border {
            radius: radius.into(),
            color: Color::TRANSPARENT,
            width: 0.0,
        },
        ..button::Style::default()
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
    use iced::{Background, Color};

    use crate::ui::color;
    use crate::ui::tokens::{Mode, TOKENS};
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

    #[test]
    fn style_leaves_all_buttons_without_visible_outline() {
        for variant in [
            ButtonVariant::Primary,
            ButtonVariant::Secondary,
            ButtonVariant::Destructive,
            ButtonVariant::Ghost,
        ] {
            let style = super::style(
                &iced::Theme::Light,
                variant,
                iced::widget::button::Status::Active,
            );

            assert_eq!(style.border.color, Color::TRANSPARENT);
            assert_eq!(style.border.width, 0.0);
        }
    }

    #[test]
    fn style_uses_overlay_as_background_when_ghost_button_is_hovered() {
        let style = super::style(
            &iced::Theme::Light,
            ButtonVariant::Ghost,
            iced::widget::button::Status::Hovered,
        );

        assert_eq!(
            style.background,
            Some(Background::Color(
                TOKENS.interaction_overlay.get(Mode::Light).hover
            ))
        );
    }

    #[test]
    fn style_blends_overlay_with_base_when_primary_button_is_hovered() {
        let style = super::style(
            &iced::Theme::Light,
            ButtonVariant::Primary,
            iced::widget::button::Status::Hovered,
        );
        let expected = color::overlay(
            TOKENS.colors.accent.get(Mode::Light),
            TOKENS.interaction_overlay.get(Mode::Light).hover,
        );

        assert_eq!(style.background, Some(Background::Color(expected)));
    }
}
