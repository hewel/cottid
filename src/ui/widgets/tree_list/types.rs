use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub end: Option<String>,
    pub children: Vec<TreeNode>,
    pub disabled: bool,
    pub initially_expanded: bool,
}

impl TreeNode {
    pub fn leaf(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            icon: None,
            end: None,
            children: Vec::new(),
            disabled: false,
            initially_expanded: false,
        }
    }

    pub fn branch(
        id: impl Into<String>,
        label: impl Into<String>,
        children: Vec<TreeNode>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            icon: None,
            end: None,
            children,
            disabled: false,
            initially_expanded: false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeState {
    pub expanded: HashSet<String>,
    pub selected: Option<String>,
}

impl TreeState {
    pub fn from_items(items: &[TreeNode]) -> Self {
        let mut state = Self::default();
        collect_initial_expanded(items, &mut state.expanded);
        state
    }

    pub fn toggle(&mut self, id: impl Into<String>) {
        let id = id.into();
        if !self.expanded.remove(&id) {
            self.expanded.insert(id);
        }
    }

    pub fn select(&mut self, id: impl Into<String>) {
        self.selected = Some(id.into());
    }

    pub fn is_expanded(&self, id: &str) -> bool {
        self.expanded.contains(id)
    }

    pub fn is_selected(&self, id: &str) -> bool {
        self.selected.as_deref() == Some(id)
    }
}

fn collect_initial_expanded(items: &[TreeNode], expanded: &mut HashSet<String>) {
    for item in items {
        if item.initially_expanded {
            expanded.insert(item.id.clone());
        }
        collect_initial_expanded(&item.children, expanded);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeMessage {
    Toggle(String),
    Select(String),
}

impl TreeMessage {
    pub fn id(&self) -> &str {
        match self {
            Self::Toggle(id) | Self::Select(id) => id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    dead_code,
    reason = "reusable TreeList supports caller-selected densities"
)]
pub enum TreeDensity {
    Compact,
    Balanced,
    Spacious,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct DensityMetrics {
    pub(crate) row_padding_y: f32,
    pub(crate) row_padding_x: f32,
    pub(crate) row_spacing: f32,
    pub(crate) item_spacing: f32,
    pub(crate) min_row_height: f32,
    pub(crate) indent_width: f32,
    pub(crate) chevron_width: f32,
    pub(crate) label_size: u32,
    pub(crate) description_size: u32,
}

impl DensityMetrics {
    pub(crate) fn row_height(self, has_description: bool) -> f32 {
        let text_height = self.label_size as f32
            + if has_description {
                2.0 + self.description_size as f32
            } else {
                0.0
            };

        (text_height + self.row_padding_y * 2.0).max(self.min_row_height)
    }
}

impl TreeDensity {
    pub(crate) fn metrics(self) -> DensityMetrics {
        match self {
            Self::Compact => DensityMetrics {
                row_padding_y: 4.0,
                row_padding_x: 6.0,
                row_spacing: 4.0,
                item_spacing: 2.0,
                min_row_height: 28.0,
                indent_width: 22.0,
                chevron_width: 24.0,
                label_size: 12,
                description_size: 11,
            },
            Self::Balanced => DensityMetrics {
                row_padding_y: 6.0,
                row_padding_x: 8.0,
                row_spacing: 6.0,
                item_spacing: 4.0,
                min_row_height: 34.0,
                indent_width: 24.0,
                chevron_width: 26.0,
                label_size: 13,
                description_size: 12,
            },
            Self::Spacious => DensityMetrics {
                row_padding_y: 9.0,
                row_padding_x: 10.0,
                row_spacing: 8.0,
                item_spacing: 6.0,
                min_row_height: 42.0,
                indent_width: 28.0,
                chevron_width: 28.0,
                label_size: 14,
                description_size: 12,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TreeDensity, TreeNode, TreeState};

    #[test]
    fn tree_state_from_items_collects_initial_expansion_recursively() {
        let mut child = TreeNode::leaf("child", "Child");
        child.initially_expanded = true;
        let items = vec![TreeNode::branch("root", "Root", vec![child])];

        let state = TreeState::from_items(&items);

        assert!(state.is_expanded("child"));
    }

    #[test]
    fn tree_state_toggle_adds_and_removes_expansion() {
        let mut state = TreeState::default();

        state.toggle("root");
        assert!(state.is_expanded("root"));

        state.toggle("root");
        assert!(!state.is_expanded("root"));
    }

    #[test]
    fn tree_state_select_replaces_selected_item() {
        let mut state = TreeState::default();

        state.select("one");
        state.select("two");

        assert!(state.is_selected("two"));
    }

    #[test]
    fn density_metrics_grow_from_compact_to_spacious() {
        let compact = TreeDensity::Compact.metrics();
        let spacious = TreeDensity::Spacious.metrics();

        assert!(compact.row_padding_y < spacious.row_padding_y);
    }
}
