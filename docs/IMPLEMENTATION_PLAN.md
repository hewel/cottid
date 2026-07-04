# Cottid Phased Implementation Plan

This plan builds Cottid as a Rust `iced` desktop frontend for `aria2c`
controlled through aria2 JSON-RPC. The current app supports an approved managed
local `aria2c` child process and external user-managed RPC daemons. Future
torrent/Metalink depth, browser integration, and multi-profile support must stay
out of scope unless explicitly approved.

## Dependency Approval Gates

No dependency may be added without human approval.

- Before app-shell work: propose and approve `iced`.
- Before RPC work: propose and approve JSON serialization and HTTP transport
  crates.
- Before config persistence: propose and approve config path/format crates, or
  document a stdlib-only fallback and its limits.
- Future-only approvals: file dialog, tray, browser integration, and any new
  process-management helpers.

Each proposal must include crate name, purpose, alternatives, why existing
dependencies or std are insufficient, and build/runtime/security/maintenance
impact.

## MVP Milestones

### 1. Repository Skeleton And Architecture Documents

Goal: lock the architecture guide, dependency policy, module layout, and MVP
boundary.

Affected modules: docs first; later empty `app/`, `aria2/`, `ui/`, `config/`,
and `util/` module shells.

User-visible behavior: none.

Risks: adding too much structure before implementation needs it.

Do not include: app logic, dependencies, RPC code, UI behavior.

Architecture check: `AGENTS.md` remains the source of repo rules.

### 2. App Shell With Iced MVU Structure

Goal: create a minimal `iced` app with top-level state, routed messages,
`update`, `view`, task return path, and subscription hook.

Affected modules: `main`, `app`, basic `ui`.

User-visible behavior: a desktop window with an offline/empty shell.

Risks: allowing the top-level app state or message enum to become the only
architectural unit.

Do not include: real RPC, config persistence, download actions.

Architecture check: top-level message/update delegates to domain handlers.

### 3. Configuration Model And Connection Settings

Goal: model endpoint URL, optional secret policy, polling interval, and UI
preferences.

Affected modules: `config`, settings state, settings UI.

User-visible behavior: editable connection settings persisted as local TOML,
with OS-keyring token storage and explicit plaintext fallback when secure
storage is unavailable.

Risks: insecure secret persistence or premature multi-profile design.

Do not include: daemon config, database, multi-profile UI.

Architecture check: secret handling policy is explicit and documented.

Settings migration decisions from AriaNg are tracked in
`docs/SETTINGS_MIGRATION.md`; use that document to choose future app, RPC,
aria2 global, and task-default settings deliberately instead of copying AriaNg's
full settings surface.

### 4. Aria2 JSON-RPC Client Abstraction

Goal: introduce transport abstraction, JSON-RPC envelopes, typed method
wrappers, raw DTOs, domain conversion, and typed errors.

Affected modules: `aria2::client`, `aria2::methods`, `aria2::raw_types`,
`aria2::domain`, `aria2::errors`.

User-visible behavior: none unless wired to a test action.

Risks: leaking raw JSON or raw RPC DTOs into app/UI state.

Do not include in this phase: WebSocket transport, daemon management, deep
torrent/Metalink models.

Architecture check: UI cannot import raw RPC DTOs.

### 5. Connection Test And Global Stats

Goal: wire connection test and global stats fetch into `iced` tasks.

Affected modules: connection state/update, RPC client, settings UI, status bar.

User-visible behavior: user can test endpoint and see connected/offline/error
state plus basic global stats.

Risks: stale async responses overwriting newer endpoint state.

Do not include: download list, mutations, persistent secrets.

Architecture check: task results carry typed domain data or typed errors.

### 6. Download List Polling

Goal: poll active, waiting, and stopped downloads and normalize them into domain
models.

Affected modules: downloads state/update, subscription, RPC methods, downloads
UI.

User-visible behavior: user sees a periodically updating download list.

Risks: clearing useful data on transient refresh failure.

Do not include in this phase: WebSocket notifications, queue reordering,
torrent file tree.

Architecture check: aria2 GIDs and numeric string fields are parsed before app
state.

### 7. Add URI/Magnet Flow

Goal: add dialog state, basic validation, `aria2.addUri`, and refresh after
success.

Affected modules: add-download state/update, add dialog UI, RPC methods.

User-visible behavior: user can submit a URI or magnet link.

Risks: over-validating inputs that aria2 can handle itself.

Do not include: torrent file picker, Metalink picker, advanced per-download
options.

Architecture check: UI emits intent; RPC call stays in app/client layer.

### 8. Basic Download Actions

Goal: support pause, unpause, remove, and purge stopped results with operation
state.

Affected modules: downloads state/update, task row UI, toolbar/status UI, RPC
methods.

User-visible behavior: user can control individual downloads and purge stopped
results.

Risks: destructive remove without clear UI affordance.

Do not include: force remove, pause all, queue reorder, scheduler.

Architecture check: per-download errors attach to the affected domain item where
possible.

### 9. Detail Panel

Goal: show selected download details from normalized domain data.

Affected modules: selection state/update, detail panel UI, domain models.

User-visible behavior: user sees GID, status, progress, speeds, paths/files, and
basic error/message details for the selected download.

Risks: making the detail model a raw aria2 mirror.

Do not include: peers, trackers, piece maps, torrent file selection.

Architecture check: detail UI consumes domain detail models only.

### 10. Error Display And Recovery

Goal: show typed errors, preserve stale snapshots, and provide retry/recovery
paths.

Affected modules: error types, connection/download updates, status UI.

User-visible behavior: clear errors for invalid endpoint, auth failure,
transport failure, RPC errors, parse errors, and config problems.

Risks: exposing secrets in diagnostics.

Do not include: telemetry, crash reporting, daemon log viewer.

Architecture check: secrets are never rendered or logged.

### 11. Polish MVP

Goal: improve layout, empty/loading states, formatting, keyboard basics, and
focused tests.

Affected modules: all MVP modules, especially `ui` and `util`.

User-visible behavior: the app feels usable for controlling an existing aria2
server.

Risks: drifting into non-MVP features.

Do not include in this phase: WebSocket, daemon lifecycle, advanced
torrent/Metalink UI, multi-profile support, tray.

Architecture check: MVP remains one-endpoint, polling-first, and JSON-RPC-only.

## Later Approved Milestones

### 12. WebSocket Notification Module

Goal: add `aria2::websocket` and subscription integration for aria2
notifications and preferred WebSocket RPC transport where enabled.

User-visible behavior: faster updates with less polling.

Risks: reconnect, backoff, and event ordering complexity.

Do not include in this phase: daemon management or UI rewrites.

### 13. Managed Aria2c Daemon Module

Goal: add managed local `aria2c` process lifecycle in top-level `daemon/`.

User-visible behavior: new/default config can run a managed local daemon; users
may still choose an external daemon.

Risks: process cleanup, port/secret conflicts, and platform differences.

Do not include: embedding aria2, using libaria2, making managed mode required,
or shutting down external daemons.

### 14. Torrent And Metalink Enhancements

Goal: add torrent/metalink add flows, file selection, peers, trackers, and rich
detail variants.

User-visible behavior: user can inspect and manage BitTorrent/Metalink-specific
data.

Risks: bloating the generic download model.

Do not include: browser integration or multi-server profiles.

### 15. Multi-Profile And Integration Features

Goal: add server profiles, browser/native messaging, tray, speed profiles,
scheduler, and queue tools.

User-visible behavior: user can switch/control multiple endpoints and accept
external add requests.

Risks: config migration and security boundaries.

Do not include: rewriting the core RPC/domain layers.

## Test Strategy

- Unit-test raw RPC DTO to domain conversion.
- Unit-test GID typing and numeric string parsing.
- Unit-test RPC request construction, including secret-token placement.
- Unit-test formatting helpers for bytes, speed, ETA, progress, and statuses.
- Unit-test update handlers with fake clients for connection, refresh, add,
  pause, unpause, remove, and purge flows.
- Test config defaults, load/save, invalid config recovery, and secret policy
  once persistence exists.
- Keep live aria2 tests manual or opt-in; they must not be required for normal
  unit test runs.

## Manual QA Scenarios

- Launch with no config and verify default offline/settings state.
- Connect to a valid local aria2 RPC endpoint with and without secret.
- Enter invalid endpoint or secret and verify clear errors.
- Add a normal URI and a magnet link.
- Pause, unpause, remove, and purge stopped results.
- Stop aria2 while the app is running; verify stale data remains visible and
  recovery works after restart.
- Verify raw JSON and secrets never appear in the UI.

## Architecture Checks After Each Milestone

- No dependency added without recorded human approval.
- UI modules consume only domain/view models.
- RPC modules do not import or depend on `iced` widgets.
- App state remains split by domain.
- Top-level update/message stays a router, not a giant implementation block.
- Raw aria2 numeric strings are parsed before entering app/UI state.
- WebSocket logic remains in `aria2::websocket`; managed process lifecycle
  remains in top-level `daemon/`.
- MVP work does not add database storage, multi-profile UI, tray integration,
  or advanced torrent/Metalink behavior.
