# Dependency Approval

Do not add any dependency without explicit human approval.

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
