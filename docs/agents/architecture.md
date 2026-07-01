# Architecture Boundaries

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
