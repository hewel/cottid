# Iced MVU Rules

Organize the app around explicit state, domain-grouped messages, update
handlers, pure views, async tasks, and subscriptions.

- Split state by domain: connection, downloads, stats, selection, add dialog,
  settings, and transient operations.
- Avoid one giant `App` struct, one giant `Message` enum, or one giant update
  function. Use a top-level router that delegates to domain handlers.
- Views read state and emit messages only. No RPC calls, config writes,
  blocking IO, or hidden mutation in views.
- Tasks wrap async RPC/config work and return typed result messages.
- Subscriptions emit central scheduler ticks; individual views, rows, and detail
  panels must not start their own polling.
- The app state must reject refresh re-entry while a refresh task is in flight.
  Stale generation results are ignored.
- WebSocket events should enter the app as dirty invalidation messages. They
  must not directly replace large UI state; the scheduler performs the next
  typed refresh.
