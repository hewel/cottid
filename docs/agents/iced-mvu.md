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
- Subscriptions should start with polling and later allow WebSocket events
  without changing UI components.
