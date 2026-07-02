use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer, widget};
use iced::keyboard::{Key, key};
use iced::widget::{container, opaque};
use iced::{Element, Event, Length, Rectangle, Size, Theme, Vector};

use crate::app::Message;
use crate::ui::overlay::positioning::position_layer;
use crate::ui::overlay::style;
use crate::ui::overlay::{Alignment, Placement};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PopoverId(pub u64);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PopoverState {
    open: Option<PopoverId>,
}

impl PopoverState {
    pub(crate) fn is_open(&self, id: PopoverId) -> bool {
        self.open == Some(id)
    }

    pub(crate) fn toggle(&mut self, id: PopoverId) {
        if self.is_open(id) {
            self.close();
        } else {
            self.open(id);
        }
    }

    pub(crate) fn open(&mut self, id: PopoverId) {
        self.open = Some(id);
    }

    pub(crate) fn close(&mut self) {
        self.open = None;
    }
}

#[allow(
    dead_code,
    reason = "retained reusable popover API after removing sidebar demo"
)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct PopoverOptions {
    pub(crate) placement: Placement,
    pub(crate) alignment: Alignment,
    pub(crate) gap: f32,
    pub(crate) width: Option<f32>,
    pub(crate) match_trigger_width: bool,
    pub(crate) close_on_escape: bool,
    pub(crate) close_on_outside_click: bool,
    pub(crate) snap_to_viewport: bool,
}

impl Default for PopoverOptions {
    fn default() -> Self {
        Self {
            placement: Placement::Below,
            alignment: Alignment::Start,
            gap: 8.0,
            width: None,
            match_trigger_width: false,
            close_on_escape: true,
            close_on_outside_click: true,
            snap_to_viewport: true,
        }
    }
}

#[allow(
    dead_code,
    reason = "retained reusable popover API after removing sidebar demo"
)]
pub(crate) fn app_popover<'a>(
    id: PopoverId,
    trigger: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
    is_open: bool,
    options: PopoverOptions,
    on_dismiss: Message,
) -> Element<'a, Message> {
    let content = opaque(
        container(content)
            .padding(8)
            .width(popover_width(options))
            .style(style::popover_surface),
    );

    Popover {
        _id: id,
        trigger: trigger.into(),
        content: content.into(),
        is_open,
        options,
        on_dismiss,
    }
    .into()
}

#[allow(dead_code, reason = "used by retained popover API")]
fn popover_width(options: PopoverOptions) -> Length {
    options.width.map_or(Length::Shrink, Length::Fixed)
}

struct Popover<'a> {
    _id: PopoverId,
    trigger: Element<'a, Message>,
    content: Element<'a, Message>,
    is_open: bool,
    options: PopoverOptions,
    on_dismiss: Message,
}

impl Widget<Message, Theme, iced::Renderer> for Popover<'_> {
    fn children(&self) -> Vec<widget::Tree> {
        vec![
            widget::Tree::new(&self.trigger),
            widget::Tree::new(&self.content),
        ]
    }

    fn diff(&self, tree: &mut widget::Tree) {
        tree.diff_children(&[self.trigger.as_widget(), self.content.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        self.trigger.as_widget().size()
    }

    fn size_hint(&self) -> Size<Length> {
        self.trigger.as_widget().size_hint()
    }

    fn layout(
        &mut self,
        tree: &mut widget::Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.trigger
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn update(
        &mut self,
        tree: &mut widget::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.trigger.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.trigger.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.trigger.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut widget::Tree,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, iced::Renderer>> {
        let mut children = tree.children.iter_mut();
        let trigger_tree = children.next().expect("popover trigger tree");
        let content_tree = children.next().expect("popover content tree");

        let trigger_overlay = self.trigger.as_widget_mut().overlay(
            trigger_tree,
            layout,
            renderer,
            viewport,
            translation,
        );

        let popover_overlay = self.is_open.then(|| {
            overlay::Element::new(Box::new(PopoverOverlay {
                content: &mut self.content,
                tree: content_tree,
                anchor_bounds: layout.bounds() + translation,
                viewport_bounds: *viewport,
                options: self.options,
                on_dismiss: self.on_dismiss.clone(),
            }))
        });

        if trigger_overlay.is_some() || popover_overlay.is_some() {
            Some(
                overlay::Group::with_children(
                    trigger_overlay.into_iter().chain(popover_overlay).collect(),
                )
                .overlay(),
            )
        } else {
            None
        }
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.trigger
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
}

impl<'a> From<Popover<'a>> for Element<'a, Message> {
    fn from(popover: Popover<'a>) -> Self {
        Element::new(popover)
    }
}

struct PopoverOverlay<'a, 'b> {
    content: &'b mut Element<'a, Message>,
    tree: &'b mut widget::Tree,
    anchor_bounds: Rectangle,
    viewport_bounds: Rectangle,
    options: PopoverOptions,
    on_dismiss: Message,
}

impl overlay::Overlay<Message, Theme, iced::Renderer> for PopoverOverlay<'_, '_> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let viewport = viewport_for_layout(self.viewport_bounds, bounds);
        let mut limits = layout::Limits::new(
            Size::ZERO,
            if self.options.snap_to_viewport {
                viewport.size()
            } else {
                Size::INFINITE
            },
        );

        if let Some(width) = self.options.width {
            limits = limits.width(Length::Fixed(width));
        } else if self.options.match_trigger_width {
            limits = limits.width(Length::Fixed(self.anchor_bounds.width));
        } else {
            limits = limits.width(Length::Shrink);
        }

        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let position = position_layer(
            self.anchor_bounds,
            node.size(),
            viewport,
            self.options.placement,
            self.options.alignment,
            self.options.gap,
            self.options.snap_to_viewport,
        );

        node.move_to(position)
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        if self.options.close_on_escape && is_escape_press(event) {
            shell.publish(self.on_dismiss.clone());
            shell.capture_event();
            return;
        }

        let overlay_bounds = layout.bounds();

        if self.options.close_on_outside_click
            && is_primary_pointer_press(event)
            && !cursor.is_over(overlay_bounds)
            && !cursor
                .position()
                .is_some_and(|position| self.anchor_bounds.contains(position))
        {
            shell.publish(self.on_dismiss.clone());
            shell.capture_event();
            return;
        }

        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &overlay_bounds,
        );
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let bounds = layout.bounds();
        self.content
            .as_widget()
            .draw(self.tree, renderer, theme, style, layout, cursor, &bounds);
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }

    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
    ) -> Option<overlay::Element<'a, Message, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            self.tree,
            layout,
            renderer,
            &layout.bounds(),
            Vector::ZERO,
        )
    }
}

fn viewport_for_layout(viewport: Rectangle, bounds: Size) -> Rectangle {
    if viewport.width > 0.0 && viewport.height > 0.0 {
        viewport
    } else {
        Rectangle::with_size(bounds)
    }
}

fn is_escape_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: Key::Named(key::Named::Escape),
            ..
        })
    )
}

fn is_primary_pointer_press(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(iced::mouse::Event::ButtonPressed(iced::mouse::Button::Left))
            | Event::Touch(iced::touch::Event::FingerPressed { .. })
    )
}

#[cfg(test)]
mod tests {
    use super::{PopoverId, PopoverState};

    #[test]
    fn popover_state_opens_requested_id() {
        let mut state = PopoverState::default();

        state.open(PopoverId(1));

        assert!(state.is_open(PopoverId(1)));
    }

    #[test]
    fn popover_state_allows_only_one_open_popover() {
        let mut state = PopoverState::default();

        state.open(PopoverId(1));
        state.open(PopoverId(2));

        assert!(!state.is_open(PopoverId(1)));
        assert!(state.is_open(PopoverId(2)));
    }

    #[test]
    fn popover_state_toggle_closes_open_id() {
        let mut state = PopoverState::default();

        state.open(PopoverId(1));
        state.toggle(PopoverId(1));

        assert!(!state.is_open(PopoverId(1)));
    }

    #[test]
    fn popover_state_close_clears_open_id() {
        let mut state = PopoverState::default();

        state.open(PopoverId(1));
        state.close();

        assert!(!state.is_open(PopoverId(1)));
    }
}
