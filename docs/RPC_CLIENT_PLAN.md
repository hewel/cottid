# Cottid Aria2 JSON-RPC Client Plan

This document defines the planned aria2 JSON-RPC client layer for Cottid. The
client layer is independent from `iced` UI widgets. It may use raw RPC DTOs
internally, but app-facing APIs return domain models or typed errors.

No dependencies are approved by this document.

## Module Responsibilities

### `aria2::client`

App-facing facade for RPC operations.

Responsibilities:

- Own client configuration derived from `RpcEndpoint` and `RpcAuth`.
- Expose high-level operations such as connection test, refresh snapshot,
  selected-download detail, add URI, pause, unpause, remove, and purge stopped
  results.
- Depend on an internal transport abstraction, not on `iced`.
- Return domain models or typed errors.

### `aria2::methods`

Typed request builders for aria2 methods.

Responsibilities:

- Own method names.
- Own parameter ordering.
- Own method-specific key lists.
- Insert auth tokens in the correct position.
- Build batch or multicall requests without exposing raw construction details
  to app code.

### `aria2::raw_types`

Serde-facing JSON-RPC and aria2 DTOs.

Responsibilities:

- JSON-RPC request envelopes.
- JSON-RPC success responses.
- JSON-RPC error responses.
- Batch response DTOs.
- Raw aria2 result DTOs with aria2 field names and string-heavy values.

Raw DTOs must not be imported by `ui/`.

### `aria2::domain`

Normalized app-facing models.

Responsibilities:

- `Gid`.
- `DownloadItem`.
- `DownloadStatus`.
- `DownloadProgress`.
- `DownloadFile`.
- `DownloadError`.
- `GlobalStats`.
- Future typed torrent and Metalink details.

### `aria2::errors`

Typed error categories.

Responsibilities:

- Endpoint validation errors.
- Transport errors.
- HTTP status errors.
- JSON parse errors.
- JSON-RPC/aria2 RPC errors.
- Domain conversion errors.
- Timeout and cancellation errors.
- Future daemon and WebSocket errors.

## RPC Request Structure

Use JSON-RPC 2.0 over HTTP POST for MVP.

Each single request contains:

- JSON-RPC version.
- Client-generated request id.
- Method name.
- Params array.

The client layer owns request id generation and response id validation. Request
ids are also used to correlate batch responses.

Do not use JSON-RPC over HTTP GET in MVP.

## RPC Response And Error Structure

Each single response is one of:

- Success response with matching id and typed raw result.
- Error response with matching id and aria2 RPC code/message.

Response handling should distinguish:

- Transport failure before any valid JSON-RPC response exists.
- HTTP status failure.
- Malformed JSON.
- Valid JSON-RPC error object from aria2.
- Valid JSON-RPC success object whose result cannot convert into a domain
  model.

Aria2 RPC error code/message should be preserved for diagnostics but mapped to
display-safe domain/client errors before reaching UI state.

## Authentication Token Handling

`RpcAuth` controls auth behavior.

- No secret: send method params unchanged.
- Session secret: prepend `token:<secret>` as the first method parameter for
  protected aria2 methods.
- Future persisted secret: same wire behavior, different storage policy.

Important rules:

- Do not put secrets in debug output, logs, error strings, or UI-facing text.
- For `system.multicall`, inject the token into each nested aria2 call rather
  than treating the outer call as authenticated.
- Methods documented as not requiring a secret can remain unauthenticated if
  used later.

## Endpoint Validation

MVP accepts HTTP and HTTPS JSON-RPC endpoints.

Validation rules:

- Reject empty endpoint values.
- Reject non-HTTP(S) schemes for MVP.
- Reject malformed URLs.
- Reject credential-bearing URLs.
- Preserve an explicit path.
- If a user enters a host/port-style endpoint with no path, normalize to
  `/jsonrpc` after approval of the URL parsing approach.

WebSocket URL derivation is future-only and should not affect MVP endpoint
validation.

## MVP Method Wrappers

### `getVersion`

Purpose:

- Test connection.
- Read daemon version and enabled features for display/diagnostics.

### `getGlobalStat`

Purpose:

- Fetch global download speed, upload speed, active count, waiting count,
  stopped count, and stopped total.

Conversion:

- Parse all numeric string fields before returning `GlobalStats`.

### `tellActive`

Purpose:

- Fetch active downloads for the list.

MVP behavior:

- Request only fields needed for `DownloadItem` summaries where practical.

### `tellWaiting`

Purpose:

- Fetch waiting and paused downloads for the list.

MVP behavior:

- Use offset `0`.
- Use an explicit limit chosen by refresh policy.

### `tellStopped`

Purpose:

- Fetch stopped, complete, error, and removed downloads for the list.

MVP behavior:

- Use offset `0`.
- Use an explicit limit chosen by refresh policy.

### `tellStatus`

Purpose:

- Fetch a targeted download status, mainly for selected details or targeted
  refresh.

### `getFiles`

Purpose:

- Fetch selected-download file summaries for the detail panel.

### `getUris`

Purpose:

- Fetch selected-download URI/source summaries when details need them.

### `addUri`

Purpose:

- Add one URI or magnet link in MVP.

MVP exclusions:

- No torrent file picker.
- No Metalink picker.
- No batch input.
- No advanced options.

### `pause`

Purpose:

- Pause one download by `Gid`.

### `unpause`

Purpose:

- Unpause one download by `Gid`.

### `remove`

Purpose:

- Remove one download by `Gid`.

### `purgeDownloadResult`

Purpose:

- Purge completed, error, and removed results from aria2 memory.

## Refresh And Batch Strategy

The app-facing refresh operation should be a batch-style snapshot request.

It returns:

- `GlobalStats`.
- Active downloads.
- Waiting downloads.
- Stopped downloads.
- Enough metadata to preserve ordering and selection by `Gid`.
- A bounded stopped/history page, not unbounded daemon history.

Preferred MVP strategy:

- Use a `BatchRefreshRequest` to decide whether to fetch active, waiting,
  stopped, and selected-detail data for the current scheduler tick.
- Request only fields needed by the current UI.
- Cap stopped refreshes to the latest page during normal polling.
- Preserve the request shape behind the client facade so JSON-RPC batch HTTP
  POST or `system.multicall` can be added later without changing UI code.
- Correlate each response by request id.
- Allow partial failure to become a typed refresh error unless the app later
  explicitly chooses partial snapshot behavior.

Reserved optimization:

- `system.multicall` may be evaluated later.
- Its request shape is hidden behind `fetch_snapshot`.
- Auth token insertion must happen inside each nested aria2 method call.

## Timeout And Retry Strategy

Timeouts:

- Use a short timeout for connection tests.
- Use a normal timeout for refresh and actions.
- Ensure request timeout produces a typed timeout error.
- Do not allow iced tasks to hang indefinitely.

Retries:

- Do not automatically retry mutating methods: `addUri`, `pause`, `unpause`,
  `remove`, `purgeDownloadResult`.
- Optional single retry may be considered for idempotent reads after transport
  failure only.
- Polling refresh naturally retries on the next tick.
- Aria2 RPC errors are not retried automatically.

## Transport Errors Vs Aria2 RPC Errors

Transport errors happen outside a valid JSON-RPC response:

- DNS failure.
- Connection refused.
- TLS failure.
- Timeout.
- HTTP status failure.
- Empty body.

Aria2 RPC errors are valid JSON-RPC error responses:

- They include an aria2 code and message.
- They should be preserved diagnostically.
- They should be converted to display-safe typed errors before reaching UI
  state.

Domain conversion errors happen after a valid success response when required
fields are missing or malformed.

## Domain Conversion Strategy

Conversion happens inside `aria2` before data reaches `app/`.

Rules:

- Convert raw GID strings into `Gid`.
- Convert raw status strings into `DownloadStatus`.
- Parse numeric strings into numeric domain fields.
- Parse boolean strings into booleans.
- Treat omitted optional fields as optional domain data.
- Fail conversion for malformed required fields.
- Preserve unknown status-like values through typed unknown variants.
- Ignore unused unknown non-MVP fields at the raw DTO boundary.

Do not return raw DTOs, raw maps, or untyped JSON from app-facing client
methods.

## Future WebSocket Notification Strategy

WebSocket support belongs in `aria2::websocket` and requires explicit dependency
approval.

Future behavior:

- Keep HTTP JSON-RPC polling as the source of truth.
- Parse aria2 notification frames into typed invalidation events.
- Key events by `Gid`.
- Coalesce notification bursts before requesting a dirty refresh.
- Feed invalidations into the same central refresh scheduler used by polling.
- Never directly mutate canonical download state from WebSocket events.
- Keep UI unaware of whether data came from polling or WebSocket.
- If RPC-over-WebSocket is added later, use request id correlation, connection
  generation ids, pending request cleanup on reconnect, and stale response
  discard.

## Dependency Approval Required

### `serde`

Purpose:

- Typed JSON-RPC and raw DTO serialization/deserialization.

Alternatives:

- Manual serialization/deserialization.

Why std/existing dependencies are insufficient:

- Rust standard library has no JSON serialization support.

Risk:

- Low runtime risk.
- Common crate.
- Derive macros add compile-time cost.

### `serde_json`

Purpose:

- JSON encoding and decoding for JSON-RPC.

Alternatives:

- Manual JSON construction/parsing.

Why std/existing dependencies are insufficient:

- Rust standard library has no JSON parser.

Risk:

- Moderate parsing surface.
- Keep parsing confined to raw DTO and transport boundaries.

### HTTP client crate

Recommended candidate to evaluate:

- `reqwest` with minimal features and Rustls.

Purpose:

- HTTP POST transport to `/jsonrpc`.
- Timeout and HTTP status handling.

Alternatives:

- `ureq` for blocking HTTP.
- Lower-level `hyper` stack.
- Manual TCP/HTTP.

Why std/existing dependencies are insufficient:

- Rust standard library has no ergonomic HTTP or TLS client.

Risk:

- Potentially large dependency tree.
- Async runtime compatibility must be confirmed before approval.
- TLS and HTTP stack increase security maintenance surface.

### Async runtime support

Purpose:

- Only needed if the approved HTTP client requires an async runtime beyond what
  `iced` already provides.

Alternatives:

- Choose a blocking HTTP client inside iced tasks.
- Choose a client compatible with iced's executor.

Why std/existing dependencies are insufficient:

- Rust standard library does not provide a complete async runtime.

Risk:

- Runtime integration complexity.
- Extra dependency size.

### `url` or equivalent URL parser

Purpose:

- Robust endpoint validation and path normalization.

Alternatives:

- Minimal string validation for MVP.

Why std/existing dependencies are insufficient:

- Rust standard library has no full URL parser.

Risk:

- Small dependency.
- Improves validation correctness.

### `thiserror`

Purpose:

- Ergonomic typed error definitions.

Alternatives:

- Manual `Display` and `Error` implementations.

Why std/existing dependencies are insufficient:

- Rust standard library supports errors but not derive-based definitions.

Risk:

- Low.
- Optional convenience, not required.

### WebSocket client crate

Purpose:

- Future aria2 notification stream support.

Alternatives:

- Continue polling.

Why std/existing dependencies are insufficient:

- Rust standard library has no WebSocket client.

Risk:

- Future-only dependency.
- Reconnect/backoff and TLS behavior need careful handling.

## Test Strategy

Request construction tests:

- Correct JSON-RPC version.
- Correct method names.
- Correct request ids.
- Correct params order.
- Correct auth token insertion.
- Correct batch request id correlation.
- Correct `system.multicall` nested-token behavior if multicall is added.

Response parsing tests:

- Successful `getVersion`.
- Successful `getGlobalStat`.
- Successful status/list responses.
- Successful file and URI responses.
- JSON-RPC error object mapping.
- Mismatched id handling.
- Malformed JSON handling.
- Batch partial failure handling.

Domain conversion tests:

- Numeric string parsing.
- Boolean string parsing.
- GID validation.
- Known and unknown status conversion.
- Missing optional fields.
- Malformed required fields.
- Raw DTOs do not leak into app/UI-facing APIs.

Transport behavior tests:

- Timeout.
- Connection refused.
- HTTP status error.
- Invalid endpoint.
- Auth failure.
- No automatic retry for mutating methods.

Live aria2 tests should be manual or opt-in only. They must not be required for
normal unit test runs.
