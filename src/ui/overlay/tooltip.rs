use std::time::Duration;

use iced::Element;
use iced::widget::{container, text, tooltip};

use crate::app::Message;
use crate::ui::overlay::Placement;
use crate::ui::overlay::style;
use crate::ui::tokens::TOKENS;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TooltipOptions {
    pub(crate) placement: Placement,
    pub(crate) delay: Duration,
    pub(crate) enabled: bool,
    pub(crate) max_width: f32,
}

impl Default for TooltipOptions {
    fn default() -> Self {
        Self {
            placement: Placement::Above,
            delay: Duration::from_millis(200),
            enabled: true,
            max_width: 300.0,
        }
    }
}

pub(crate) fn app_tooltip<'a>(
    trigger: impl Into<Element<'a, Message>>,
    content: impl Into<String>,
    options: TooltipOptions,
) -> Element<'a, Message> {
    let trigger = trigger.into();

    if !options.enabled {
        return trigger;
    }

    let tooltip_content = container(text(content.into()).size(TOKENS.typography.caption))
        .max_width(options.max_width);

    tooltip(
        trigger,
        tooltip_content,
        tooltip_position(options.placement),
    )
    .delay(options.delay)
    .gap(6.0)
    .padding(8.0)
    .snap_within_viewport(true)
    .style(style::tooltip_surface)
    .into()
}

fn tooltip_position(placement: Placement) -> tooltip::Position {
    match placement {
        Placement::Above => tooltip::Position::Top,
        Placement::Below => tooltip::Position::Bottom,
        Placement::Start => tooltip::Position::Left,
        Placement::End => tooltip::Position::Right,
    }
}
