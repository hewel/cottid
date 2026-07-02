#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Placement {
    Above,
    Below,
    Start,
    End,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Alignment {
    Start,
    Center,
    End,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct LayerOptions {
    pub(crate) placement: Placement,
    pub(crate) alignment: Alignment,
    pub(crate) gap: f32,
    pub(crate) snap_to_viewport: bool,
}

impl Default for LayerOptions {
    fn default() -> Self {
        Self {
            placement: Placement::Below,
            alignment: Alignment::Center,
            gap: 8.0,
            snap_to_viewport: true,
        }
    }
}
