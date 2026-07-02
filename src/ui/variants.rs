#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonVariant {
    Primary,
    Subtle,
    Selected,
    Icon,
    Danger,
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
    Accent,
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

#[expect(dead_code, reason = "reserved Astryx-style categorical vocabulary")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CategoricalTone {
    Red,
    Orange,
    Yellow,
    Green,
    Teal,
    Cyan,
    Blue,
    Purple,
    Pink,
    Gray,
}
