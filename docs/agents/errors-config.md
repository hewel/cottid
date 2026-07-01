# Errors And Config

Use typed error categories and keep diagnostic detail available without exposing
secrets. Show concise user-facing errors. Preserve the last known good download
snapshot when refresh fails. Attach command errors to the affected download
where possible; show connection/config errors in status or settings UI.

Persist only basic local config in the MVP: endpoint URL, optional secret
handling policy, polling interval, and UI preferences. Secret persistence must
be explicit; if secure storage is not approved, keep secrets session-only or
require a documented plaintext opt-in. Do not add database storage unless
explicitly approved.
