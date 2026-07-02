# Iced-native tooltip and popover overlays

Status: accepted

Cottid uses iced-native overlay primitives for Astryx-inspired tooltip and
popover behavior instead of porting React, DOM refs, CSS anchor positioning, or
the browser Popover API.

Tooltip is a non-interactive hover hint built with iced's built-in `tooltip`
widget. It shares Cottid's placement vocabulary, maps that vocabulary to iced's
tooltip positions, uses the central overlay surface style, and relies on iced
for delay and viewport snapping.

Popover is an interactive floating layer built as a custom iced widget and
overlay. The trigger remains in normal layout, while open content is laid out in
overlay space relative to the trigger bounds. The first version supports one
open app popover at a time, Escape dismissal, outside-click dismissal, and
interactive content. It intentionally does not implement focus traps, focus
restoration, animations, nested popovers, browser accessibility attributes, or
fallback placement flipping.

This requires enabling iced's `advanced` feature because the widget and overlay
traits are gated there. No third-party dependency is added. The feature is used
only inside the UI overlay module; RPC, scheduler, and domain code remain
unaware of iced widgets beyond the existing UI boundary.

The shared `position_layer` function stays widget-independent so placement,
alignment, gap, viewport snapping, and future flip fallback can be tested
without rendering the app.
