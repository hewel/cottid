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

## Root Layout

Use a stable desktop shell:

- Top toolbar.
- Main content row.
- Bottom status bar.
- Overlay layer for add/settings/error dialogs.

The main content row contains:

- A narrow sidebar or filter-tabs area.
- A central download list.
- An optional right-side detail panel for the selected download.

Offline, empty, loading, and stale states should appear inside the main content
area instead of forcing separate app layouts.

## Toolbar

The toolbar shows current app and daemon context.

MVP content:

- App title or active endpoint label.
- Connection indicator.
- Global download speed.
- Global upload speed.
- Refresh state.
- Add URI/magnet action.
- Manual refresh action.
- Pause selected action.
- Unpause selected action.
- Remove selected action.
- Purge stopped action.
- Settings action.

Toolbar actions should be disabled or visually softened when disconnected, when
no task is selected, or when a relevant operation is already pending.

Future toolbar space:

- Profile switcher.
- Speed profile selector.
- Queue controls.
- Daemon start/stop controls.
- Tray-related status affordances.

## Sidebar Or Filter Tabs

MVP should use simple filter tabs or a narrow sidebar with counts.

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

## Task Row

Each task row displays one `DownloadItem` summary.

MVP fields:

- Display name or path summary.
- Status.
- Progress.
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

Task rows should emit messages with `Gid`; parent update logic decides whether
to run RPC tasks. Do not expose raw aria2 status strings or raw RPC errors in
the row.

Future row additions:

- Queue position.
- Seeding indicator.
- Torrent/Metalink badges.
- Speed limit profile marker.

## Add Download Dialog

The add dialog is a modal overlay.

MVP fields and behavior:

- URI or magnet input.
- Minimal validation feedback.
- Submit action.
- Cancel action.
- Pending/error state while submit is in progress.

The MVP accepts one URI or magnet per submission. Do not include torrent file
picker, Metalink picker, batch input, or advanced aria2 options yet.

## Detail Panel

The detail panel shows information for the selected `DownloadItem`.

MVP content:

- GID.
- Status.
- Directory or path summary.
- Progress.
- Download/upload speed.
- Completed, uploaded, and total bytes.
- File summaries.
- Basic error details.

When nothing is selected, show a compact empty-selection state. When the
selected download disappears, clear or update the selection through
`DownloadsState` logic.

Future detail sections:

- Torrent file list and selected files.
- Peer list.
- Trackers.
- Piece information.
- Metalink source details.
- Daemon/request diagnostics.

These future sections should be backed by typed domain models, not raw aria2
payloads.

## Settings Page

Use a modal or side panel opened from the toolbar.

MVP settings:

- RPC endpoint URL.
- Auth mode.
- Optional secret input.
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

The status bar remains visible across the main shell.

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
- Dialog visibility.
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
