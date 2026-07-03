# HTTP polling remains the aria2 source of truth

Cottid keeps HTTP JSON-RPC polling as the canonical command and refresh path.
Future WebSocket support is an optional notification channel first, not a
replacement RPC transport.

Superseded for download refreshes and download actions by
[ADR-0009](0009-websocket-primary-refresh-and-actions.md): WebSocket is now the
preferred transport for those operations when enabled, while HTTP remains the
fallback and settings transport.

AriaNg and Motrix both demonstrate useful aria2 WebSocket patterns: classify
incoming messages by `id` and `method`, dispatch aria2 notifications by method
name, and clear pending request state on reconnect when WebSocket is used for
RPC commands. Motrix also demonstrates grouped refreshes with
`system.multicall` and a separate future concern for managed aria2c lifecycle.

Cottid borrows those concepts without copying the framework architecture:

- HTTP JSON-RPC remains the source of truth for download snapshots.
- `system.multicall` groups refresh reads.
- WebSocket notifications are typed invalidation signals.
- WebSocket events never directly mutate canonical download state.
- The central scheduler coalesces dirty state and prevents refresh re-entry.
- RPC-over-WebSocket remains future-only and must use request id correlation,
  pending request cleanup on reconnect, connection generation ids, and stale
  response discard if added.

This keeps MVP behavior reliable for users with only an existing aria2 daemon
and avoids adding a WebSocket dependency before the notification channel is
explicitly approved.
