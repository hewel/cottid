use iced::Color;

pub(crate) fn overlay(base: Color, over: Color) -> Color {
    let a = over.a + base.a * (1.0 - over.a);

    if a <= f32::EPSILON {
        return Color::TRANSPARENT;
    }

    Color {
        r: (over.r * over.a + base.r * base.a * (1.0 - over.a)) / a,
        g: (over.g * over.a + base.g * base.a * (1.0 - over.a)) / a,
        b: (over.b * over.a + base.b * base.a * (1.0 - over.a)) / a,
        a,
    }
}

#[cfg(test)]
mod tests {
    use iced::Color;

    #[test]
    fn overlay_returns_base_when_overlay_is_transparent() {
        let base = Color::from_rgb8(38, 38, 38);

        assert_eq!(super::overlay(base, Color::TRANSPARENT), base);
    }

    #[test]
    fn overlay_returns_overlay_when_overlay_is_opaque() {
        let over = Color::from_rgb8(255, 255, 255);

        assert_eq!(super::overlay(Color::BLACK, over), over);
    }

    #[test]
    fn overlay_lightens_black_when_overlay_is_translucent_white() {
        let blended = super::overlay(Color::BLACK, Color::from_rgba8(255, 255, 255, 0.50));

        assert!(blended.r > Color::BLACK.r);
        assert!(blended.g > Color::BLACK.g);
        assert!(blended.b > Color::BLACK.b);
    }

    #[test]
    fn overlay_darkens_white_when_overlay_is_translucent_black() {
        let blended = super::overlay(Color::WHITE, Color::from_rgba8(0, 0, 0, 0.50));

        assert!(blended.r < Color::WHITE.r);
        assert!(blended.g < Color::WHITE.g);
        assert!(blended.b < Color::WHITE.b);
    }
}
