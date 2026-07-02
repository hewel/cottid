# Manual QA

Run against an existing aria2 RPC server.

- Valid connection: save endpoint, test connection, and confirm stats/list refresh.
- Failed connection: stop aria2 or use a bad endpoint and confirm display-safe failure.
- Add URI: submit one HTTP(S) URI and confirm pending, success, and refreshed list.
- Add magnet: submit one magnet link and confirm pending, success, and refreshed list.
- Actions: pause, unpause, remove one row, and confirm pending state plus refresh.
- Clear results: complete or stop a download, clear stopped results, and confirm refresh.
- Recovery: stop the daemon after a successful refresh and confirm stale snapshot remains.
- Slow refresh: make aria2 slow or unreachable and confirm repeated refresh
  ticks do not pile up while one refresh is still running.
- Many stopped downloads: confirm the UI remains responsive and shows only the
  latest bounded stopped/history page during normal refresh.
- Active to stopped: let a selected active download complete and confirm the row
  moves sections without losing unrelated UI state.
- Secret storage: use a token, save/test connection, restart Cottid, and confirm
  the token is restored without appearing in the config file when keyring
  storage is available.
- Plaintext fallback: simulate unavailable keyring storage if possible and
  confirm the settings panel requires choosing plaintext fallback or
  session-only token use.
- Secret redaction: use a token and confirm UI/loggable debug text does not
  expose it.
