# Cottid Domain Model

This document defines the high-level domain model for Cottid, a Rust `iced`
frontend for controlling `aria2c` through aria2 JSON-RPC. Cottid can connect to
an external user-managed daemon or own a managed local child process.

The UI must consume normalized domain models only. Raw aria2 JSON-RPC envelopes,
responses, and string-heavy DTOs stay inside `aria2::raw_types` and are
converted before reaching `app/` or `ui/`.

## Core Download Domain

### Gid

`Gid` is the app's opaque identifier for an aria2 download.

- Wraps the aria2 GID string instead of passing arbitrary strings throughout the
  app.
- Is used for selection, row identity, actions, parent/child relationships, and
  future WebSocket events.
- Validation should reject empty values and control characters.
- Do not over-restrict the format to hex unless all future aria2 cases are
  proven to match that constraint.

### DownloadStatus

`DownloadStatus` represents normalized aria2 status values.

Known MVP statuses:

- Active.
- Waiting.
- Paused.
- Error.
- Complete.
- Removed.

The model should also preserve unknown future statuses through a typed unknown
variant with a sanitized label. UI code must switch on this enum, not raw aria2
status strings.

### DownloadProgress

`DownloadProgress` contains numeric transfer state.

Expected fields:

- Total bytes.
- Completed bytes.
- Uploaded bytes.
- Download speed.
- Upload speed.
- Connection count.

Percent, remaining bytes, and ETA should be derived by helpers instead of
stored as primary state. Unknown total length should be represented explicitly
rather than guessed.

### DownloadFile

`DownloadFile` is a summary of one aria2 file entry.

Expected fields:

- File index.
- Path or display name.
- Total length.
- Completed length.
- Selected flag.
- URI summaries when needed for detail views.

The MVP only needs enough file data for the detail panel. Future torrent file
selection should extend this model instead of exposing raw file DTOs.

### DownloadError

`DownloadError` represents a download-specific error.

Expected fields:

- Optional aria2 error code.
- Optional aria2 error message.
- Domain error category when known.
- Display-safe summary.

Diagnostic details may be retained for troubleshooting, but secrets must never
be included in display-ready error text.

### DownloadItem

`DownloadItem` is the main app-facing download model.

Expected fields:

- `Gid`.
- `DownloadStatus`.
- `DownloadProgress`.
- Optional display name.
- Directory or path summary.
- File summaries.
- Optional `DownloadError`.
- Relationship GIDs for followed-by, following, or belongs-to relationships.

`DownloadItem` must not contain raw RPC DTOs or untyped JSON. Torrent and
Metalink-specific details should be optional future extensions, not required MVP
fields.

### GlobalStats

`GlobalStats` represents daemon-wide transfer state.

Expected fields:

- Overall download speed.
- Overall upload speed.
- Active download count.
- Waiting download count.
- Stopped download count.
- Optional total stopped count if aria2 reports it separately.

All numeric fields from aria2 are parsed before entering this model. Display
formatting belongs in `util/`.

## Connection And Settings Domain

### RpcEndpoint

`RpcEndpoint` describes the aria2 JSON-RPC endpoint.

Expected fields:

- Endpoint URL.
- Display-safe label.
- Timeout or polling-related settings only after approval.

Validation should happen when settings are edited and again before connection
attempts. The default MVP target is an existing local aria2 RPC daemon.

### RpcAuth

`RpcAuth` describes RPC authentication.

Expected modes:

- No secret.
- Session-only secret.
- Persistent token loaded from the OS keyring.
- Plaintext fallback token after explicit user confirmation.

Secret values must not appear in debug output, logs, UI labels, or error text.
Persistent tokens are bound to the exact `RpcEndpoint` URL. Changing endpoints
requires storing a new token and deleting the old endpoint's stored token.

### PersistedConfig

`PersistedConfig` describes local Cottid settings stored on disk.

Expected fields:

- RPC endpoint URL.
- Polling interval.
- Selected download filter or other UI preferences.
- Auth storage policy.

The config file is TOML at the existing Cottid config path. It may contain a
plaintext fallback token only after explicit user confirmation. Keyring-backed
tokens store only metadata in the config file; the token value lives in the OS
credential store.

### AuthStorage

`AuthStorage` describes where a token is expected to live.

Expected modes:

- None.
- Keyring.
- Plaintext fallback.
- Session only.

Keyring is the preferred persistent mode. Plaintext fallback is a convenience
fallback for unavailable keyrings, not the default security posture. Session
only means the token is usable until exit and must be entered again next launch.

### Managed Local Daemon

A managed local daemon is an `aria2c` child process started and owned by
Cottid.

- It is configured by top-level `daemon/` modules, not by `aria2/` RPC modules.
- It listens on loopback only with an ephemeral port and generated session
  secret.
- It is never started with `--daemon=true`; Cottid must keep the child handle so
  lifecycle ownership is explicit.
- Its generated endpoint and secret are runtime-only and must not be persisted.

### External Daemon

An external daemon is an aria2 JSON-RPC server started and administered outside
Cottid.

- Cottid may connect, authenticate, query, and send user-requested aria2 RPC
  commands to it.
- Cottid must not shut it down, overwrite its startup configuration, or assume
  ownership of its process lifecycle.
- External endpoint and auth settings are persisted according to the configured
  `AuthStorage` policy.

### DaemonState

`DaemonState` tracks Cottid's process-lifecycle ownership separately from RPC
connectivity.

Expected states:

- External.
- Managed stopped.
- Managed starting.
- Managed running.
- Managed stopping.
- Managed crashed.
- Managed failed.

It owns the managed child manager only while Cottid is responsible for that
child. Intentional shutdown must not be treated as a crash and must not trigger
auto-restart.

### ConnectionState

`ConnectionState` tracks connection lifecycle.

Expected states:

- Disconnected.
- Editing or untested settings.
- Testing or connecting.
- Connected.
- Stale after refresh failure.
- Failed.

It should store the last successful endpoint identity and a request generation
or equivalent guard so stale async responses cannot overwrite newer settings.
Connection-level errors stay separate from per-download errors. It does not own
the managed child process; it only records whether the current RPC settings have
tested successfully.

### ManagedDaemonConfig

`ManagedDaemonConfig` describes how Cottid starts a managed local `aria2c`
child.

Expected responsibilities:

- Create the managed root/config/session/log/download paths once and preserve
  existing config files.
- Resolve a configured binary path before falling back to `PATH` lookup.
- Reserve an ephemeral loopback RPC port.
- Generate a high-entropy session-only RPC secret.
- Build startup arguments for RPC, session, log, and download paths without
  `--daemon=true`.
- Carry readiness timeout and polling/WebSocket preferences into the runtime
  connection settings.

It must not include user-facing download data or raw JSON-RPC DTOs.

## App State Separation

### AppState

`AppState` is only the top-level composition root.

It owns:

- `DaemonState`.
- `ConnectionState`.
- `DownloadsState`.
- `SettingsState`.
- `UiState`.
- Global app status or error banner state.

It should route messages and delegate update logic to domain handlers rather
than becoming the only implementation unit.

### DownloadsState

`DownloadsState` owns the current download snapshot.

Expected fields:

- Downloads keyed by `Gid`.
- Ordered or grouped views for active, waiting, and stopped downloads.
- Selected `Gid`.
- `GlobalStats`.
- Refresh lifecycle state.
- Per-download operation state.
- Last successful snapshot for recovery.

Refresh failures should not clear the last known good snapshot.

### UiState

`UiState` contains only view concerns.

Expected fields:

- Active panel.
- Active modal identity.
- Sort, filter, and grouping mode.
- Transient input text.
- Focus or request flags.

It must not hold raw RPC data or trigger RPC calls directly.

### SettingsState

`SettingsState` contains editable settings.

Expected fields:

- Draft endpoint.
- Draft auth policy.
- Polling interval.
- UI preferences.
- Validation errors.

Draft settings should be separate from the currently applied connection
settings.

## Raw RPC And Conversion Boundary

### Raw DTO Layer

`aria2::raw_types` mirrors JSON-RPC and aria2 response shapes.

It owns:

- JSON-RPC request and response envelopes.
- RPC error DTOs.
- Raw status/download/file/global-stat responses.
- Raw fields with aria2 names and string-heavy values.

Raw DTOs may use serde-facing names and shapes. They should not be imported by
`ui/`.

### Domain Conversion Layer

The conversion layer lives inside `aria2` and converts raw DTOs into domain
models before returning data to `app/`.

It is responsible for:

- Parsing `Gid`.
- Mapping raw status strings to `DownloadStatus`.
- Parsing numeric strings into numeric domain fields.
- Parsing boolean strings such as selected flags.
- Treating omitted optional fields as optional domain data.
- Returning typed parse errors for malformed required fields.

App and UI state should never need to know whether aria2 sent a number as a
string.

## Numeric String Handling

Aria2 commonly returns numeric fields as strings. These are parsed at the
raw-to-domain boundary.

Examples:

- Lengths and byte counters become byte-count numeric fields.
- Speeds become byte-per-second numeric fields.
- Counts become integer count fields.
- File indexes become integer indexes.

Malformed required numeric fields should fail conversion for that response.
Malformed optional numeric fields should become a typed partial-data error only
when the missing value affects user-visible correctness.

## Unknown And Future Fields

Do not store arbitrary raw JSON in app or UI state.

Unknown future data should be handled by:

- Typed unknown enum variants for status-like values.
- Ignoring unused non-MVP fields at the raw DTO boundary.
- Adding typed domain fields when future features need the data.
- Adding typed extension structs for larger feature areas.

Untyped maps or `serde_json::Value` should only appear in documented raw-layer
code paths, never in view-facing state.

## Future Detail Extensions

### TorrentDetails

`TorrentDetails` is an optional typed detail extension for selected downloads.
It is display-only in the MVP UI and must not introduce torrent-specific
commands.

Current fields:

- Info hash.
- Seeder flag.
- Seeder count.

Potential future fields:

- Leecher count.
- Peers.
- Trackers.
- Piece information.
- Selected file state.

Keep this detail-oriented so the MVP list model remains small.

### MetalinkDetails

Reserve a typed optional Metalink detail model for later.

Potential future fields:

- Metalink source metadata.
- Generated child downloads.
- File grouping.
- Mirror/source details.

Use relationship GIDs to represent followed-by, following, and belongs-to links
without adding raw aria2 structures to UI state.

### WebSocket Events

WebSocket notifications become typed invalidation events keyed by `Gid`. They
do not directly replace or mutate canonical download state.

Known notification methods:

- `aria2.onDownloadStart`.
- `aria2.onDownloadPause`.
- `aria2.onDownloadStop`.
- `aria2.onDownloadComplete`.
- `aria2.onDownloadError`.
- `aria2.onBtDownloadComplete`.

Notification handling marks affected list sections dirty and lets the central
refresh scheduler perform the next refresh through the preferred transport.
BitTorrent completion stays distinct from normal completion because aria2
reports torrent download completion before seeding has ended.

### RefreshInvalidation

`RefreshInvalidation` is a scheduler input that says cached download data may
be stale.

Sources include:

- User commands that changed aria2 state.
- Polling cadence.
- WebSocket notifications.

Invalidation is not download data. It carries only enough information to choose
which sections or selected detail should be refreshed.

### Managed Daemon State

Local `aria2c` process lifecycle belongs outside download domain models.
Connection state may reference daemon availability, but downloads remain
RPC-derived. Managed daemon state belongs in `app::State` plus top-level
`daemon/` types, not inside download rows or aria2 raw DTOs.

## Validation And Testing Targets

Implementation should test:

- Valid and invalid `Gid` parsing.
- Known and unknown status conversion.
- Numeric string parsing for progress, files, and global stats.
- Optional missing fields.
- Required malformed fields.
- RPC error DTO to domain error conversion.
- That UI-facing APIs expose domain models only.
- That secrets never appear in display/debug output.
