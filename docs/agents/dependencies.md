# Dependency Approval

Do not add any dependency without explicit human approval.

After approval, use Cargo tooling to change dependencies:

- Add packages with `cargo add`.
- Enable dependency features with `cargo add <crate> --features ...`.
- Disable default features with `cargo add <crate> --no-default-features ...`.
- Do not hand-edit dependency entries in `Cargo.toml` when Cargo can express
  the change.

Every dependency proposal must include:

- Crate name.
- Purpose.
- Why existing dependencies or the standard library are insufficient.
- Alternatives considered.
- Build size risk.
- Runtime risk.
- Security risk.
- Maintenance risk.

Put useful but unapproved crates under `Dependency approval required` and do not
write code that assumes they exist.

## Approved Dependencies

### `getrandom`

- Approved for: managed local daemon mode.
- Added with: `cargo add getrandom`.
- Purpose: generate a high-entropy session-only RPC secret for each managed
  `aria2c` child process.
- Why existing dependencies or the standard library are insufficient: Rust's
  standard library does not expose a cross-platform cryptographically secure
  random byte API. Reusing existing config or endpoint data would make the
  managed RPC secret predictable.
- Alternatives considered:
  - `rand`: broader API and larger dependency surface than needed.
  - Reading `/dev/urandom` directly: Unix-specific and easy to implement
    incorrectly across platforms.
  - Deterministic or timestamp-based tokens: rejected because they are
    guessable.
- Build size risk: low; the crate is small and focused.
- Runtime risk: low; startup fails display-safely if the OS randomness source
  is unavailable.
- Security risk: low relative to alternatives; the generated value is
  session-only and is not persisted.
- Maintenance risk: low; `getrandom` is a narrow, widely used OS randomness
  wrapper.
