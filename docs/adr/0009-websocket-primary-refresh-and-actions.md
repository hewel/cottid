# WebSocket primary refresh and actions

Cottid uses aria2 WebSocket support as the preferred transport for download
refresh snapshots and download actions when the setting is enabled. HTTP
JSON-RPC remains the fallback transport and continues to handle connection
testing and runtime global option settings.

## Context

The earlier HTTP-polling decision kept WebSocket support future-only and
notification-oriented. The current requirement is broader: progress snapshots
should prefer WebSocket once connected, while still falling back to HTTP if the
WebSocket path fails.

aria2 notifications do not carry byte-level progress. Progress still comes from
RPC snapshot methods such as `tellActive`, `tellWaiting`, `tellStopped`,
`tellStatus`, and `getGlobalStat`.

## Decision

- Derive `ws://` from `http://` and `wss://` from `https://`, preserving host,
  port, and path.
- Keep endpoint and secret separate; secrets remain JSON-RPC params and are not
  encoded into WebSocket URLs.
- Use WebSocket first for refresh snapshots, selected detail refreshes, add URI,
  pause, unpause, remove, and purge stopped.
- Retry the same operation over HTTP if WebSocket send, response, timeout, or
  connection handling fails.
- Keep connection testing and runtime global option reads/writes on HTTP.
- Treat WebSocket notification frames as dirty invalidation signals feeding the
  existing scheduler.

## Consequences

WebSocket improves update latency and can carry the regular refresh/action RPC
traffic, but it does not replace the scheduler or snapshot merge path. HTTP
remains necessary for fallback and settings operations.
