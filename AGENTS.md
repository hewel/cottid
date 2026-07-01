# Cottid Agent Guide

Cottid is a Rust `iced` desktop frontend for controlling an external `aria2c`
engine through aria2 JSON-RPC. Do not embed aria2, wrap libaria2, or
reimplement download logic. The MVP connects to an already-running aria2 RPC
server.

## Dependency Rule

Do not add any dependency without explicit human approval.

Every dependency proposal must include the crate name, purpose, why existing
dependencies or the standard library are insufficient, alternatives considered,
and build size, runtime, security, and maintenance risk. Put useful but
unapproved crates under `Dependency approval required` and do not write code
that assumes they exist.

## Architecture

Use clear module boundaries:

- `app/`: top-level state, message routing, update orchestration, tasks,
  subscriptions, and root view composition.
- `aria2/`: JSON-RPC client abstraction, typed methods, raw RPC DTOs, domain
  models, errors, and reserved `websocket`/`daemon` modules.
- `ui/`: `iced` widgets and view composition only.
- `config/`: local settings, endpoint/secret policy, UI preferences, and
  persistence.
- `util/`: formatting and small reusable helpers.

Keep UI code separate from RPC/client code. UI components must consume
normalized domain models, not raw JSON or raw RPC DTOs. RPC code must not know
about `iced` widgets.

## Iced MVU Rules

Organize the app around explicit state, domain-grouped messages, update
handlers, pure views, async tasks, and subscriptions.

- Split state by domain: connection, downloads, stats, selection, add dialog,
  settings, and transient operations.
- Avoid one giant `App` struct, one giant `Message` enum, or one giant update
  function. Use a top-level router that delegates to domain handlers.
- Views read state and emit messages only. No RPC calls, config writes,
  blocking IO, or hidden mutation in views.
- Tasks wrap async RPC/config work and return typed result messages.
- Subscriptions should start with polling and later allow WebSocket events
  without changing UI components.

## Aria2 Modeling

Model aria2 JSON-RPC explicitly:

- `aria2::raw_types`: JSON-RPC envelopes, params, and serde-facing responses.
- `aria2::methods`: method-specific request construction.
- `aria2::client`: app-facing client operations.
- `aria2::domain`: normalized downloads, stats, files, and GID types.
- `aria2::errors`: transport, auth, RPC, parse, config, and future daemon
  errors.

Do not store raw `serde_json::Value` in app state unless a specific reason is
documented. Favor typed models over loose maps. Treat aria2 GID as a domain type
and parse aria2 numeric string fields into numeric domain fields before they
reach app or UI state.

## Errors And Config

Use typed error categories and keep diagnostic detail available without exposing
secrets. Show concise user-facing errors. Preserve the last known good download
snapshot when refresh fails. Attach command errors to the affected download
where possible; show connection/config errors in status or settings UI.

Persist only basic local config in the MVP: endpoint URL, optional secret
handling policy, polling interval, and UI preferences. Secret persistence must
be explicit; if secure storage is not approved, keep secrets session-only or
require a documented plaintext opt-in. Do not add database storage unless
explicitly approved.

## MVP And Future Features

MVP scope: connect/test an existing RPC endpoint, fetch global stats, fetch
active/waiting/stopped downloads, add URI or magnet, pause/unpause/remove
downloads, purge stopped results, show a basic detail panel, and persist basic
config subject to dependency approval.

Keep these out of MVP unless explicitly approved: WebSocket notifications,
managed `aria2c` lifecycle, torrent-file upload, Metalink upload, torrent file
selection, peer/BitTorrent details, scheduler, queue reordering, speed profiles,
browser extension integration, multi-server profiles, and system tray.

Future WebSocket work belongs in `aria2::websocket`; future local process
management belongs in `aria2::daemon`. Add advanced features by extending domain
models and app messages, not by coupling UI to raw RPC shapes.

## Engineering And Tests

Keep functions small and domain-oriented. Make the MVP useful before adding
advanced features.

Once implementation starts, add focused tests for raw-to-domain conversion, GID
and numeric parsing, formatting helpers, RPC request construction including
secret-token placement, update handlers, and config defaults/persistence.
