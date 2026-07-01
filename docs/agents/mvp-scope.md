# MVP And Future Features

MVP scope: connect/test an existing RPC endpoint, fetch global stats, fetch
active/waiting/stopped downloads, add URI or magnet, pause/unpause/remove
downloads, purge stopped results, show a basic detail panel, and persist basic
config subject to dependency approval.

Keep these out of MVP unless explicitly approved: WebSocket notifications,
managed `aria2c` lifecycle, torrent-file upload, Metalink upload, torrent file
selection, peer/BitTorrent details, scheduler, queue reordering, speed profiles,
browser extension integration, multi-server profiles, and system tray.

Future WebSocket work belongs in `aria2::websocket`; future local process
management belongs in `aria2::daemon`. Add advanced features by extending domain
models and app messages, not by coupling UI to raw RPC shapes.
