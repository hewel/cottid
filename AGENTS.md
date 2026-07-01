# Cottid Agent Guide

Cottid is a Rust `iced` desktop frontend for controlling an external `aria2c`
engine through aria2 JSON-RPC. The MVP connects to an already-running aria2 RPC
server.

Critical rules:

- Do not embed aria2, wrap libaria2, or reimplement download logic.
- Do not add dependencies without explicit human approval.
- Use Cargo commands such as `cargo add` to add packages or enable dependency
  features; do not hand-edit dependency entries for that.
- Keep UI code separate from RPC/client code.
- UI consumes normalized domain models, never raw JSON-RPC DTOs.
- Model aria2 GIDs and numeric string fields as typed domain data before they
  reach app or UI state.
- Keep MVP work focused on the approved existing-RPC-server scope.

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

## Realtime update rules

For all future implementation:

- Use a central refresh scheduler.
- Use iced Subscription for ticks and future external event streams.
- Use iced Task for async RPC work.
- Never call RPC from view functions or UI widgets.
- Never issue one RPC request per visible row.
- Never start a new refresh while the previous refresh is still in flight.
- Use refresh generation IDs and discard stale responses.
- Prefer aria2 batch requests or system.multicall for grouped refreshes.
- Request only the fields needed by the current UI.
- Parse raw aria2 numeric string fields during domain conversion, not in views.
- Store downloads by GID and merge incrementally.
- Preserve selection, scroll position, dialogs, form drafts, and row expansion state across refreshes.
- Use different refresh frequencies for active downloads, waiting downloads, stopped downloads, selected details, and global stats.
- Reduce refresh frequency when unfocused, minimized, disconnected, idle, or when there are no active downloads.
- Use exponential backoff on connection failures.
- Treat future WebSocket notifications as invalidation signals, not as direct full-state replacements.
- Coalesce WebSocket events and refresh through the normal scheduler.
- Do not render unlimited stopped/history rows.
- Keep expensive formatting and derived display data out of hot view paths where possible.
- Keep canonical domain state separate from display-only progress estimation.
- Add tests around scheduler decisions, stale response handling, GID merge, backoff, and domain conversion.
- Do not add dependencies without explicit human approval.
