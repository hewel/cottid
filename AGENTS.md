# Cottid Agent Guide

Cottid is a Rust `iced` desktop frontend for controlling an external `aria2c`
engine through aria2 JSON-RPC. The MVP connects to an already-running aria2 RPC
server.

## Critical Rules

- **Dependencies:** Do not add dependencies without explicit human approval. Use Cargo commands such as `cargo add` to add packages or enable dependency features; do not hand-edit dependency entries.
- **Aria2 Integration:** Do not embed aria2, wrap libaria2, or reimplement download logic. Keep MVP work focused on the approved existing-RPC-server scope.
- **Architectural Separation:** Keep UI code separate from RPC/client code. RPC code must not know about `iced` widgets.

## UI & State Boundaries

- **Domain-Driven UI:** UI consumes normalized domain models, never raw JSON-RPC DTOs.
- **Type Conversion:** Model aria2 GIDs and numeric string fields as typed domain data (e.g., parse raw numeric string fields during domain conversion, not in views) before they reach app or UI state.
- **Pure Views:** Never call RPC or perform side-effects from view functions or UI widgets.
- **No Per-Row RPC:** Never issue one RPC request per visible row. Keep expensive formatting and derived display data out of hot view paths.
- **Design Tokens:** Prefer `TOKENS` and UI component/style wrappers for visual spacing, sizing, radius, typography, colors, shadows, and border widths. Use raw numeric literals only when no token exists or the value is an external/API/layout constraint; add a token first when a visual value is reused.

## Realtime Updates & Scheduler

For all current and future implementations:

- **Central Scheduler:** Use a central refresh scheduler. Use `iced::Subscription` for ticks/future event streams and `iced::Task` for async RPC work.
- **Refresh Control:** Never start a new refresh while the previous refresh is still in flight. Use refresh generation IDs and discard stale responses.
- **Optimized Queries:** Prefer aria2 batch requests or `system.multicall` for grouped refreshes, and request only the fields needed by the current UI.
- **Download Storage:** Store downloads by GID and merge incrementally. Do not render unlimited stopped/history rows.
- **State Preservation:** Preserve selection, scroll position, dialogs, form drafts, and row expansion state across refreshes. Keep canonical domain state separate from display-only progress estimation.
- **Adaptive Frequency:** Use different refresh frequencies for active, waiting, and stopped downloads, selected details, and global stats. Reduce frequency when unfocused, minimized, disconnected, idle, or when there are no active downloads.
- **Connection Failures:** Use exponential backoff on connection failures.
- **WebSockets:** Treat future WebSocket notifications as invalidation signals, not as direct full-state replacements. Coalesce WebSocket events and refresh through the normal scheduler.

## Engineering & Testing

- **Testing Requirements:** Add tests around scheduler decisions, stale response handling, GID merge, backoff, and domain conversion. Keep functions small and domain-oriented.

## Detailed agent docs

- [Dependency approval](docs/agents/dependencies.md)
- [Architecture boundaries](docs/agents/architecture.md)
- [Iced MVU rules](docs/agents/iced-mvu.md)
- [Aria2 modeling](docs/agents/aria2-modeling.md)
- [Errors and config](docs/agents/errors-config.md)
- [MVP and future scope](docs/agents/mvp-scope.md)
- [Engineering and tests](docs/agents/engineering-tests.md)

## Agent skills

### Issue tracker

Issues and PRDs are tracked in GitHub Issues; external PRs are not a triage
request surface. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default five-label triage vocabulary: `needs-triage`, `needs-info`,
`ready-for-agent`, `ready-for-human`, and `wontfix`. See
`docs/agents/triage-labels.md`.

### Domain docs

This repo uses a single-context domain-doc layout. See
`docs/agents/domain.md`.
