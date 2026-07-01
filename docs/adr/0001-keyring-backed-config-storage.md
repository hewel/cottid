# Keyring-backed config storage

Cottid persists local settings as TOML at the existing config path and stores
aria2 RPC tokens in the OS keyring, bound to the exact RPC endpoint URL. If the
keyring is unavailable, the user must explicitly choose plaintext fallback or
keep the token session-only; this preserves the approved convenience path
without making plaintext secret storage the default.
