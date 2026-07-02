use iced::{Color, Theme};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DesignTokens {
    pub(crate) background: BackgroundTokens,
    pub(crate) text: TextTokens,
    pub(crate) border: BorderTokens,
    pub(crate) accent: AccentTokens,
    pub(crate) status: StatusTokens,
    pub(crate) radius: RadiusTokens,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BackgroundTokens {
    pub(crate) body: Color,
    pub(crate) sidebar: Color,
    pub(crate) card: Color,
    pub(crate) surface: Color,
    pub(crate) muted: Color,
    pub(crate) hover: Color,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TextTokens {
    pub(crate) primary: Color,
    pub(crate) secondary: Color,
    pub(crate) on_accent: Color,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BorderTokens {
    pub(crate) default: Color,
    pub(crate) emphasized: Color,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AccentTokens {
    pub(crate) base: Color,
    pub(crate) hover: Color,
    pub(crate) pressed: Color,
    pub(crate) muted: Color,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StatusTokens {
    pub(crate) info: ToneTokens,
    pub(crate) success: ToneTokens,
    pub(crate) warning: ToneTokens,
    pub(crate) error: ToneTokens,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ToneTokens {
    pub(crate) color: Color,
    pub(crate) muted: Color,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RadiusTokens {
    pub(crate) element: f32,
    pub(crate) container: f32,
    pub(crate) progress: f32,
}

pub(crate) fn design_tokens(theme: &Theme) -> DesignTokens {
    if theme.extended_palette().is_dark {
        dark_tokens()
    } else {
        light_tokens()
    }
}

fn light_tokens() -> DesignTokens {
    DesignTokens {
        background: BackgroundTokens {
            body: Color::from_rgb8(241, 241, 241),
            sidebar: Color::from_rgb8(255, 255, 255),
            card: Color::from_rgb8(255, 255, 255),
            surface: Color::from_rgb8(255, 255, 255),
            muted: Color::from_rgb8(241, 241, 241),
            hover: Color::from_rgb8(229, 229, 229),
        },
        text: TextTokens {
            primary: Color::from_rgb8(23, 23, 23),
            secondary: Color::from_rgb8(115, 115, 115),
            on_accent: Color::from_rgb8(255, 255, 255),
        },
        border: BorderTokens {
            default: Color::from_rgb8(235, 235, 235),
            emphasized: Color::from_rgb8(212, 212, 212),
        },
        accent: AccentTokens {
            base: Color::from_rgb8(38, 38, 38),
            hover: Color::from_rgb8(64, 64, 64),
            pressed: Color::from_rgb8(23, 23, 23),
            muted: Color::from_rgb8(241, 241, 241),
        },
        status: StatusTokens {
            info: ToneTokens {
                color: Color::from_rgb8(0, 69, 140),
                muted: Color::from_rgb8(196, 221, 251),
            },
            success: ToneTokens {
                color: Color::from_rgb8(0, 112, 4),
                muted: Color::from_rgb8(197, 229, 192),
            },
            warning: ToneTokens {
                color: Color::from_rgb8(116, 91, 0),
                muted: Color::from_rgb8(248, 218, 157),
            },
            error: ToneTokens {
                color: Color::from_rgb8(165, 12, 37),
                muted: Color::from_rgb8(250, 206, 203),
            },
        },
        radius: RadiusTokens {
            element: 8.0,
            container: 10.0,
            progress: 6.0,
        },
    }
}

fn dark_tokens() -> DesignTokens {
    DesignTokens {
        background: BackgroundTokens {
            body: Color::from_rgb8(27, 27, 27),
            sidebar: Color::from_rgb8(27, 27, 27),
            card: Color::from_rgb8(27, 27, 27),
            surface: Color::from_rgb8(38, 38, 38),
            muted: Color::from_rgb8(38, 38, 38),
            hover: Color::from_rgb8(64, 64, 64),
        },
        text: TextTokens {
            primary: Color::from_rgb8(250, 250, 250),
            secondary: Color::from_rgb8(163, 163, 163),
            on_accent: Color::from_rgb8(23, 23, 23),
        },
        border: BorderTokens {
            default: Color::from_rgba8(255, 255, 255, 0.10),
            emphasized: Color::from_rgb8(82, 82, 82),
        },
        accent: AccentTokens {
            base: Color::from_rgb8(235, 235, 235),
            hover: Color::from_rgb8(245, 245, 245),
            pressed: Color::from_rgb8(212, 212, 212),
            muted: Color::from_rgb8(38, 38, 38),
        },
        status: StatusTokens {
            info: ToneTokens {
                color: Color::from_rgb8(199, 211, 255),
                muted: Color::from_rgba8(158, 183, 255, 0.24),
            },
            success: ToneTokens {
                color: Color::from_rgb8(159, 229, 155),
                muted: Color::from_rgba8(132, 201, 128, 0.24),
            },
            warning: ToneTokens {
                color: Color::from_rgb8(253, 207, 79),
                muted: Color::from_rgba8(222, 180, 51, 0.24),
            },
            error: ToneTokens {
                color: Color::from_rgb8(255, 198, 193),
                muted: Color::from_rgba8(255, 158, 151, 0.24),
            },
        },
        radius: RadiusTokens {
            element: 8.0,
            container: 10.0,
            progress: 6.0,
        },
    }
}
