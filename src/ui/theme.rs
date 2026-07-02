use iced::widget::{button, container, progress_bar, text, text_input};
use iced::{Background, Border, Color, Theme};

use crate::ui::tokens::{DesignTokens, ToneTokens, design_tokens};
use crate::ui::variants::{
    ButtonVariant, FeedbackVariant, InputVariant, ProgressVariant, SurfaceVariant, TextVariant,
};

pub(crate) fn text_variant(theme: &Theme, variant: TextVariant) -> Color {
    let tokens = design_tokens(theme);
    match variant {
        TextVariant::Primary => tokens.text.primary,
        TextVariant::Muted => tokens.text.secondary,
        TextVariant::Accent => tokens.accent.base,
        TextVariant::Danger => tokens.status.error.color,
        TextVariant::Warning => tokens.status.warning.color,
    }
}

pub(crate) fn surface_variant(theme: &Theme, variant: SurfaceVariant) -> container::Style {
    let tokens = design_tokens(theme);
    match variant {
        SurfaceVariant::App => container_style(tokens.background.body, tokens.text.primary, None),
        SurfaceVariant::Sidebar => container_style(
            tokens.background.sidebar,
            tokens.text.primary,
            Some(border(tokens.radius.container, tokens.border.default, 1.0)),
        ),
        SurfaceVariant::Card => card_style(&tokens, tokens.border.default),
        SurfaceVariant::SelectedCard => card_style(&tokens, tokens.accent.base),
        SurfaceVariant::Muted => container_style(
            tokens.background.muted,
            tokens.text.secondary,
            Some(border(tokens.radius.container, tokens.border.default, 1.0)),
        ),
        SurfaceVariant::Search => container_style(
            tokens.background.muted,
            tokens.text.secondary,
            Some(border(tokens.radius.element, tokens.border.default, 1.0)),
        ),
        SurfaceVariant::Modal => container_style(
            tokens.background.surface,
            tokens.text.primary,
            Some(border(tokens.radius.container, tokens.border.default, 1.0)),
        ),
        SurfaceVariant::Feedback(variant) => {
            let tone = feedback_tone(&tokens, variant);
            container_style(
                tone.muted,
                tokens.text.primary,
                Some(border(tokens.radius.element, tone.color, 1.0)),
            )
        }
    }
}

pub(crate) fn button_variant(
    theme: &Theme,
    status: button::Status,
    variant: ButtonVariant,
) -> button::Style {
    let tokens = design_tokens(theme);
    match variant {
        ButtonVariant::Primary => {
            let background = match status {
                button::Status::Hovered => tokens.accent.hover,
                button::Status::Pressed => tokens.accent.pressed,
                _ => tokens.accent.base,
            };
            button_style(
                background,
                tokens.text.on_accent,
                border(tokens.radius.element, background, 1.0),
            )
        }
        ButtonVariant::Subtle | ButtonVariant::Icon => {
            let background = match status {
                button::Status::Hovered => tokens.background.hover,
                button::Status::Pressed => tokens.accent.muted,
                _ => tokens.background.muted,
            };
            button_style(
                background,
                tokens.text.primary,
                border(tokens.radius.element, tokens.border.default, 1.0),
            )
        }
        ButtonVariant::Selected => {
            let background = match status {
                button::Status::Hovered => tokens.background.hover,
                _ => tokens.accent.muted,
            };
            button_style(
                background,
                tokens.text.primary,
                border(tokens.radius.element, tokens.accent.base, 1.0),
            )
        }
        ButtonVariant::Danger => {
            let background = match status {
                button::Status::Hovered | button::Status::Pressed => tokens.status.error.muted,
                _ => tokens.background.muted,
            };
            button_style(
                background,
                tokens.status.error.color,
                border(tokens.radius.element, tokens.status.error.color, 1.0),
            )
        }
    }
}

pub(crate) fn input_variant(
    theme: &Theme,
    status: text_input::Status,
    variant: InputVariant,
) -> text_input::Style {
    match variant {
        InputVariant::Form => {
            let tokens = design_tokens(theme);
            let border_color = match status {
                text_input::Status::Focused { .. } => tokens.accent.base,
                text_input::Status::Hovered => tokens.border.emphasized,
                text_input::Status::Disabled => tokens.border.default,
                text_input::Status::Active => tokens.border.default,
            };
            let background = match status {
                text_input::Status::Hovered | text_input::Status::Focused { is_hovered: true } => {
                    tokens.background.hover
                }
                text_input::Status::Disabled => tokens.background.surface,
                _ => tokens.background.muted,
            };

            text_input::Style {
                background: Background::Color(background),
                border: border(tokens.radius.element, border_color, 1.0),
                icon: tokens.text.secondary,
                placeholder: tokens.text.secondary,
                value: tokens.text.primary,
                selection: tokens.accent.muted,
            }
        }
    }
}

pub(crate) fn progress_variant(theme: &Theme, variant: ProgressVariant) -> progress_bar::Style {
    match variant {
        ProgressVariant::Accent => {
            let tokens = design_tokens(theme);
            progress_bar::Style {
                background: Background::Color(tokens.background.muted),
                bar: Background::Color(tokens.accent.base),
                border: border(tokens.radius.progress, tokens.background.muted, 0.0),
            }
        }
    }
}

pub(crate) fn feedback_color(theme: &Theme, variant: FeedbackVariant) -> Color {
    let tokens = design_tokens(theme);
    feedback_tone(&tokens, variant).color
}

pub fn text_color(theme: &Theme) -> Color {
    text_variant(theme, TextVariant::Primary)
}

pub fn muted_color(theme: &Theme) -> Color {
    text_variant(theme, TextVariant::Muted)
}

pub fn accent_color(theme: &Theme) -> Color {
    text_variant(theme, TextVariant::Accent)
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

pub fn subtle_button(theme: &Theme, status: button::Status) -> button::Style {
    button_variant(theme, status, ButtonVariant::Subtle)
}

pub fn selected_button(theme: &Theme, status: button::Status) -> button::Style {
    button_variant(theme, status, ButtonVariant::Selected)
}

pub fn icon_button(theme: &Theme, status: button::Status) -> button::Style {
    button_variant(theme, status, ButtonVariant::Icon)
}

pub fn danger_button(theme: &Theme, status: button::Status) -> button::Style {
    button_variant(theme, status, ButtonVariant::Danger)
}

pub fn progress(theme: &Theme) -> progress_bar::Style {
    progress_variant(theme, ProgressVariant::Accent)
}

pub fn form_text_input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    input_variant(theme, status, InputVariant::Form)
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

fn card_style(tokens: &DesignTokens, border_color: Color) -> container::Style {
    container_style(
        tokens.background.card,
        tokens.text.primary,
        Some(border(tokens.radius.container, border_color, 1.0)),
    )
}

fn container_style(
    background: Color,
    text_color: Color,
    border: Option<Border>,
) -> container::Style {
    container::Style {
        background: Some(Background::Color(background)),
        text_color: Some(text_color),
        border: border.unwrap_or_default(),
        ..container::Style::default()
    }
}

fn button_style(background: Color, text_color: Color, border: Border) -> button::Style {
    button::Style {
        background: Some(Background::Color(background)),
        text_color,
        border,
        ..button::Style::default()
    }
}

fn feedback_tone(tokens: &DesignTokens, variant: FeedbackVariant) -> ToneTokens {
    match variant {
        FeedbackVariant::Info => tokens.status.info,
        FeedbackVariant::Success => tokens.status.success,
        FeedbackVariant::Warning => tokens.status.warning,
        FeedbackVariant::Error => tokens.status.error,
    }
}

fn border(radius: f32, color: Color, width: f32) -> Border {
    Border {
        radius: radius.into(),
        color,
        width,
    }
}
