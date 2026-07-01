use iced::widget::{button, container, progress_bar};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

pub const APP_BACKGROUND: Color = Color::from_rgb8(244, 247, 251);
pub const SURFACE: Color = Color::from_rgb8(255, 255, 255);
pub const SURFACE_MUTED: Color = Color::from_rgb8(238, 243, 249);
pub const TEXT: Color = Color::from_rgb8(26, 32, 44);
pub const TEXT_MUTED: Color = Color::from_rgb8(96, 112, 132);
pub const BLUE: Color = Color::from_rgb8(37, 99, 235);
pub const BLUE_SOFT: Color = Color::from_rgb8(219, 234, 254);
pub const AMBER: Color = Color::from_rgb8(217, 119, 6);
pub const RED: Color = Color::from_rgb8(220, 38, 38);
pub const RED_SOFT: Color = Color::from_rgb8(254, 226, 226);
pub const BORDER: Color = Color::from_rgb8(221, 229, 239);

pub fn app_background(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(APP_BACKGROUND)),
        text_color: Some(TEXT),
        ..container::Style::default()
    }
}

pub fn sidebar(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE)),
        text_color: Some(TEXT),
        border: border(18.0, BORDER, 1.0),
        shadow: soft_shadow(),
        ..container::Style::default()
    }
}

pub fn surface(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE)),
        text_color: Some(TEXT),
        border: border(14.0, BORDER, 1.0),
        shadow: soft_shadow(),
        ..container::Style::default()
    }
}

pub fn selected_surface(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(250, 253, 255))),
        text_color: Some(TEXT),
        border: border(14.0, BLUE, 1.0),
        shadow: soft_shadow(),
        ..container::Style::default()
    }
}

pub fn muted_surface(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SURFACE_MUTED)),
        text_color: Some(TEXT_MUTED),
        border: border(10.0, BORDER, 1.0),
        ..container::Style::default()
    }
}

pub fn status_strip(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(248, 250, 252))),
        text_color: Some(TEXT_MUTED),
        border: border(12.0, BORDER, 1.0),
        ..container::Style::default()
    }
}

pub fn primary_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => Color::from_rgb8(29, 78, 216),
        button::Status::Pressed => Color::from_rgb8(30, 64, 175),
        _ => BLUE,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::WHITE,
        border: border(10.0, background, 1.0),
        shadow: small_shadow(),
        ..button::Style::default()
    }
}

pub fn subtle_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => BLUE_SOFT,
        button::Status::Pressed => Color::from_rgb8(191, 219, 254),
        _ => SURFACE_MUTED,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: border(10.0, BORDER, 1.0),
        ..button::Style::default()
    }
}

pub fn icon_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => SURFACE_MUTED,
        button::Status::Pressed => Color::from_rgb8(226, 232, 240),
        _ => SURFACE,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: TEXT,
        border: border(10.0, BORDER, 1.0),
        ..button::Style::default()
    }
}

pub fn danger_button(_theme: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Hovered => RED_SOFT,
        button::Status::Pressed => Color::from_rgb8(254, 202, 202),
        _ => SURFACE,
    };

    button::Style {
        background: Some(Background::Color(background)),
        text_color: RED,
        border: border(10.0, Color::from_rgb8(254, 202, 202), 1.0),
        ..button::Style::default()
    }
}

pub fn progress(_theme: &Theme) -> progress_bar::Style {
    progress_bar::Style {
        background: Background::Color(SURFACE_MUTED),
        bar: Background::Color(BLUE),
        border: border(6.0, SURFACE_MUTED, 0.0),
    }
}

fn border(radius: f32, color: Color, width: f32) -> Border {
    Border {
        radius: radius.into(),
        color,
        width,
    }
}

fn soft_shadow() -> Shadow {
    Shadow {
        color: Color {
            a: 0.08,
            ..Color::BLACK
        },
        offset: Vector::new(0.0, 6.0),
        blur_radius: 18.0,
    }
}

fn small_shadow() -> Shadow {
    Shadow {
        color: Color {
            a: 0.10,
            ..Color::BLACK
        },
        offset: Vector::new(0.0, 2.0),
        blur_radius: 6.0,
    }
}
