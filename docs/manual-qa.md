# Manual QA

Run against an existing aria2 RPC server.

- Valid connection: save endpoint, test connection, and confirm stats/list refresh.
- Failed connection: stop aria2 or use a bad endpoint and confirm display-safe failure.
- Add URI: submit one HTTP(S) URI and confirm pending, success, and refreshed list.
- Add magnet: submit one magnet link and confirm pending, success, and refreshed list.
- Actions: pause, unpause, remove one row, and confirm pending state plus refresh.
- Purge: complete or stop a download, purge stopped results, and confirm refresh.
- Recovery: stop the daemon after a successful refresh and confirm stale snapshot remains.
- Secret redaction: use a session token and confirm UI/loggable debug text does not expose it.
