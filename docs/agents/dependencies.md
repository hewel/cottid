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
