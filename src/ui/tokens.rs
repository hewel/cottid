use iced::{Color, Shadow, Theme, Vector};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Pair<T> {
    pub(crate) light: T,
    pub(crate) dark: T,
}

impl<T: Copy> Pair<T> {
    pub(crate) const fn get(self, mode: Mode) -> T {
        match mode {
            Mode::Light => self.light,
            Mode::Dark => self.dark,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct DesignTokens {
    pub(crate) colors: AppColors,
    pub(crate) interaction_overlay: Pair<InteractionOverlay>,
    pub(crate) spacing: AppSpacing,
    pub(crate) radius: AppRadius,
    pub(crate) scrollbar: AppScrollbar,
    pub(crate) border_width: AppBorderWidth,
    pub(crate) shadow: AppShadow,
    pub(crate) typography: AppTypography,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct InteractionOverlay {
    pub(crate) hover: Color,
    pub(crate) pressed: Color,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct AppColors {
    pub(crate) background_body: Pair<Color>,
    pub(crate) background_surface: Pair<Color>,
    pub(crate) background_card: Pair<Color>,
    pub(crate) background_popover: Pair<Color>,
    pub(crate) background_muted: Pair<Color>,
    pub(crate) background_scrim: Pair<Color>,
    pub(crate) background_hover: Pair<Color>,
    pub(crate) background_pressed: Pair<Color>,

    pub(crate) text_primary: Pair<Color>,
    pub(crate) text_secondary: Pair<Color>,
    pub(crate) text_disabled: Pair<Color>,

    pub(crate) border: Pair<Color>,
    pub(crate) border_emphasized: Pair<Color>,

    pub(crate) accent: Pair<Color>,
    pub(crate) accent_hover: Pair<Color>,
    pub(crate) accent_pressed: Pair<Color>,
    pub(crate) accent_muted: Pair<Color>,
    pub(crate) on_accent: Pair<Color>,

    pub(crate) info: Pair<Color>,
    pub(crate) on_info: Pair<Color>,
    pub(crate) info_muted: Pair<Color>,

    pub(crate) success: Pair<Color>,
    pub(crate) warning: Pair<Color>,
    pub(crate) error: Pair<Color>,
    pub(crate) on_success: Pair<Color>,
    pub(crate) on_warning: Pair<Color>,
    pub(crate) on_error: Pair<Color>,

    pub(crate) success_muted: Pair<Color>,
    pub(crate) warning_muted: Pair<Color>,
    pub(crate) error_muted: Pair<Color>,

    pub(crate) badge_neutral_background: Pair<Color>,
    pub(crate) badge_neutral_text: Pair<Color>,
    pub(crate) badge_blue_background: Pair<Color>,
    pub(crate) badge_blue_text: Pair<Color>,
    pub(crate) badge_green_background: Pair<Color>,
    pub(crate) badge_green_text: Pair<Color>,
    pub(crate) badge_red_background: Pair<Color>,
    pub(crate) badge_red_text: Pair<Color>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct AppSpacing {
    pub(crate) s1: f32,
    pub(crate) s2: f32,
    pub(crate) s3: f32,
    pub(crate) s4: f32,
    pub(crate) s5: f32,
    pub(crate) s6: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct AppRadius {
    pub(crate) none: f32,
    pub(crate) inner: f32,
    pub(crate) element: f32,
    pub(crate) container: f32,
    pub(crate) page: f32,
    pub(crate) full: f32,
    pub(crate) progress: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct AppScrollbar {
    pub(crate) rail_width: f32,
    pub(crate) thumb_width: f32,
    pub(crate) content_gap: f32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppBorderWidth {
    pub(crate) hairline: f32,
    pub(crate) regular: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct AppShadow {
    pub(crate) none: Shadow,
    pub(crate) low: Pair<Shadow>,
    pub(crate) medium: Pair<Shadow>,
    pub(crate) high: Pair<Shadow>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub(crate) struct AppTypography {
    pub(crate) body: u32,
    pub(crate) label: u32,
    pub(crate) caption: u32,
    pub(crate) heading: u32,
    pub(crate) display: u32,
}

pub(crate) const TOKENS: DesignTokens = DesignTokens {
    colors: AppColors {
        background_body: pair_rgb((241, 241, 241), (27, 27, 27)),
        background_surface: pair_rgb((255, 255, 255), (38, 38, 38)),
        background_card: pair_rgb((255, 255, 255), (27, 27, 27)),
        background_popover: pair_rgb((255, 255, 255), (27, 27, 27)),
        background_muted: pair_rgb((241, 241, 241), (27, 27, 27)),
        background_scrim: Pair {
            light: Color::from_rgba8(0, 0, 0, 0.40),
            dark: Color::from_rgba8(0, 0, 0, 0.64),
        },
        background_hover: Pair {
            light: Color::from_rgba8(0, 0, 0, 0.05),
            dark: Color::from_rgba8(255, 255, 255, 0.05),
        },
        background_pressed: Pair {
            light: Color::from_rgba8(0, 0, 0, 0.10),
            dark: Color::from_rgba8(255, 255, 255, 0.10),
        },

        text_primary: pair_rgb((23, 23, 23), (250, 250, 250)),
        text_secondary: pair_rgb((115, 115, 115), (163, 163, 163)),
        text_disabled: pair_rgb((163, 163, 163), (82, 82, 82)),

        border: Pair {
            light: Color::from_rgb8(235, 235, 235),
            dark: Color::from_rgba8(255, 255, 255, 0.10),
        },
        border_emphasized: pair_rgb((212, 212, 212), (82, 82, 82)),

        accent: pair_rgb((38, 38, 38), (235, 235, 235)),
        accent_hover: pair_rgb((64, 64, 64), (245, 245, 245)),
        accent_pressed: pair_rgb((23, 23, 23), (212, 212, 212)),
        accent_muted: pair_rgb((241, 241, 241), (38, 38, 38)),
        on_accent: pair_rgb((255, 255, 255), (23, 23, 23)),

        info: pair_rgb((0, 116, 226), (109, 156, 254)),
        on_info: pair_rgb((255, 255, 255), (23, 23, 23)),
        info_muted: Pair {
            light: Color::from_rgb8(196, 221, 251),
            dark: Color::from_rgba8(158, 183, 255, 0.24),
        },

        success: pair_rgb((0, 112, 4), (159, 229, 155)),
        warning: pair_rgb((116, 91, 0), (253, 207, 79)),
        error: pair_rgb((165, 12, 37), (255, 198, 193)),
        on_success: pair_rgb((255, 255, 255), (23, 23, 23)),
        on_warning: pair_rgb((23, 23, 23), (23, 23, 23)),
        on_error: pair_rgb((255, 255, 255), (23, 23, 23)),

        success_muted: Pair {
            light: Color::from_rgb8(197, 229, 192),
            dark: Color::from_rgba8(132, 201, 128, 0.24),
        },
        warning_muted: Pair {
            light: Color::from_rgb8(248, 218, 157),
            dark: Color::from_rgba8(222, 180, 51, 0.24),
        },
        error_muted: Pair {
            light: Color::from_rgb8(250, 206, 203),
            dark: Color::from_rgba8(255, 158, 151, 0.24),
        },

        badge_neutral_background: Pair {
            light: Color::from_rgb8(229, 229, 229),
            dark: Color::from_rgba8(255, 255, 255, 0.10),
        },
        badge_neutral_text: pair_rgb((38, 38, 38), (229, 229, 229)),
        badge_blue_background: Pair {
            light: Color::from_rgb8(196, 221, 251),
            dark: Color::from_rgba8(158, 183, 255, 0.24),
        },
        badge_blue_text: pair_rgb((0, 69, 140), (199, 211, 255)),
        badge_green_background: Pair {
            light: Color::from_rgb8(197, 229, 192),
            dark: Color::from_rgba8(132, 201, 128, 0.24),
        },
        badge_green_text: pair_rgb((12, 87, 0), (159, 229, 155)),
        badge_red_background: Pair {
            light: Color::from_rgb8(250, 206, 203),
            dark: Color::from_rgba8(255, 158, 151, 0.24),
        },
        badge_red_text: pair_rgb((137, 0, 26), (255, 198, 193)),
    },
    interaction_overlay: Pair {
        light: InteractionOverlay {
            hover: Color::from_rgba8(0, 0, 0, 0.06),
            pressed: Color::from_rgba8(0, 0, 0, 0.10),
        },
        dark: InteractionOverlay {
            hover: Color::from_rgba8(255, 255, 255, 0.08),
            pressed: Color::from_rgba8(255, 255, 255, 0.12),
        },
    },
    spacing: AppSpacing {
        s1: 4.0,
        s2: 8.0,
        s3: 12.0,
        s4: 16.0,
        s5: 20.0,
        s6: 24.0,
    },
    radius: AppRadius {
        none: 4.0,
        inner: 6.0,
        element: 10.0,
        container: 12.0,
        page: 28.0,
        full: 9999.0,
        progress: 6.0,
    },
    scrollbar: AppScrollbar {
        rail_width: 8.0,
        thumb_width: 6.0,
        content_gap: 12.0,
    },
    border_width: AppBorderWidth {
        hairline: 0.0,
        regular: 1.0,
    },
    shadow: AppShadow {
        none: Shadow {
            color: Color::TRANSPARENT,
            offset: Vector::new(0.0, 0.0),
            blur_radius: 0.0,
        },
        low: Pair {
            light: Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.10),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
            dark: Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.40),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 8.0,
            },
        },
        medium: Pair {
            light: Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.10),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            dark: Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.50),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
        },
        high: Pair {
            light: Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.18),
                offset: Vector::new(0.0, 18.0),
                blur_radius: 44.0,
            },
            dark: Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.70),
                offset: Vector::new(0.0, 18.0),
                blur_radius: 44.0,
            },
        },
    },
    typography: AppTypography {
        body: 14,
        label: 13,
        caption: 12,
        heading: 20,
        display: 30,
    },
};

pub(crate) const S1: f32 = TOKENS.spacing.s1;
pub(crate) const S2: f32 = TOKENS.spacing.s2;

pub(crate) fn mode_from_theme(theme: &Theme) -> Mode {
    if theme.extended_palette().is_dark {
        Mode::Dark
    } else {
        Mode::Light
    }
}

const fn pair_rgb(light: (u8, u8, u8), dark: (u8, u8, u8)) -> Pair<Color> {
    Pair {
        light: Color::from_rgb8(light.0, light.1, light.2),
        dark: Color::from_rgb8(dark.0, dark.1, dark.2),
    }
}

#[cfg(test)]
mod tests {
    use iced::Color;

    use super::{Mode, Pair, mode_from_theme};

    #[test]
    fn pair_get_returns_light_value_for_light_mode() {
        let pair = Pair {
            light: Color::WHITE,
            dark: Color::BLACK,
        };

        assert_eq!(pair.get(Mode::Light), Color::WHITE);
    }

    #[test]
    fn pair_get_returns_dark_value_for_dark_mode() {
        let pair = Pair {
            light: Color::WHITE,
            dark: Color::BLACK,
        };

        assert_eq!(pair.get(Mode::Dark), Color::BLACK);
    }

    #[test]
    fn mode_from_theme_reads_iced_light_theme_as_light() {
        assert_eq!(mode_from_theme(&iced::Theme::Light), Mode::Light);
    }

    #[test]
    fn mode_from_theme_reads_iced_dark_theme_as_dark() {
        assert_eq!(mode_from_theme(&iced::Theme::Dark), Mode::Dark);
    }
}
