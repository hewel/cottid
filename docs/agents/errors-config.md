# Errors And Config

Use typed error categories and keep diagnostic detail available without exposing
secrets. Show concise user-facing errors. Preserve the last known good download
snapshot when refresh fails. Attach command errors to the affected download
where possible; show connection/config errors in status or settings UI.

Persist only basic local config in the MVP: endpoint URL, optional secret
handling policy, polling interval, and UI preferences. Store RPC tokens in the
OS keyring when available. If keyring storage fails, require an explicit
plaintext fallback confirmation or keep the token session-only. Do not add
database storage unless explicitly approved.
