# Cottid Iced UI Structure

This document defines the planned MVP UI composition for Cottid. The UI is a
desktop-oriented `iced` frontend for an external aria2 daemon. It consumes
domain models only and emits messages; it must not call the RPC client directly
or depend on raw aria2 JSON-RPC DTOs.

## Composition Rules

- Keep views small and domain-oriented.
- Avoid one giant root view function.
- Root view composes child views and maps child messages into app messages.
- Child views receive domain/view models and emit messages only.
- UI modules do not create tasks, perform IO, call RPC, persist config, or parse
  raw aria2 data.
- No extra UI dependency may be added without explicit approval.

## Theme Architecture

The UI theme is layered so visual choices can change without rewriting view
logic:

- `src/ui/tokens.rs` defines semantic Rust design tokens.
- `src/ui/variants.rs` defines the small set of button, surface, text, input,
  progress, and feedback variants currently used by the app.
- `src/ui/theme.rs` adapts those tokens and variants into `iced` style
  functions.
- `src/ui/components.rs` exposes UI-internal wrappers for repeated controls and
  surfaces.

View modules should prefer component wrappers for repeated app controls such as
cards, feedback banners, text buttons, icon buttons, and form inputs. Direct
`iced` widgets remain fine for one-off layout composition.

For visual constants, prefer existing design tokens and semantic wrappers over
raw numeric literals. Use `TOKENS.spacing`, `TOKENS.radius`,
`TOKENS.typography`, semantic variants, and component helpers when expressing
spacing, sizing, border widths, colors, shadows, and text sizes. Raw numbers are
acceptable for external constraints, protocol/API values, or one-off layout
limits with no matching token. If a visual value is reused, add it to
`src/ui/tokens.rs` instead of scattering literals through view code.

The token names are semantic, not CSS names. External CSS theme systems can
inform the palette and variant vocabulary, but Rust views should depend on the
local tokens and variants only.

## Root Layout

Use a stable desktop shell:

- Full-navigation left sidebar.
- Main content row.
- Bottom status strip.
- Overlay layer for add/settings/error dialogs.

The main content row contains:

- A left sidebar with filters, counts, global commands, and connection context.
- A central compact card list with a contextual list header.
- A right-side detail drawer only when a download is selected.

Offline, empty, loading, and stale states should appear inside the main content
area instead of forcing separate app layouts.

When the window is narrow, the sidebar collapses first. If a detail is selected
in compact layout, the detail view replaces the list and exposes a clear/back
control.

## Sidebar

MVP content:

- App title or active endpoint label.
- Connection indicator.
- Status filters with counts.
- Add URI/magnet action.
- Manual refresh action.
- Purge stopped action.
- Settings action.

Sidebar actions should be disabled or visually softened when disconnected or
when a relevant operation is already pending.

Future sidebar space:

- Profile switcher.
- Speed profile selector.
- Queue controls.
- Daemon start/stop controls.
- Tray-related status affordances.

Filters:

- All.
- Active.
- Waiting.
- Paused.
- Stopped or complete.
- Error.

Filter selection belongs in `UiState`. Counts come from `DownloadsState`.

Do not include queue management, profile management, or torrent-specific
navigation in MVP.

## List Header

The central list header shows only contextual list information:

- Current filter.
- Active/waiting/stopped counts.
- Refresh state.

Do not add search or sorting in this slice.

## Download List

The download list consumes ordered `DownloadItem` summaries from
`DownloadsState`.

MVP behavior:

- Show active, waiting, and stopped downloads according to the selected filter.
- Preserve the last known good list during refresh failures.
- Show loading, empty, stale, and filtered-empty states.
- Support single task selection by `Gid`.
- Emit list-level and row-level messages only.

The list must not:

- Import raw RPC DTOs.
- Display raw JSON.
- Trigger RPC calls directly.
- Own connection or persistence state.

## Task Card

Each task card displays one `DownloadItem` summary.

MVP fields:

- Curated file-type icon.
- Display name or path summary.
- Status.
- Thin progress bar.
- Percent/completed/total progress text.
- Completed and total size.
- Download speed.
- Upload speed when relevant.
- Basic error summary.
- Per-row pending/error operation state.

MVP row actions:

- Select.
- Pause.
- Unpause.
- Remove.

Task cards should emit messages with `Gid`; parent update logic decides whether
to run RPC tasks. Do not expose raw aria2 status strings or raw RPC errors in
the card.

The first file-type icon mapping is intentionally small:

- Archive.
- Video.
- Audio.
- Image.
- Document.
- Executable.
- Torrent or magnet.
- Generic file fallback.

Future card additions:

- Queue position.
- Seeding indicator.
- Torrent/Metalink badges.
- Speed limit profile marker.

## Add Download Dialog

The add dialog is a modal overlay. Cottid modals render as one active,
centered floating layer above the shell with an Astryx-inspired surface,
high-elevation shadow, blocking backdrop, 400px target width, 90% viewport max
width, and 75% viewport max height. Overflow scrolls inside the modal. Backdrop
clicks dismiss the active modal through the same cancel path as Escape.

MVP fields and behavior:

- URI or magnet input.
- Minimal validation feedback.
- Submit action.
- Cancel action.
- Pending/error state while submit is in progress.

The MVP accepts one URI or magnet per submission. Do not include torrent file
picker, Metalink picker, batch input, or advanced aria2 options yet.

## Detail Panel

The detail panel shows information for the selected download. It renders only
when a download is selected.

MVP content:

- GID.
- Status.
- Directory or path summary.
- Progress.
- Download/upload speed.
- Completed, uploaded, and total bytes.
- File summaries.
- Basic error details.
- Operational metadata from selected `tellStatus`, such as directory,
  connections, piece length, piece count, and aria2 error code/message.
- Display-only torrent metadata when aria2 returns it, such as info hash,
  seeding flag, and seeder count.

When nothing is selected, the right detail drawer is not rendered. When the
selected download disappears, clear or update the selection through
`DownloadsState` logic.

Future detail sections:

- Peer list.
- Trackers.
- Metalink source details.
- Daemon/request diagnostics.

These future sections should be backed by typed domain models, not raw aria2
payloads.

## Settings Page

Use a modal opened from the sidebar. It uses the same single active,
Astryx-inspired centered floating modal layer as Add Download, but keeps a
640px target width for form ergonomics. Opening settings closes an idle add
dialog, but a pending add submission keeps the add dialog active.

MVP settings:

- RPC endpoint URL.
- Optional secret input. Entering a secret enables token authentication; clearing
  it disables authentication and clears the stored token.
- Polling interval when supported.
- Test connection action.
- Apply/save action.
- Cancel action.

Settings use draft state separate from applied connection settings. Secret
values must not appear in logs, error text, status bar text, or debug-style UI.
Successful connection tests from the settings panel apply and save the tested
settings while leaving the panel open with confirmation. If secure token storage
fails, show inline choices for plaintext fallback or session-only token use.

Do not add multi-profile management, daemon settings, or database-backed
settings in MVP.

## Status Bar

The bottom status strip remains visible across the main shell.

MVP content:

- Connection state.
- Last refresh state.
- Stale-data indicator.
- Global active/waiting/stopped counts.
- Concise current error or operation status.

The status bar should be display-safe and must never render secrets.

## Error And Modal Strategy

Use the least disruptive error surface that fits the problem.

- Connection and refresh failures: inline banner or status bar detail.
- Per-download command failures: attach to the affected row when possible.
- Add/settings validation failures: inline inside the dialog or settings page.
- Destructive remove: use confirmation only when the action scope is not obvious.
- Toasts: optional future polish; do not add a toast dependency for MVP.

Refresh failure should keep the last known good download snapshot visible and
mark it stale.

## State And Message Boundaries

UI state:

- Active panel.
- Selected filter.
- Active modal identity.
- Draft input text.
- Sort/group preference.
- Focus/request flags.

Domain/app state:

- Downloads and global stats live in `DownloadsState`.
- Connection lifecycle lives in `ConnectionState`.
- Editable configuration lives in `SettingsState`.
- Selected download identity lives with downloads state and uses `Gid`.

Message groups should mirror UI domains:

- Toolbar messages.
- Filter/sidebar messages.
- Download list messages.
- Task row messages.
- Add dialog messages.
- Detail panel messages.
- Settings messages.
- Error/status messages.

Root update logic maps these into domain handlers and tasks. Child UI modules
never create tasks.

## MVP Behavior Checklist

- Launch shows offline or settings-ready state.
- Connection state is visible.
- Global download/upload speed is visible when known.
- Active, waiting, paused, stopped/complete, and error downloads can be viewed.
- Each task shows progress, speed, status, size, and basic error information.
- User can add a URI or magnet.
- User can pause, unpause, remove, and purge stopped results.
- User can select a task and inspect details.
- User can edit RPC endpoint and secret.
- UI remains useful when aria2 is unreachable or refresh fails.

## Future UI Space

The current UI uses a light, neutral premium desktop palette with blue primary
actions and standard warning/error colors. Phosphor regular SVG icons are kept
as a curated repo-local asset set and rendered through a typed icon component.

Future WebSocket live updates should feed the same UI state as polling. The UI
should not care whether updates came from polling or notifications.

Future torrent and Metalink support should add typed detail sections without
turning task rows into advanced torrent dashboards.

Future queue management can add move/reorder controls to the list and toolbar.
It should not change the basic `DownloadItem` row contract.

Future multiple-profile support can add a profile switcher near the connection
indicator and settings page. MVP remains single-endpoint.

Future managed daemon logs should live in a separate daemon/log panel, not in
the download detail panel.

Future system tray and speed profile features are shell/integration features and
require dependency approval before implementation.

## Test And QA Targets

Implementation should test:

- Filter state changes.
- Task selection by `Gid`.
- Dialog open/close behavior.
- Add dialog validation messages.
- Settings draft/apply/cancel behavior.
- Action enablement for connected/disconnected and selected/unselected states.
- View-facing APIs accept domain/view models only.

Manual QA should cover:

- Offline launch.
- Successful connection.
- Failed connection.
- Stale refresh while preserving the list.
- Empty list.
- Populated active/waiting/stopped lists.
- Add URI and magnet.
- Pause, unpause, remove, purge.
- Settings validation.
- Secret never displayed.
