# Context Glossary

## Exact download status filter

Internal status-specific filtering for compatibility and tests. These filters
match one stored download status exactly and are not exposed as sidebar
navigation in the current UI.

## Sidebar filter group

User-facing grouped filter shown in the sidebar. The current groups are
`Active`, which contains waiting, paused, and active downloads, and `Complete`,
which contains error and complete downloads.

## Folder download

A download whose aria2 file entries share one top-level folder. It is presented
as a folder rather than as any one child file.

## TreeList

Reusable `iced` widget for hierarchical rows with expansion, selection,
disabled state, density, optional leading/trailing text, and visual branch
guides.

## Branch guide

Visual connector lines that show hierarchy between TreeList rows.

## Canvas TreeList

TreeList variant whose branch guides are drawn by `iced` Canvas while expansion
and selection remain ordinary application state.

## Download file tree

TreeList data built from a selected download's normalized file paths. It is
display-only UI state; aria2 remains the source of truth for file entries.

## Field

Labeled form unit that groups a label, optional description, input control, and
field-specific validation or status message.

## Field status

Inline message attached to a single Field. It describes validation or
field-specific state and is distinct from modal-level workflow feedback.

## Runtime global option

Live aria2 download-manager setting that Cottid may read or change through the
connected RPC daemon. It is distinct from local Cottid config and daemon startup
configuration.
_Avoid_: aria2 setting, daemon config, aria2.conf option

## New-download default

Cottid-side value applied when creating a new aria2 download. It is not a live
change to existing downloads unless a separate download action explicitly does
that.
_Avoid_: task preset, raw task option

## WebSocket notification

aria2 event frame received over the JSON-RPC WebSocket endpoint. It indicates
that a download changed and is converted into a dirty refresh request; it does
not contain progress snapshot data.

## WebSocket RPC transport

Cottid's preferred transport for download refresh snapshots and download
actions when enabled and connected. It sends the same aria2 JSON-RPC requests as
the HTTP transport.

## HTTP fallback

Recovery path used when WebSocket RPC send, response, timeout, or connection
handling fails. The same operation is retried over HTTP JSON-RPC so WebSocket is
not a single point of failure.

## Dirty refresh

Scheduler refresh requested because an external event indicates cached download
sections may be stale. Dirty refreshes coalesce through the central scheduler
and rebuild canonical download state from aria2 RPC snapshots.
