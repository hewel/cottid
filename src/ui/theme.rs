use iced::widget::{button, container, progress_bar, text, text_input};
use iced::{Background, Border, Color, Theme};

const CARD_RADIUS: f32 = 10.0;
const CONTROL_RADIUS: f32 = 8.0;

#[derive(Debug, Clone, Copy)]
struct Palette {
    app_background: Color,
    sidebar: Color,
    surface: Color,
    surface_muted: Color,
    surface_hover: Color,
    text: Color,
    text_muted: Color,
    border: Color,
    accent: Color,
    accent_hover: Color,
    accent_pressed: Color,
    accent_soft: Color,
    danger: Color,
    danger_soft: Color,
    warning: Color,
    warning_soft: Color,
}

fn palette(theme: &Theme) -> Palette {
    if theme.extended_palette().is_dark {
        Palette {
            app_background: Color::from_rgb8(13, 18, 28),
            sidebar: Color::from_rgb8(31, 39, 53),
            surface: Color::from_rgb8(34, 43, 57),
            surface_muted: Color::from_rgb8(48, 59, 77),
            surface_hover: Color::from_rgb8(58, 70, 90),
            text: Color::from_rgb8(244, 247, 251),
            text_muted: Color::from_rgb8(166, 176, 193),
            border: Color::from_rgb8(66, 80, 103),
            accent: Color::from_rgb8(45, 212, 191),
            accent_hover: Color::from_rgb8(34, 211, 238),
            accent_pressed: Color::from_rgb8(14, 165, 233),
            accent_soft: Color::from_rgb8(37, 58, 76),
            danger: Color::from_rgb8(251, 113, 133),
            danger_soft: Color::from_rgb8(74, 39, 54),
            warning: Color::from_rgb8(251, 191, 36),
            warning_soft: Color::from_rgb8(79, 62, 30),
        }
    } else {
        Palette {
            app_background: Color::from_rgb8(244, 247, 251),
            sidebar: Color::from_rgb8(255, 255, 255),
            surface: Color::from_rgb8(255, 255, 255),
            surface_muted: Color::from_rgb8(238, 243, 249),
            surface_hover: Color::from_rgb8(226, 234, 244),
            text: Color::from_rgb8(26, 32, 44),
            text_muted: Color::from_rgb8(96, 112, 132),
            border: Color::from_rgb8(221, 229, 239),
            accent: Color::from_rgb8(37, 99, 235),
            accent_hover: Color::from_rgb8(29, 78, 216),
            accent_pressed: Color::from_rgb8(30, 64, 175),
            accent_soft: Color::from_rgb8(219, 234, 254),
            danger: Color::from_rgb8(220, 38, 38),
            danger_soft: Color::from_rgb8(254, 226, 226),
            warning: Color::from_rgb8(217, 119, 6),
            warning_soft: Color::from_rgb8(254, 243, 199),
        }
    }
}

pub fn text_color(theme: &Theme) -> Color {
    palette(theme).text
}

pub fn muted_color(theme: &Theme) -> Color {
    palette(theme).text_muted
}

pub fn accent_color(theme: &Theme) -> Color {
    palette(theme).accent
}

pub fn danger_color(theme: &Theme) -> Color {
    palette(theme).danger
}

pub fn warning_color(theme: &Theme) -> Color {
    palette(theme).warning
}

pub fn muted_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).text_muted),
    }
}

pub fn danger_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(palette(theme).danger),
    }
}

pub fn app_background(theme: &Theme) -> container::Style {
    let palette = palette(theme);
    container::Style {
        background: Some(Background::Color(palette.app_background)),
        text_color: Some(palette.text),
        ..container::Style::default()
    }
}

pub fn sidebar(theme: &Theme) -> container::Style {
    let palette = palette(theme);
    container::Style {
        background: Some(Background::Color(palette.sidebar)),
        text_color: Some(palette.text),
        border: border(CARD_RADIUS, palette.border, 1.0),
        ..container::Style::default()
    }
}

pub fn surface(theme: &Theme) -> container::Style {
    let palette = palette(theme);
    container::Style {
        background: Some(Background::Color(palette.surface)),
        text_color: Some(palette.text),
        border: border(CARD_RADIUS, palette.border, 1.0),
        ..container::Style::default()
    }
}

pub fn selected_surface(theme: &Theme) -> container::Style {
    let palette = palette(theme);
    container::Style {
        background: Some(Background::Color(palette.surface)),
        text_color: Some(palette.text),
        border: border(CARD_RADIUS, palette.accent, 1.0),
        ..container::Style::default()
    }
}

pub fn muted_surface(theme: &Theme) -> container::Style {
    let palette = palette(theme);
    container::Style {
        background: Some(Background::Color(palette.surface_muted)),
        text_color: Some(palette.text_muted),
        border: border(CARD_RADIUS, palette.border, 1.0),
        ..container::Style::default()
    }
}

pub fn search_surface(theme: &Theme) -> container::Style {
    let palette = palette(theme);
    container::Style {
        background: Some(Background::Color(palette.surface_muted)),
        text_color: Some(palette.text_muted),
        border: border(CONTROL_RADIUS, palette.border, 1.0),
        ..container::Style::default()
    }
}

pub fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);
    let background = match status {
        button::Status::Hovered => palette.accent_hover,
        button::Status::Pressed => palette.accent_pressed,
        _ => palette.accent,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: if theme.extended_palette().is_dark {
            Color::from_rgb8(8, 13, 23)
        } else {
            Color::WHITE
        },
        border: border(CONTROL_RADIUS, background, 1.0),
        ..button::Style::default()
    }
}

pub fn subtle_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);
    let background = match status {
        button::Status::Hovered => palette.surface_hover,
        button::Status::Pressed => palette.accent_soft,
        _ => palette.surface_muted,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.text,
        border: border(CONTROL_RADIUS, palette.border, 1.0),
        ..button::Style::default()
    }
}

pub fn selected_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);
    let background = match status {
        button::Status::Hovered => palette.surface_hover,
        button::Status::Pressed => palette.accent_soft,
        _ => palette.accent_soft,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.text,
        border: border(CONTROL_RADIUS, palette.accent, 1.0),
        ..button::Style::default()
    }
}

pub fn icon_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);
    let background = match status {
        button::Status::Hovered => palette.surface_hover,
        button::Status::Pressed => palette.accent_soft,
        _ => palette.surface_muted,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.text,
        border: border(CONTROL_RADIUS, palette.border, 1.0),
        ..button::Style::default()
    }
}

pub fn danger_button(theme: &Theme, status: button::Status) -> button::Style {
    let palette = palette(theme);
    let background = match status {
        button::Status::Hovered | button::Status::Pressed => palette.danger_soft,
        _ => palette.surface_muted,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: palette.danger,
        border: border(CONTROL_RADIUS, palette.danger, 1.0),
        ..button::Style::default()
    }
}

pub fn progress(theme: &Theme) -> progress_bar::Style {
    let palette = palette(theme);
    progress_bar::Style {
        background: Background::Color(palette.surface_muted),
        bar: Background::Color(palette.accent),
        border: border(6.0, palette.surface_muted, 0.0),
    }
}

pub fn form_text_input(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = palette(theme);
    let border_color = match status {
        text_input::Status::Focused { .. } => palette.accent,
        text_input::Status::Hovered => palette.text_muted,
        text_input::Status::Disabled => palette.border,
        text_input::Status::Active => palette.border,
    };
    let background = match status {
        text_input::Status::Hovered | text_input::Status::Focused { is_hovered: true } => {
            palette.surface_hover
        }
        text_input::Status::Disabled => palette.surface,
        _ => palette.surface_muted,
    };

    text_input::Style {
        background: Background::Color(background),
        border: border(CONTROL_RADIUS, border_color, 1.0),
        icon: palette.text_muted,
        placeholder: palette.text_muted,
        value: palette.text,
        selection: palette.accent_soft,
    }
}

pub fn feedback_info_surface(theme: &Theme) -> container::Style {
    let palette = palette(theme);
    container::Style {
        background: Some(Background::Color(palette.surface_muted)),
        text_color: Some(palette.text),
        border: border(CONTROL_RADIUS, palette.border, 1.0),
        ..container::Style::default()
    }
}

pub fn feedback_success_surface(theme: &Theme) -> container::Style {
    feedback_tinted_surface(
        theme,
        |palette| palette.accent_soft,
        |palette| palette.accent,
    )
}

pub fn feedback_warning_surface(theme: &Theme) -> container::Style {
    feedback_tinted_surface(
        theme,
        |palette| palette.warning_soft,
        |palette| palette.warning,
    )
}

pub fn feedback_error_surface(theme: &Theme) -> container::Style {
    feedback_tinted_surface(
        theme,
        |palette| palette.danger_soft,
        |palette| palette.danger,
    )
}

pub fn feedback_info_color(theme: &Theme) -> Color {
    palette(theme).text_muted
}

pub fn feedback_success_color(theme: &Theme) -> Color {
    palette(theme).accent
}

pub fn feedback_warning_color(theme: &Theme) -> Color {
    palette(theme).warning
}

pub fn feedback_error_color(theme: &Theme) -> Color {
    palette(theme).danger
}

fn feedback_tinted_surface(
    theme: &Theme,
    background: fn(Palette) -> Color,
    border_color: fn(Palette) -> Color,
) -> container::Style {
    let palette = palette(theme);
    container::Style {
        background: Some(Background::Color(background(palette))),
        text_color: Some(palette.text),
        border: border(CONTROL_RADIUS, border_color(palette), 1.0),
        ..container::Style::default()
    }
}

fn border(radius: f32, color: Color, width: f32) -> Border {
    Border {
        radius: radius.into(),
        color,
        width,
    }
}
