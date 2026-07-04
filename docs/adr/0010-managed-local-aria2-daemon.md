# Managed local aria2 daemon lifecycle

## Status

Accepted.

## Context

Cottid originally connected only to an already-running aria2 JSON-RPC daemon.
The managed local daemon work adds an easier default path while preserving the
external-daemon boundary for users who already administer aria2 themselves.

Managed mode must not turn Cottid into an aria2 implementation or a general
`aria2.conf` editor. The app only owns child processes that it starts.

## Decision

- New/default config uses managed local daemon mode. Existing pre-mode config
  and legacy key-value config continue to load as external mode so established
  setups are not silently taken over.
- Managed mode starts an `aria2c` child process and never passes
  `--daemon=true`. Cottid keeps the child handle in app state through
  `DaemonManager`.
- Managed runtime RPC listens on loopback with an ephemeral port and a
  generated session-only secret. The generated endpoint and secret are not
  persisted.
- Managed paths are created under the managed root once. Existing config files
  are preserved instead of rewritten.
- The top-level `daemon/` module owns binary resolution, path preparation,
  process arguments, readiness, monitoring, shutdown, and daemon errors.
  `aria2/` remains the JSON-RPC client/domain boundary.
- The app starts refreshes, runtime option reads, and WebSocket notification
  checks only after managed readiness succeeds.
- `State` owns the current managed `DaemonManager`. UI and RPC code do not own
  or infer child-process lifecycle.
- Unexpected child exit is treated as a crash. The app preserves the current
  download snapshot, records a display-safe crash error, and attempts one
  automatic restart with fresh runtime port and secret.
- Intentional shutdown is separate from crash handling. Closing the window is
  gated by a graceful stop workflow: attempt `aria2.saveSession`, attempt
  `aria2.shutdown`, wait for child exit, and kill plus wait after timeout.
- Managed-to-external mode switch stops only the owned managed child before
  testing the external connection. External-to-managed mode switch never sends
  shutdown to an external user-managed daemon.
- `aria2.saveSession` failure is recorded as a warning, but shutdown continues.

## Consequences

Managed mode is easier for first launch but still uses aria2's JSON-RPC API as
the only download-control boundary. Runtime credentials are short-lived and
process ownership is explicit.

External mode remains safe for user-managed daemons: Cottid may connect and send
ordinary user actions, but it does not shut down or reconfigure the process
lifecycle.

The shutdown path now depends on window close interception. The app disables
automatic close-on-request and closes the window programmatically after managed
shutdown completes.
