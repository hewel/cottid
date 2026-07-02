use iced::{Point, Rectangle, Size};

use crate::ui::overlay::{Alignment, Placement};

pub(crate) fn position_layer(
    anchor_bounds: Rectangle,
    popup_size: Size,
    viewport_bounds: Rectangle,
    placement: Placement,
    alignment: Alignment,
    gap: f32,
    snap_to_viewport: bool,
) -> Point {
    let point = unclamped_position(anchor_bounds, popup_size, placement, alignment, gap);

    if snap_to_viewport {
        Point::new(
            clamp_axis(
                point.x,
                popup_size.width,
                viewport_bounds.x,
                viewport_bounds.width,
            ),
            clamp_axis(
                point.y,
                popup_size.height,
                viewport_bounds.y,
                viewport_bounds.height,
            ),
        )
    } else {
        point
    }
}

fn unclamped_position(
    anchor_bounds: Rectangle,
    popup_size: Size,
    placement: Placement,
    alignment: Alignment,
    gap: f32,
) -> Point {
    match placement {
        Placement::Above => Point::new(
            aligned_start(
                anchor_bounds.x,
                anchor_bounds.width,
                popup_size.width,
                alignment,
            ),
            anchor_bounds.y - popup_size.height - gap,
        ),
        Placement::Below => Point::new(
            aligned_start(
                anchor_bounds.x,
                anchor_bounds.width,
                popup_size.width,
                alignment,
            ),
            anchor_bounds.y + anchor_bounds.height + gap,
        ),
        Placement::Start => Point::new(
            anchor_bounds.x - popup_size.width - gap,
            aligned_start(
                anchor_bounds.y,
                anchor_bounds.height,
                popup_size.height,
                alignment,
            ),
        ),
        Placement::End => Point::new(
            anchor_bounds.x + anchor_bounds.width + gap,
            aligned_start(
                anchor_bounds.y,
                anchor_bounds.height,
                popup_size.height,
                alignment,
            ),
        ),
    }
}

fn aligned_start(
    anchor_start: f32,
    anchor_size: f32,
    popup_size: f32,
    alignment: Alignment,
) -> f32 {
    match alignment {
        Alignment::Start => anchor_start,
        Alignment::Center => anchor_start + (anchor_size - popup_size) / 2.0,
        Alignment::End => anchor_start + anchor_size - popup_size,
    }
}

fn clamp_axis(position: f32, popup_size: f32, viewport_start: f32, viewport_size: f32) -> f32 {
    let viewport_end = viewport_start + viewport_size;
    let max_position = viewport_end - popup_size;

    if max_position < viewport_start {
        viewport_start
    } else {
        position.clamp(viewport_start, max_position)
    }
}

#[cfg(test)]
mod tests {
    use iced::{Point, Rectangle, Size};

    use crate::ui::overlay::{Alignment, Placement};

    fn anchor() -> Rectangle {
        Rectangle {
            x: 100.0,
            y: 100.0,
            width: 40.0,
            height: 20.0,
        }
    }

    fn popup() -> Size {
        Size::new(20.0, 10.0)
    }

    fn viewport() -> Rectangle {
        Rectangle {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 300.0,
        }
    }

    #[test]
    fn position_layer_places_popup_above_anchor() {
        let point = super::position_layer(
            anchor(),
            popup(),
            viewport(),
            Placement::Above,
            Alignment::Center,
            8.0,
            true,
        );

        assert_eq!(point, Point::new(110.0, 82.0));
    }

    #[test]
    fn position_layer_places_popup_below_anchor() {
        let point = super::position_layer(
            anchor(),
            popup(),
            viewport(),
            Placement::Below,
            Alignment::Center,
            8.0,
            true,
        );

        assert_eq!(point, Point::new(110.0, 128.0));
    }

    #[test]
    fn position_layer_places_popup_at_start_side() {
        let point = super::position_layer(
            anchor(),
            popup(),
            viewport(),
            Placement::Start,
            Alignment::Center,
            8.0,
            true,
        );

        assert_eq!(point, Point::new(72.0, 105.0));
    }

    #[test]
    fn position_layer_places_popup_at_end_side() {
        let point = super::position_layer(
            anchor(),
            popup(),
            viewport(),
            Placement::End,
            Alignment::Center,
            8.0,
            true,
        );

        assert_eq!(point, Point::new(148.0, 105.0));
    }

    #[test]
    fn position_layer_aligns_popup_to_anchor_start() {
        let point = super::position_layer(
            anchor(),
            popup(),
            viewport(),
            Placement::Below,
            Alignment::Start,
            8.0,
            true,
        );

        assert_eq!(point.x, 100.0);
    }

    #[test]
    fn position_layer_aligns_popup_to_anchor_end() {
        let point = super::position_layer(
            anchor(),
            popup(),
            viewport(),
            Placement::Below,
            Alignment::End,
            8.0,
            true,
        );

        assert_eq!(point.x, 120.0);
    }

    #[test]
    fn position_layer_clamps_popup_inside_viewport() {
        let point = super::position_layer(
            Rectangle {
                x: 290.0,
                y: 290.0,
                width: 10.0,
                height: 10.0,
            },
            Size::new(40.0, 30.0),
            viewport(),
            Placement::Below,
            Alignment::Start,
            8.0,
            true,
        );

        assert_eq!(point, Point::new(260.0, 270.0));
    }

    #[test]
    fn position_layer_can_leave_popup_unclamped() {
        let point = super::position_layer(
            anchor(),
            popup(),
            viewport(),
            Placement::Above,
            Alignment::Center,
            200.0,
            false,
        );

        assert_eq!(point.y, -110.0);
    }

    #[test]
    fn position_layer_places_oversized_popup_at_viewport_start_when_clamping() {
        let point = super::position_layer(
            anchor(),
            Size::new(400.0, 400.0),
            viewport(),
            Placement::Below,
            Alignment::Start,
            8.0,
            true,
        );

        assert_eq!(point, Point::new(0.0, 0.0));
    }
}
