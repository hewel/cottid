# Aria2 Modeling

Model aria2 JSON-RPC explicitly:

- `aria2::raw_types`: JSON-RPC envelopes, params, and serde-facing responses.
- `aria2::methods`: method-specific request construction.
- `aria2::client`: app-facing client operations.
- `aria2::domain`: normalized downloads, stats, files, and GID types.
- `aria2::errors`: transport, auth, RPC, parse, and config-facing client
  errors.
- `daemon`: top-level managed local `aria2c` lifecycle, including child
  process ownership and display-safe daemon errors.

Do not store raw `serde_json::Value` in app state unless a specific reason is
documented. Favor typed models over loose maps. Treat aria2 GID as a domain type
and parse aria2 numeric string fields into numeric domain fields before they
reach app or UI state.
