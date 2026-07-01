# Cottid Agent Guide

Cottid is a Rust `iced` desktop frontend for controlling an external `aria2c`
engine through aria2 JSON-RPC. The MVP connects to an already-running aria2 RPC
server.

Critical rules:

- Do not embed aria2, wrap libaria2, or reimplement download logic.
- Do not add dependencies without explicit human approval.
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
