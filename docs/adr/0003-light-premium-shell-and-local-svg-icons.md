# Light premium shell and local SVG icons

Status: superseded by ADR-0004

Cottid uses a light, neutral, premium desktop shell for the MVP UI instead of a
toolbar-first utility layout.

The shell is organized as:

- Full-navigation sidebar for filters, counts, global commands, and connection
  context.
- Central compact card list with a contextual list header.
- Right detail drawer rendered only when a download is selected.
- Bottom status strip for speeds, refresh state, stale/error status, and counts.

The implementation uses Phosphor regular SVG icons through iced's existing
`svg` feature. Only the icons referenced by the typed icon component are copied
into the repository under `assets/icons/phosphor/regular`; runtime behavior must
not depend on `/home/hewel/PKG/phosphor-icons`.

This decision keeps the app visually richer than the first plain text layout
while preserving download-manager density and scanability. It also avoids
adding a new icon crate or UI dependency. Future UI work should extend the typed
icon component and curated asset set instead of loading arbitrary SVG paths from
view code.
