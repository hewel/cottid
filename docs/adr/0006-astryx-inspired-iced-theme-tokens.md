# Astryx-inspired iced theme tokens

Status: accepted

Cottid's theme architecture uses Astryx neutral as a design-token reference, not
as a CSS runtime or a CSS-to-iced conversion target. Theme code stays inside the
existing `src/ui` boundary:

- `src/ui/tokens.rs` owns semantic light/dark token pairs.
- `src/ui/variants.rs` owns Rust enums for supported component variants.
- `src/ui/widgets/*` adapts tokens and variants into `iced` widget styles.
- `src/ui/theme.rs` maps only the basic `iced::theme::Palette` values and keeps
  compatibility helpers for existing views.

This supersedes ADR-0005 by making the Astryx relationship explicit and by
adding a dedicated widget style adapter layer. The app still avoids CSS
semantics that do not exist in `iced`: cascade, selectors, inheritance, runtime
CSS variables, and full browser layout behavior.

Light and dark mode are derived from the resolved `iced::Theme` inside style
closures. That keeps the System theme preference aligned with iced's platform
theme handling instead of duplicating OS appearance state in app state.

Primary actions remain neutral, following Astryx neutral's black/white accent
spine. Vivid blue is reserved for information and data accents such as progress
bars and categorical badges. Badge variants distinguish filled semantic badges
from tinted categorical badges.

Shadows are intentionally minimal. `iced` exposes one `Shadow` per widget style,
while Astryx shadows can contain multiple layers and inset highlights. Cottid
approximates this with single low/medium/high shadows; modal surfaces use the
high shadow and no border, while cards remain mostly flat and rely on borders,
spacing, and contrast.

Astryx `Dialog` remains a visual reference, not a direct behavior port. Cottid
uses the modal surface, radius, high shadow, 400px default dialog width, 90%
viewport max width, 75% viewport max height, and overlay color as guidance. It
does not port browser-only mechanics such as native `<dialog>`, CSS
`::backdrop`, backdrop blur, static dialog positioning, fullscreen variants, or
keyframe enter animation.

Future theme work should add tokens and variants first, then route widget
styling through `src/ui/widgets`. View modules should continue using UI wrapper
helpers instead of constructing colors and borders directly.
