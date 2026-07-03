use iced::theme::Palette;
use iced::widget::{button, container, progress_bar, text};
use iced::{Background, Border, Color, Theme};

use crate::ui::tokens::{Mode, TOKENS, mode_from_theme};
use crate::ui::variants::{
    ButtonVariant, FeedbackVariant, ProgressVariant, SurfaceVariant, TextVariant,
};
use crate::ui::widgets;

pub(crate) fn iced_theme_for_mode(mode: Mode) -> Theme {
    let name = match mode {
        Mode::Light => "Cottid Neutral Light",
        Mode::Dark => "Cottid Neutral Dark",
    };

    Theme::custom(name, palette_for_mode(mode))
}

pub(crate) fn palette_for_mode(mode: Mode) -> Palette {
    Palette {
        background: TOKENS.colors.background_body.get(mode),
        text: TOKENS.colors.text_primary.get(mode),
        primary: TOKENS.colors.accent.get(mode),
        success: TOKENS.colors.success.get(mode),
        warning: TOKENS.colors.warning.get(mode),
        danger: TOKENS.colors.error.get(mode),
    }
}

pub(crate) fn text_variant(theme: &Theme, variant: TextVariant) -> Color {
    let mode = mode_from_theme(theme);
    match variant {
        TextVariant::Primary => TOKENS.colors.text_primary.get(mode),
        TextVariant::Muted => TOKENS.colors.text_secondary.get(mode),
        TextVariant::Info => TOKENS.colors.badge_blue_text.get(mode),
        TextVariant::Danger => TOKENS.colors.error.get(mode),
        TextVariant::Warning => TOKENS.colors.warning.get(mode),
    }
}

pub(crate) fn surface_variant(theme: &Theme, variant: SurfaceVariant) -> container::Style {
    match variant {
        SurfaceVariant::App => widgets::container::app(theme),
        SurfaceVariant::Sidebar => widgets::container::surface(theme),
        SurfaceVariant::Card => widgets::container::card(theme),
        SurfaceVariant::SelectedCard => widgets::container::selected_card(theme),
        SurfaceVariant::Muted => widgets::container::muted(theme),
        SurfaceVariant::Search => widgets::container::search(theme),
        SurfaceVariant::Modal => widgets::container::modal(theme),
        SurfaceVariant::Scrim => widgets::container::scrim(theme),
        SurfaceVariant::Feedback(variant) => {
            let mode = mode_from_theme(theme);
            let (background, border_color) = feedback_surface_colors(mode, variant);
            widgets::container::feedback(theme, background, border_color)
        }
    }
}

pub(crate) fn button_variant(
    theme: &Theme,
    status: button::Status,
    variant: ButtonVariant,
) -> button::Style {
    widgets::button::style(theme, variant, status)
}

pub(crate) fn progress_variant(theme: &Theme, variant: ProgressVariant) -> progress_bar::Style {
    let mode = mode_from_theme(theme);
    match variant {
        ProgressVariant::Accent => progress_bar::Style {
            background: Background::Color(TOKENS.colors.border_emphasized.get(mode)),
            bar: Background::Color(TOKENS.colors.info.get(mode)),
            border: Border {
                radius: TOKENS.radius.progress.into(),
                color: Color::TRANSPARENT,
                width: TOKENS.border_width.hairline,
            },
        },
    }
}

pub(crate) fn feedback_color(theme: &Theme, variant: FeedbackVariant) -> Color {
    let mode = mode_from_theme(theme);
    match variant {
        FeedbackVariant::Info => TOKENS.colors.badge_blue_text.get(mode),
        FeedbackVariant::Success => TOKENS.colors.success.get(mode),
        FeedbackVariant::Warning => TOKENS.colors.warning.get(mode),
        FeedbackVariant::Error => TOKENS.colors.error.get(mode),
    }
}

pub fn text_color(theme: &Theme) -> Color {
    text_variant(theme, TextVariant::Primary)
}

pub fn muted_color(theme: &Theme) -> Color {
    text_variant(theme, TextVariant::Muted)
}

pub fn accent_color(theme: &Theme) -> Color {
    text_variant(theme, TextVariant::Info)
}

pub fn danger_color(theme: &Theme) -> Color {
    text_variant(theme, TextVariant::Danger)
}

pub fn warning_color(theme: &Theme) -> Color {
    text_variant(theme, TextVariant::Warning)
}

pub fn muted_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(muted_color(theme)),
    }
}

pub fn danger_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(danger_color(theme)),
    }
}

pub fn icon_button(theme: &Theme, status: button::Status) -> button::Style {
    button_variant(theme, status, ButtonVariant::Ghost)
}

pub fn danger_button(theme: &Theme, status: button::Status) -> button::Style {
    button_variant(theme, status, ButtonVariant::Destructive)
}

pub fn progress(theme: &Theme) -> progress_bar::Style {
    progress_variant(theme, ProgressVariant::Accent)
}

pub fn feedback_info_color(theme: &Theme) -> Color {
    feedback_color(theme, FeedbackVariant::Info)
}

pub fn feedback_success_color(theme: &Theme) -> Color {
    feedback_color(theme, FeedbackVariant::Success)
}

pub fn feedback_warning_color(theme: &Theme) -> Color {
    feedback_color(theme, FeedbackVariant::Warning)
}

pub fn feedback_error_color(theme: &Theme) -> Color {
    feedback_color(theme, FeedbackVariant::Error)
}

fn feedback_surface_colors(mode: Mode, variant: FeedbackVariant) -> (Color, Color) {
    match variant {
        FeedbackVariant::Info => (
            TOKENS.colors.info_muted.get(mode),
            TOKENS.colors.badge_blue_text.get(mode),
        ),
        FeedbackVariant::Success => (
            TOKENS.colors.success_muted.get(mode),
            TOKENS.colors.success.get(mode),
        ),
        FeedbackVariant::Warning => (
            TOKENS.colors.warning_muted.get(mode),
            TOKENS.colors.warning.get(mode),
        ),
        FeedbackVariant::Error => (
            TOKENS.colors.error_muted.get(mode),
            TOKENS.colors.error.get(mode),
        ),
    }
}

#[cfg(test)]
mod tests {
    use iced::{Background, Color};

    use crate::ui::tokens::{Mode, TOKENS};
    use crate::ui::variants::SurfaceVariant;

    #[test]
    fn palette_for_mode_maps_only_basic_iced_palette_fields() {
        let palette = super::palette_for_mode(Mode::Light);

        assert_eq!(palette.background, Color::from_rgb8(241, 241, 241));
    }

    #[test]
    fn iced_theme_for_mode_generates_dark_theme_for_dark_mode() {
        let theme = super::iced_theme_for_mode(Mode::Dark);

        assert!(theme.extended_palette().is_dark);
    }

    #[test]
    fn modal_surface_uses_astryx_inspired_elevation_without_border() {
        let style = super::surface_variant(&iced::Theme::Light, SurfaceVariant::Modal);

        assert_eq!(
            style.background,
            Some(Background::Color(Color::from_rgb8(255, 255, 255)))
        );
        assert_eq!(style.border.width, TOKENS.border_width.hairline);
        assert_eq!(style.border.color, Color::TRANSPARENT);
        assert_eq!(style.shadow, TOKENS.shadow.high.get(Mode::Light));
    }

    #[test]
    fn scrim_surface_uses_astryx_overlay_alpha() {
        let light = super::surface_variant(&iced::Theme::Light, SurfaceVariant::Scrim);
        let dark = super::surface_variant(&iced::Theme::Dark, SurfaceVariant::Scrim);

        assert_eq!(
            light.background,
            Some(Background::Color(Color::from_rgba8(0, 0, 0, 0.40)))
        );
        assert_eq!(
            dark.background,
            Some(Background::Color(Color::from_rgba8(0, 0, 0, 0.64)))
        );
    }
}
