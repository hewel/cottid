# MVP And Future Features

MVP scope: manage the approved local `aria2c` child or connect/test an existing
RPC endpoint, fetch global stats, fetch active/waiting/stopped downloads, add
URI or magnet, pause/unpause/remove downloads, purge stopped results, show a
basic detail panel, and persist basic config subject to dependency approval.

Keep these out of MVP unless explicitly approved: torrent-file upload, Metalink
upload, torrent file selection, peer/BitTorrent details, queue reordering, speed
profiles, browser extension integration, multi-server profiles, and system tray.

WebSocket work belongs in `aria2::websocket`; managed local process lifecycle
belongs in top-level `daemon/`. Add advanced features by extending domain models
and app messages, not by coupling UI to raw RPC shapes.
