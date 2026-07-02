//! Reusable TreeList widget for hierarchical iced views.
//!
//! Parent screens own both the tree data and the interaction state:
//!
//! ```text
//! tree_items: Vec<TreeNode>
//! tree_state: TreeState
//! ```
//!
//! A parent update function handles messages like:
//!
//! ```text
//! Message::Tree(TreeMessage::Toggle(id)) => tree_state.toggle(id)
//! Message::Tree(TreeMessage::Select(id)) => tree_state.select(id)
//! ```
//!
//! Example data:
//!
//! ```text
//! Downloads
//!   Active
//!     ubuntu.iso
//!     archlinux.iso
//!   Waiting
//!     movie.mkv
//!   Completed
//!     book.pdf
//!     backup.zip
//! ```

mod style;
mod types;

use iced::widget::{button, column, container, row, space, text};
use iced::{Alignment, Element, Length};

pub use types::{TreeDensity, TreeMessage, TreeNode, TreeState};

use types::DensityMetrics;

use crate::ui::icons::{Icon, icon};

#[allow(dead_code, reason = "borrowed TreeList API for reusable screens")]
pub fn tree_list<'a>(
    items: &'a [TreeNode],
    state: &'a TreeState,
    density: TreeDensity,
) -> Element<'a, TreeMessage> {
    let metrics = density.metrics();
    let mut content = column![].spacing(metrics.item_spacing).width(Length::Fill);

    for item in items {
        content = content.push(render_node(item, state, 0, metrics));
    }

    content.into()
}

pub fn tree_list_owned(
    items: Vec<TreeNode>,
    state: TreeState,
    density: TreeDensity,
) -> Element<'static, TreeMessage> {
    let metrics = density.metrics();
    let mut content = column![].spacing(metrics.item_spacing).width(Length::Fill);

    for item in items {
        content = content.push(render_node_owned(item, &state, 0, metrics));
    }

    content.into()
}

#[allow(dead_code, reason = "used by the borrowed TreeList API")]
fn render_node<'a>(
    item: &'a TreeNode,
    state: &'a TreeState,
    depth: usize,
    metrics: DensityMetrics,
) -> Element<'a, TreeMessage> {
    let mut content = column![render_row(item, state, depth, metrics)]
        .spacing(metrics.item_spacing)
        .width(Length::Fill);

    if !item.children.is_empty() && state.is_expanded(&item.id) {
        for child in &item.children {
            content = content.push(render_node(child, state, depth + 1, metrics));
        }
    }

    content.into()
}

fn render_node_owned(
    item: TreeNode,
    state: &TreeState,
    depth: usize,
    metrics: DensityMetrics,
) -> Element<'static, TreeMessage> {
    let mut row_item = item;
    let is_expanded = state.is_expanded(&row_item.id);
    let child_count = row_item.children.len();
    let children = std::mem::take(&mut row_item.children);

    let mut content = column![render_row_owned(
        row_item,
        state,
        depth,
        metrics,
        child_count > 0,
    )]
    .spacing(metrics.item_spacing)
    .width(Length::Fill);

    if child_count > 0 && is_expanded {
        for child in children {
            content = content.push(render_node_owned(child, state, depth + 1, metrics));
        }
    }

    content.into()
}

#[allow(dead_code, reason = "used by the borrowed TreeList API")]
fn render_row<'a>(
    item: &'a TreeNode,
    state: &'a TreeState,
    depth: usize,
    metrics: DensityMetrics,
) -> Element<'a, TreeMessage> {
    let selected = state.is_selected(&item.id);
    let expanded = state.is_expanded(&item.id);
    let has_children = !item.children.is_empty();
    let disabled = item.disabled;
    let row_height = metrics.row_height(item.description.is_some());
    let mut main = row![
        branch_spacer(depth, metrics),
        chevron_icon(expanded, has_children, disabled, metrics),
    ]
    .spacing(metrics.row_spacing)
    .align_y(Alignment::Center);

    if let Some(icon) = item.icon.as_ref() {
        main = main.push(text(icon.as_str()).size(metrics.label_size));
    }

    main = main.push(label_block(item, metrics, disabled).width(Length::Fill));

    if let Some(end) = item.end.as_ref() {
        main = main.push(
            text(end.as_str())
                .size(metrics.description_size)
                .style(move |theme| style::secondary_text(theme, disabled)),
        );
    }

    let row_button = button(main)
        .width(Length::Fill)
        .height(Length::Fixed(row_height))
        .padding([0.0, metrics.row_padding_x])
        .style(move |theme, status| {
            style::tree_row_style(theme, status, selected, disabled, metrics)
        });
    let row_button = if disabled {
        row_button
    } else {
        row_button.on_press(row_press_message(&item.id, has_children))
    };

    container(row_button)
        .style(move |theme| style::row_container_style(theme, selected, metrics))
        .width(Length::Fill)
        .into()
}

fn render_row_owned(
    item: TreeNode,
    state: &TreeState,
    depth: usize,
    metrics: DensityMetrics,
    has_children: bool,
) -> Element<'static, TreeMessage> {
    let selected = state.is_selected(&item.id);
    let expanded = state.is_expanded(&item.id);
    let disabled = item.disabled;
    let id = item.id;
    let row_height = metrics.row_height(item.description.is_some());
    let mut main = row![
        branch_spacer(depth, metrics),
        chevron_icon(expanded, has_children, disabled, metrics),
    ]
    .spacing(metrics.row_spacing)
    .align_y(Alignment::Center);

    if let Some(icon) = item.icon {
        main = main.push(text(icon).size(metrics.label_size));
    }

    main = main.push(label_block_owned(
        item.label,
        item.description,
        metrics,
        disabled,
    ));

    if let Some(end) = item.end {
        main = main.push(
            text(end)
                .size(metrics.description_size)
                .style(move |theme| style::secondary_text(theme, disabled)),
        );
    }

    let row_button = button(main)
        .width(Length::Fill)
        .height(Length::Fixed(row_height))
        .padding([0.0, metrics.row_padding_x])
        .style(move |theme, status| {
            style::tree_row_style(theme, status, selected, disabled, metrics)
        });
    let row_button = if disabled {
        row_button
    } else {
        row_button.on_press(row_press_message(&id, has_children))
    };

    container(row_button)
        .style(move |theme| style::row_container_style(theme, selected, metrics))
        .width(Length::Fill)
        .into()
}

#[allow(dead_code, reason = "used by the borrowed TreeList API")]
fn label_block<'a>(
    item: &'a TreeNode,
    metrics: DensityMetrics,
    disabled: bool,
) -> iced::widget::Column<'a, TreeMessage> {
    let mut label = column![
        text(item.label.as_str())
            .size(metrics.label_size)
            .style(move |theme| style::primary_text(theme, disabled))
    ]
    .spacing(2)
    .width(Length::Fill);

    if let Some(description) = item.description.as_ref() {
        label = label.push(
            text(description.as_str())
                .size(metrics.description_size)
                .style(move |theme| style::secondary_text(theme, disabled)),
        );
    }

    label
}

fn label_block_owned(
    label: String,
    description: Option<String>,
    metrics: DensityMetrics,
    disabled: bool,
) -> iced::widget::Column<'static, TreeMessage> {
    let mut label_block = column![
        text(label)
            .size(metrics.label_size)
            .style(move |theme| style::primary_text(theme, disabled))
    ]
    .spacing(2)
    .width(Length::Fill);

    if let Some(description) = description {
        label_block = label_block.push(
            text(description)
                .size(metrics.description_size)
                .style(move |theme| style::secondary_text(theme, disabled)),
        );
    }

    label_block
}

fn chevron_icon(
    expanded: bool,
    has_children: bool,
    disabled: bool,
    metrics: DensityMetrics,
) -> Element<'static, TreeMessage> {
    if !has_children {
        return space::horizontal()
            .width(Length::Fixed(metrics.chevron_width))
            .into();
    }

    let icon_kind = if expanded {
        Icon::CaretDown
    } else {
        Icon::CaretRight
    };

    let icon_color = if disabled {
        style::disabled_chevron_icon_color
    } else {
        style::chevron_icon_color
    };

    container(icon(icon_kind, 14, icon_color))
        .width(Length::Fixed(metrics.chevron_width))
        .padding([4, 0])
        .into()
}

fn row_press_message(id: &str, has_children: bool) -> TreeMessage {
    if has_children {
        TreeMessage::Toggle(id.to_owned())
    } else {
        TreeMessage::Select(id.to_owned())
    }
}

fn branch_spacer<'a>(depth: usize, metrics: DensityMetrics) -> Element<'a, TreeMessage> {
    space::horizontal()
        .width(Length::Fixed(depth as f32 * metrics.indent_width))
        .into()
}

#[cfg(test)]
mod tests {
    use iced::Element;

    use super::{TreeDensity, TreeMessage, TreeNode, TreeState, row_press_message, tree_list};

    #[test]
    fn tree_list_renders_multiple_levels() {
        let mut root = TreeNode::branch(
            "root",
            "Root",
            vec![TreeNode::branch(
                "child",
                "Child",
                vec![TreeNode::leaf("leaf", "Leaf")],
            )],
        );
        root.initially_expanded = true;
        root.children[0].initially_expanded = true;
        let items = vec![root];
        let state = TreeState::from_items(&items);

        let _element: Element<'_, TreeMessage> = tree_list(&items, &state, TreeDensity::Balanced);
    }

    #[test]
    fn row_press_message_toggles_branch_nodes() {
        assert_eq!(
            row_press_message("folder:root", true),
            TreeMessage::Toggle("folder:root".to_owned())
        );
    }

    #[test]
    fn row_press_message_selects_leaf_nodes() {
        assert_eq!(
            row_press_message("file:root/file.txt", false),
            TreeMessage::Select("file:root/file.txt".to_owned())
        );
    }
}
