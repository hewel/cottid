# Rust design tokens and component wrappers

Status: superseded by ADR-0006

Cottid's UI theme is organized as Rust design tokens, variant enums, `iced`
style adapters, and UI-internal component wrappers.

This keeps styling decisions separate from view composition:

- `src/ui/tokens.rs` owns semantic color and radius values.
- `src/ui/variants.rs` names the supported surface, button, text, input,
  progress, and feedback variants.
- `src/ui/theme.rs` converts variants into `iced` style functions.
- `src/ui/components.rs` provides repeated app controls such as cards, inputs,
  buttons, feedback banners, and detail rows.

The structure is inspired by token-based CSS theme systems, including Astryx's
neutral theme, but it is not a direct port. The Rust API uses semantic names
that fit Cottid's UI and only exposes variants currently used by the app.

Card shadows are intentionally avoided. Depth is expressed through spacing,
border contrast, selected borders, and subtle dark-mode inset-like contrast
inside framed overlays. Future theme changes should update tokens and variants
before editing individual view modules.
