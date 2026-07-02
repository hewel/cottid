#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonVariant {
    Primary,
    Secondary,
    Destructive,
    Ghost,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BadgeVariant {
    Neutral,
    Success,
    Warning,
    Error,
    Blue,
    Green,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SurfaceVariant {
    App,
    Sidebar,
    Card,
    SelectedCard,
    Muted,
    Search,
    Modal,
    Feedback(FeedbackVariant),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextVariant {
    Primary,
    Muted,
    Info,
    Danger,
    Warning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FeedbackVariant {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InputVariant {
    Form,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProgressVariant {
    Accent,
}
