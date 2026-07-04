use iced::widget::scrollable;
use iced::{Background, Border, Color, Shadow, Theme, Vector};

use crate::ui::tokens::{Mode, TOKENS, mode_from_theme};

pub(crate) fn style(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let mode = mode_from_theme(theme);
    let rail = rail(mode, scroller_background(mode, status));

    scrollable::Style {
        container: Default::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: Some(Background::Color(TOKENS.colors.background_muted.get(mode))),
        auto_scroll: auto_scroll(mode),
    }
}

fn rail(mode: Mode, scroller_background: Color) -> scrollable::Rail {
    scrollable::Rail {
        background: Some(Background::Color(TOKENS.colors.background_muted.get(mode))),
        border: Border {
            radius: TOKENS.radius.full.into(),
            color: Color::TRANSPARENT,
            width: TOKENS.border_width.hairline,
        },
        scroller: scrollable::Scroller {
            background: Background::Color(scroller_background),
            border: Border {
                radius: TOKENS.radius.full.into(),
                color: Color::TRANSPARENT,
                width: TOKENS.border_width.hairline,
            },
        },
    }
}

fn scroller_background(mode: Mode, status: scrollable::Status) -> Color {
    match status {
        scrollable::Status::Active { .. } => TOKENS.colors.background_hover.get(mode),
        scrollable::Status::Hovered { .. } => TOKENS.colors.background_pressed.get(mode),
        scrollable::Status::Dragged { .. } => TOKENS.colors.border_emphasized.get(mode),
    }
}

fn auto_scroll(mode: Mode) -> scrollable::AutoScroll {
    scrollable::AutoScroll {
        background: Background::Color(TOKENS.colors.background_surface.get(mode)),
        border: Border {
            radius: TOKENS.radius.full.into(),
            color: TOKENS.colors.border.get(mode),
            width: TOKENS.border_width.regular,
        },
        shadow: Shadow {
            color: Color::BLACK.scale_alpha(0.18),
            offset: Vector::ZERO,
            blur_radius: 4.0,
        },
        icon: TOKENS.colors.text_secondary.get(mode),
    }
}

#[cfg(test)]
mod tests {
    use iced::Background;

    use crate::ui::tokens::{Mode, TOKENS};

    #[test]
    fn style_uses_token_rail_background() {
        let style = super::style(
            &iced::Theme::Light,
            iced::widget::scrollable::Status::Active {
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );

        assert_eq!(
            style.vertical_rail.background,
            Some(Background::Color(
                TOKENS.colors.background_muted.get(Mode::Light)
            ))
        );
    }

    #[test]
    fn style_uses_soft_thumb_when_active() {
        let style = super::style(
            &iced::Theme::Light,
            iced::widget::scrollable::Status::Active {
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );

        assert_eq!(
            style.vertical_rail.scroller.background,
            Background::Color(TOKENS.colors.background_hover.get(Mode::Light))
        );
    }

    #[test]
    fn style_uses_stronger_thumb_when_hovered() {
        let style = super::style(
            &iced::Theme::Light,
            iced::widget::scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered: false,
                is_vertical_scrollbar_hovered: true,
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );

        assert_eq!(
            style.vertical_rail.scroller.background,
            Background::Color(TOKENS.colors.background_pressed.get(Mode::Light))
        );
    }

    #[test]
    fn style_uses_accent_thumb_when_dragged() {
        let style = super::style(
            &iced::Theme::Dark,
            iced::widget::scrollable::Status::Dragged {
                is_horizontal_scrollbar_dragged: false,
                is_vertical_scrollbar_dragged: true,
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );

        assert_eq!(
            style.vertical_rail.scroller.background,
            Background::Color(TOKENS.colors.border_emphasized.get(Mode::Dark))
        );
    }

    #[test]
    fn style_rounds_rail_and_thumb_with_scrollbar_tokens() {
        let style = super::style(
            &iced::Theme::Light,
            iced::widget::scrollable::Status::Active {
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );

        assert_eq!(style.vertical_rail.border.radius, TOKENS.radius.full.into());
        assert_eq!(
            style.vertical_rail.scroller.border.radius,
            TOKENS.radius.full.into()
        );
    }
}
