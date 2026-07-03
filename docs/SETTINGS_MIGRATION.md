# AriaNg Settings Migration Decisions

This document analyzes AriaNg's settings surface and records which settings
should migrate into Cottid. It is a decision aid, not an implementation spec.

Cottid is a desktop `iced` frontend for an external aria2 JSON-RPC daemon. It
should not copy AriaNg's browser application model or become a full aria2.conf
editor by default.

## Source Summary

Inspected AriaNg sources:

- `src/scripts/controllers/settings-ariang.js`
- `src/scripts/controllers/settings-aria2.js`
- `src/scripts/services/ariaNgSettingService.js`
- `src/scripts/services/aria2SettingService.js`
- `src/scripts/config/constants.js`
- `src/scripts/config/aria2Options.js`
- `src/views/settings-ariang.html`
- `src/views/settings-aria2.html`
- `src/views/new.html`
- `src/scripts/directives/setting.js`
- `src/views/setting.html`

Discovered setting groups:

- AriaNg app settings: frontend-only preferences, browser behavior, task-list
  behavior, import/export, debug/session flags.
- RPC connection settings: host, port, protocol, HTTP method, headers, secret,
  alias, and extra RPC profiles.
- aria2 global settings: options exposed through `aria2.getGlobalOption` and
  `aria2.changeGlobalOption`.
- aria2 task/new-download settings: options supplied when adding or changing a
  task.
- Session/local-only settings: debug mode, input history, notification history,
  selected/local UI state.
- UI-only behavior settings: list sorting, after-create navigation, copy
  behavior, browser notifications, gestures.
- Browser, legacy, obscure, or daemon-owned settings that should not be copied
  into Cottid without a concrete product reason.

## Migration Decision Table

| Source Area | Original Key | Category | Meaning | Default Value | Option Type | Recommended Migration Priority | Reason | Target UI Suggestion | Target Config Location | Dependencies / Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| RPC setting | `rpcHost`, `rpcPort`, `rpcInterface`, `protocol` | RPC | Build the aria2 JSON-RPC endpoint. | host `""`, port `6800`, interface `jsonrpc`, protocol `http` | string/enum | Must migrate | Core connection setup, but Cottid should collapse this into one endpoint URL. | URL input | `rpc.profiles` | Keep HTTP(S) endpoint validation; WebSocket is future-only. |
| RPC setting | `secret` | RPC | aria2 RPC token. | `""` | secret | Must migrate | Required for secured daemons. | password input or session prompt | `rpc.profiles` | Store in keyring or session only; never log/export in plain text. AriaNg base64 is not security. |
| RPC setting | `rpcAlias` | RPC | Friendly profile name. | `""` | string | Should migrate | Useful once profiles exist; not required for single-profile MVP. | input | `rpc.profiles` | Can default to endpoint host. |
| RPC setting | `extendRpcServers` | RPC | Additional RPC profiles. | `[]` | list/map | Optional | Valuable for power users, but profile switching is not MVP. | profile manager | `rpc.profiles` | Requires profile selection, per-profile secrets, and config migration. |
| RPC setting | `httpMethod` | RPC | JSON-RPC request method, GET or POST. | `POST` | enum | Do not migrate | Cottid uses POST; GET adds security and caching risks. | not applicable | not applicable | Keep POST only. |
| RPC setting | `rpcRequestHeaders` | RPC | Custom headers for RPC requests. | `""` | map/list | Optional | Useful behind reverse proxies, but can leak credentials. | advanced textarea | `rpc.profiles` | Redact in logs/export; validate header syntax. |
| AriaNg app setting | `webSocketReconnectInterval` | RPC/UI | Browser WebSocket reconnect delay. | `5000` ms | duration | Skip for now | Cottid does not use WebSocket RPC in MVP. | hidden config only | not applicable | Reconsider if WebSocket invalidation is added. |
| AriaNg app setting | `globalStatRefreshInterval` | Refresh | Global speed/stat polling interval. | `1000` ms | duration | Must migrate | Cottid needs refresh cadence. | number input/stepper | `app.settings` | Integrate with central scheduler and backoff. |
| AriaNg app setting | `downloadTaskRefreshInterval` | Refresh | Task list polling interval. | `1000` ms | duration | Must migrate | Core list freshness setting. | number input/stepper | `app.settings` | Scheduler must prevent concurrent refreshes and discard stale responses. |
| AriaNg app setting | `titleRefreshInterval` | UI | Browser tab title update interval. | `5000` ms | duration | Do not migrate | Browser-tab behavior, not a desktop settings concern. | not applicable | not applicable | Desktop title can be derived internally if needed. |
| AriaNg app setting | `language` | UI | UI language selection. | `en` | enum | Skip for now | Requires an i18n system. | select | `ui.preferences` | Revisit when localization is real. |
| AriaNg app setting | `theme` | UI | Light/dark AriaNg theme. | `light` | enum | Do not migrate | Cottid should use local design tokens and system-aware theme architecture, not copy AriaNg theme settings. | not applicable | not applicable | Keep theme separate from this migration. |
| AriaNg app setting | `debugMode` | Session | Enables debug output. | `false` session-only | boolean | Optional | Useful for diagnostics, but not a normal user setting. | hidden config or diagnostics toggle | `app.session` | Must redact secrets. |
| AriaNg app setting | `browserNotification` | Notification | Browser notification enablement. | `false` | boolean | Do not migrate | Browser API-specific. | not applicable | not applicable | Replace with native desktop notifications only if approved. |
| AriaNg app setting | `browserNotificationSound` | Notification | Plays sound for browser notifications. | `true` | boolean | Do not migrate | Browser/UI-specific. | not applicable | not applicable | Native notification policy should be separate. |
| AriaNg app setting | `browserNotificationFrequency` | Notification | Limits browser notification frequency. | `unlimited` | enum | Do not migrate | Browser-notification-specific. | not applicable | not applicable | Native notification throttling can be designed later. |
| AriaNg app setting | `keyboardShortcuts` | UI | Enables AriaNg shortcuts. | `true` | boolean | Optional | Good desktop feature, but not required before shortcuts exist. | checkbox | `ui.preferences` | Needs shortcut map and accessibility review. |
| AriaNg app setting | `swipeGesture` | UI | Mobile swipe gestures. | `true` | boolean | Do not migrate | Mobile/browser-specific. | not applicable | not applicable | Not applicable to desktop `iced` MVP. |
| AriaNg app setting | `dragAndDropTasks` | Task List | Drag-sort task order. | `true` | boolean | Skip for now | Queue ordering is future scope. | checkbox | `ui.preferences` | Requires aria2 queue-position operations. |
| AriaNg app setting | `rpcListDisplayOrder` | RPC/UI | Sorts RPC profile list. | `recentlyUsed` | enum | Skip for now | Only matters after multi-profile support. | select | `ui.preferences` | Depends on profile manager. |
| AriaNg app setting | `afterCreatingNewTask` | Task Behavior | Destination after adding a task. | `task-list` | enum | Do not migrate | SPA navigation behavior does not map cleanly to desktop. | not applicable | not applicable | Cottid can select or flash the created item without a setting. |
| AriaNg app setting | `afterRetryingTask` | Task Behavior | Destination after retrying a task. | `task-list-downloading` | enum | Do not migrate | AriaNg route behavior. | not applicable | not applicable | Prefer deterministic desktop behavior. |
| AriaNg app setting | `removeOldTaskAfterRetrying` | Task Behavior | Removes old stopped task after retry. | `false` | boolean | Optional | Potentially useful but risky. | checkbox | `ui.preferences` | Requires retry workflow and clear undo/error behavior. |
| AriaNg app setting | `confirmTaskRemoval` | Task Behavior | Confirms before removing tasks. | `true` | boolean | Should migrate | Prevents destructive mistakes. | checkbox | `ui.preferences` | Default should remain confirm-on. |
| AriaNg app setting | `includePrefixWhenCopyingFromTaskDetails` | UI | Include labels when copying task details. | `true` | boolean | Skip for now | Copy affordance is non-core. | checkbox | `ui.preferences` | Revisit with task detail copy actions. |
| AriaNg app setting | `showPiecesInfoInTaskDetailPage` | Task Detail | Controls piece bitmap display threshold. | `le10240` | enum | Optional | Useful only for torrent detail diagnostics. | select | `ui.preferences` | Requires torrent piece model and performance guard. |
| AriaNg app setting | `taskListIndependentDisplayOrder`, `displayOrder`, `waitingTaskListPageDisplayOrder`, `stoppedTaskListPageDisplayOrder` | Task List | Sort order preferences. | `false`, `default:asc` | enum | Should migrate | Users expect stable sorting eventually; not required for first list MVP. | sort menu | `ui.preferences` | Keep domain ordering separate from UI sort state. |
| AriaNg app setting | `fileListDisplayOrder`, `peerListDisplayOrder` | Detail | Sorts file and peer lists. | `default:asc` | enum | Optional | Useful only after file/peer detail views exist. | sort menu | `ui.preferences` | Depends on selected-download detail model. |
| Local/session setting | setting history keys | Session | Remembers previous input values such as paths. | max `10` | list | Optional | Nice quality-of-life for path/header inputs. | recent-values menu | `app.session` or `ui.preferences` | Avoid storing secrets or auth headers. |
| AriaNg app action | import/export settings | App | Imports/exports AriaNg settings JSON. | n/a | action | Skip for now | Useful later, but config schema should stabilize first. | command palette action | `app.settings` | Must omit or redact secrets by default. |
| aria2 global option | `dir` | Basic | Default download directory. | aria2-defined / required in AriaNg metadata | path | Must migrate | Most important aria2 behavior setting for users. | path input | `aria2.global`, `aria2.taskDefaults` | Uses `getGlobalOption`/`changeGlobalOption`; path is daemon-local, not desktop-local. |
| aria2 global option | `max-concurrent-downloads` | Basic | Maximum active downloads. | `5` | number | Should migrate | Common queue control. | stepper/input | `aria2.global` | Direct aria2 global option. |
| aria2 global option | `continue` | Basic | Continue partially downloaded files. | aria2-defined | boolean | Should migrate | Common expectation for resumable downloads. | checkbox | `aria2.global`, `aria2.taskDefaults` | Also valid as task/new-download option. |
| aria2 global option | `check-integrity` | Basic | Hash-check downloaded files when possible. | `false` | boolean | Optional | Useful but can surprise users with extra work. | checkbox | `aria2.global`, `aria2.taskDefaults` | Also task option. |
| aria2 global option | `log` | Basic | aria2 daemon log path. | aria2-defined | path | Skip for now | Daemon-side operational setting, not frontend core. | advanced input | `aria2.global` | Path is daemon-local and may be host-specific. |
| aria2 quick/global option | `max-overall-download-limit` | Advanced / Quick | Global download speed limit. | `0` | bandwidth | Should migrate | Core download-manager control. | bandwidth input / quick limiter | `aria2.global` | Direct `changeGlobalOption`; `0` means unlimited. |
| aria2 quick/global option | `max-overall-upload-limit` | BitTorrent / Quick | Global upload speed limit. | `0` | bandwidth | Should migrate | Important for torrents and shared networks. | bandwidth input / quick limiter | `aria2.global` | Direct `changeGlobalOption`; `0` means unlimited. |
| aria2 task option | `out` | HTTP task | Output filename for new HTTP/FTP download. | unknown | string | Should migrate | Common per-download customization. | input | `aria2.taskDefaults` | Send with add URI; new-task only. |
| aria2 task option | `max-download-limit` | Task | Per-task download speed limit. | `0` | bandwidth | Should migrate | Expected per-download control. | bandwidth input | `aria2.taskDefaults` | Can be changed on existing tasks. |
| aria2 task option | `max-upload-limit` | BitTorrent task | Per-task upload speed limit. | `0` | bandwidth | Should migrate | Expected for torrent users. | bandwidth input | `aria2.taskDefaults` | Applies to BitTorrent. |
| aria2 HTTP/task options | `split`, `min-split-size`, `max-connection-per-server` | HTTP task | Connection splitting controls. | `5`, `20M`, `1` | number/size | Optional | Power-user performance tuning. | advanced inputs | `aria2.taskDefaults` | Can affect server load and reliability. |
| aria2 HTTP/task options | `lowest-speed-limit`, `timeout`, `connect-timeout`, `retry-wait`, `max-tries` | HTTP | Retry/timeout behavior. | `0`, `60`, `60`, `0`, `5` | bandwidth/duration/number | Optional | Useful advanced network tuning, not first-run required. | advanced inputs | `aria2.global`, `aria2.taskDefaults` | Direct aria2 options. |
| aria2 HTTP/task options | `header`, `referer`, `user-agent` | HTTP | Custom request metadata. | user-agent `aria2/$VERSION` | list/string | Optional | Needed for some sites; easy to misuse. | advanced textarea | `aria2.taskDefaults` | Do not store sensitive headers in history/export. |
| aria2 HTTP/task options | `http-user`, `http-passwd` | HTTP | HTTP basic auth credentials. | unknown | secret/string | Optional | Needed occasionally, but security-sensitive. | advanced credential fields | `aria2.taskDefaults` | Store carefully; avoid global persistence by default. |
| aria2 proxy options | `all-proxy`, `http-proxy`, `https-proxy`, `ftp-proxy` | HTTP/FTP/SFTP | Proxy URLs. | unknown | string | Optional | Useful for some environments, but not core. | advanced input | `aria2.global`, `aria2.taskDefaults` | Consider per-profile vs daemon-global ownership. |
| aria2 proxy credential options | `all-proxy-user`, `all-proxy-passwd`, `http-proxy-user`, `http-proxy-passwd`, `https-proxy-user`, `https-proxy-passwd`, `ftp-proxy-user`, `ftp-proxy-passwd` | HTTP/FTP/SFTP | Proxy credentials. | unknown | secret/string | Skip for now | Security-heavy and uncommon for MVP. | hidden advanced editor | `aria2.global`, `aria2.taskDefaults` | Never include in logs/history/export. |
| aria2 HTTP option | `checksum` | HTTP task | Expected checksum for verification. | unknown | string | Optional | Useful but specialized. | input | `aria2.taskDefaults` | Validate `algorithm=value` shape. |
| aria2 HTTP options | `dry-run`, `remote-time`, `reuse-uri`, `uri-selector`, `stream-piece-selector`, `http-accept-gzip`, `http-auth-challenge`, `http-no-cache`, `enable-http-keep-alive`, `enable-http-pipelining`, `use-head`, `save-cookies`, `no-proxy`, `proxy-method` | HTTP | Assorted protocol behavior. | mixed | mixed | Skip for now | Too detailed for normal settings; can live in advanced editor later. | advanced editor | `aria2.global` | Expose only if real users need it. |
| aria2 FTP/SFTP options | `ftp-user`, `ftp-passwd`, `ftp-pasv`, `ftp-type`, `ftp-reuse-connection`, `ssh-host-key-md` | FTP/SFTP | FTP/SFTP login and transfer behavior. | mixed | mixed | Skip for now | FTP/SFTP is not the likely first migration target. | advanced editor | `aria2.global`, `aria2.taskDefaults` | Credentials are sensitive. |
| aria2 BitTorrent option | `follow-torrent` | BitTorrent | Whether torrent files are followed. | `true` | enum | Should migrate | Important if adding torrent files. | select | `aria2.global` | Direct global option. |
| aria2 BitTorrent option | `pause-metadata` | BitTorrent/RPC | Pause magnet metadata downloads. | `false` | boolean | Optional | Useful magnet behavior control. | checkbox | `aria2.global`, `aria2.taskDefaults` | Appears in RPC category and task options. |
| aria2 BitTorrent options | `seed-ratio`, `seed-time` | BitTorrent task | Stop seeding by ratio/time. | `1.0`, aria2-defined | number/duration | Should migrate | Torrent users expect seeding controls. | inputs | `aria2.global`, `aria2.taskDefaults` | Needs clear unlimited/zero semantics. |
| aria2 BitTorrent options | `bt-max-peers`, `bt-request-peer-speed-limit`, `bt-remove-unselected-file`, `bt-stop-timeout`, `bt-tracker` | BitTorrent task | Peer, tracker, and cleanup behavior. | mixed | mixed | Optional | Useful but advanced. | advanced inputs/textarea | `aria2.taskDefaults` | Some can be updated on waiting/paused tasks. |
| aria2 BitTorrent options | `bt-enable-lpd`, `bt-force-encryption`, `bt-require-crypto`, `bt-min-crypto-level`, `bt-save-metadata`, `bt-metadata-only`, `bt-load-saved-metadata`, `bt-exclude-tracker`, `bt-external-ip`, `bt-hash-check-seed`, `enable-peer-exchange` | BitTorrent | Advanced torrent behavior. | mixed | mixed | Skip for now | Too much surface for first settings model. | advanced editor | `aria2.global` | Revisit with torrent-focused release. |
| aria2 BitTorrent read-only/daemon options | `dht-file-path`, `dht-file-path6`, `dht-listen-port`, `dht-message-timeout`, `enable-dht`, `enable-dht6`, `listen-port`, `peer-id-prefix`, `peer-agent`, `bt-detach-seed-only` | BitTorrent | Daemon/network identity and DHT settings. | mixed | mixed | Do not migrate | Mostly daemon startup or read-only options. | diagnostics only | not applicable | External daemon config owns these. |
| aria2 Metalink options | `follow-metalink`, `metalink-base-uri`, `metalink-language`, `metalink-location`, `metalink-os`, `metalink-version`, `metalink-preferred-protocol`, `metalink-enable-unique-protocol` | Metalink | Metalink handling and filtering. | mixed | mixed | Skip for now | Lower-priority protocol surface. | advanced editor | `aria2.global`, `aria2.taskDefaults` | Revisit only if Metalink support is a product goal. |
| aria2 RPC global options | `enable-rpc`, `rpc-allow-origin-all`, `rpc-listen-all`, `rpc-listen-port`, `rpc-max-request-size`, `rpc-secure` | RPC | aria2 daemon RPC server settings. | mixed/read-only | mixed | Do not migrate | Cottid connects to an existing daemon; it should not manage daemon startup flags. | diagnostics only | not applicable | May be displayed read-only from `getGlobalOption`. |
| aria2 RPC global option | `rpc-save-upload-metadata` | RPC | Save uploaded torrent/metalink metadata. | `true` | boolean | Optional | Niche but writable. | advanced checkbox | `aria2.global` | Only if upload/torrent workflows justify it. |
| aria2 advanced option | `allow-overwrite`, `auto-file-renaming`, `file-allocation` | Advanced/task | File conflict and allocation behavior. | `false`, `true`, `prealloc` | boolean/enum | Should migrate | Directly affects user-visible file outcomes. | checkboxes/select | `aria2.global`, `aria2.taskDefaults` | Explain overwrite risk clearly. |
| aria2 advanced option | `conditional-get`, `parameterized-uri`, `force-save` | Advanced/task | Specialized add/save behavior. | `false`, `false`, `false` | boolean | Optional | Useful for advanced download creation. | advanced checkboxes | `aria2.taskDefaults` | Present only in advanced task options. |
| aria2 advanced option | `save-session`, `save-session-interval` | Session | aria2 daemon session persistence. | `""`, `0` | path/duration | Optional | Important for daemon persistence, but owned by daemon config. | advanced input/diagnostics | `aria2.global` | Path is daemon-local; interval is read-only in AriaNg metadata. |
| aria2 advanced option | `max-download-result`, `keep-unfinished-download-result`, `download-result` | Session/history | aria2 result/history retention. | `1000`, `true`, `default` | number/boolean/enum | Optional | Affects stopped/history list behavior. | advanced inputs | `aria2.global` | Align with Cottid stopped-row retention model. |
| aria2 advanced option | `always-resume`, `allow-piece-length-change`, `max-resume-failure-tries`, `remove-control-file`, `realtime-chunk-checksum`, `hash-check-only` | Advanced | Resume/checksum/control-file behavior. | mixed | mixed | Skip for now | Powerful but obscure; risk of confusing users. | advanced editor | `aria2.global` | Add only after user demand. |
| aria2 daemon/console/system options | `daemon`, `conf-path`, `console-log-level`, `log-level`, `enable-color`, `show-console-readout`, `summary-interval`, `quiet`, `truncate-console-readout`, `no-conf`, `event-poll`, `rlimit-nofile`, `dscp`, `async-dns`, `disable-ipv6`, `disk-cache`, `enable-mmap`, `max-mmap-limit`, `min-tls-version`, `socket-recv-buffer-size`, `stop`, `auto-save-interval`, `deferred-input`, `human-readable`, `content-disposition-default-utf8`, `save-not-found`, `no-file-allocation-limit`, `optimize-concurrent-downloads`, `piece-length` | Advanced | Daemon startup, system, and console tuning. | mixed | mixed | Do not migrate | Mostly daemon-owned, read-only, obscure, or operational. | diagnostics only or not applicable | not applicable | Do not turn Cottid into an aria2.conf replacement. |

## Recommended Migration Strategy

### 1. Minimal viable settings

Implement only the settings needed to connect, refresh, and create usable
downloads:

- RPC endpoint URL derived from AriaNg's `rpcHost`, `rpcPort`,
  `rpcInterface`, and `protocol`, stored as one `rpc.profiles` endpoint.
- RPC secret as secure or session auth, not AriaNg-style base64 config.
- Polling intervals mapped from `globalStatRefreshInterval` and
  `downloadTaskRefreshInterval`, wired through the central refresh scheduler.
- Default download directory `dir`.
- Basic new-task options: `dir`, `out`, `max-download-limit`.
- Confirmation preference for destructive removal: `confirmTaskRemoval`.

### 2. Practical default settings

Add settings most real users expect after MVP:

- Global speed limits: `max-overall-download-limit`,
  `max-overall-upload-limit`.
- Queue/concurrency: `max-concurrent-downloads`.
- Resume/file behavior: `continue`, `allow-overwrite`,
  `auto-file-renaming`, `file-allocation`.
- Per-task upload limit for torrents: `max-upload-limit`.
- Torrent basics: `follow-torrent`, `seed-ratio`, `seed-time`.
- Sort preferences for task lists once sorting exists.

### 3. Advanced settings

Hide these behind an advanced mode or advanced task editor:

- HTTP performance: `split`, `min-split-size`, `max-connection-per-server`,
  timeout/retry options.
- Request metadata: `header`, `referer`, `user-agent`, `checksum`.
- Proxy options, with special care for proxy credentials.
- Torrent peer/tracker controls.
- aria2 history/session retention: `max-download-result`, `download-result`,
  `save-session`.
- Optional raw aria2 global editor for selected writable options only.

### 4. Settings to skip

Do not migrate unless there is a strong product reason:

- AriaNg browser/UI settings: `theme`, `title`, `titleRefreshInterval`,
  `browserNotification*`, `swipeGesture`, route-after-action settings.
- Browser transport settings: `httpMethod=GET`, WebSocket reconnect interval.
- aria2 daemon startup/RPC server settings: `enable-rpc`,
  `rpc-listen-port`, `rpc-listen-all`, `rpc-secure`, `daemon`, `conf-path`.
- Console/system tuning and read-only daemon internals.
- Metalink and FTP/SFTP settings until those protocols become deliberate
  feature targets.

## Migration Selection

### Must migrate

- [ ] `rpcHost` / `rpcPort` / `rpcInterface` / `protocol` - core connection
  setup, represented as a single endpoint URL.
- [ ] `secret` - needed for secured aria2 daemons, with secure/session storage.
- [ ] `globalStatRefreshInterval` - global stats refresh cadence.
- [ ] `downloadTaskRefreshInterval` - task list refresh cadence.
- [ ] `dir` - default daemon-side download directory.

### Should migrate

- [ ] `confirmTaskRemoval` - protects destructive task actions.
- [ ] `max-concurrent-downloads` - common queue control.
- [ ] `max-overall-download-limit` - global download speed limit.
- [ ] `max-overall-upload-limit` - global upload speed limit.
- [ ] `out` - per-download output filename.
- [ ] `max-download-limit` - per-task download speed limit.
- [ ] `max-upload-limit` - per-task torrent upload speed limit.
- [ ] `continue` - resumable download behavior.
- [ ] `follow-torrent` - expected torrent-file behavior.
- [ ] `seed-ratio` - common torrent seeding limit.
- [ ] `seed-time` - common torrent seeding duration.
- [ ] `allow-overwrite` - visible file conflict behavior.
- [ ] `auto-file-renaming` - visible file conflict behavior.
- [ ] `file-allocation` - visible disk behavior.
- [ ] task sort order settings - useful once sorting exists.

### Optional

- [ ] `rpcAlias` - useful with multiple profiles.
- [ ] `extendRpcServers` - multi-daemon profile support.
- [ ] `rpcRequestHeaders` - advanced reverse-proxy support.
- [ ] `debugMode` - diagnostics only.
- [ ] `keyboardShortcuts` - only after shortcuts exist.
- [ ] setting history - useful for paths/headers, never for secrets.
- [ ] `showPiecesInfoInTaskDetailPage` - torrent detail diagnostics.
- [ ] `fileListDisplayOrder` - after file detail view exists.
- [ ] `peerListDisplayOrder` - after peer detail view exists.
- [ ] `check-integrity` - optional verification behavior.
- [ ] `split` - advanced HTTP performance tuning.
- [ ] `min-split-size` - advanced HTTP performance tuning.
- [ ] `max-connection-per-server` - advanced HTTP performance tuning.
- [ ] timeout/retry options - advanced network tuning.
- [ ] `header` - advanced HTTP request metadata.
- [ ] `referer` - advanced HTTP request metadata.
- [ ] `user-agent` - advanced HTTP request metadata.
- [ ] `http-user` / `http-passwd` - per-download HTTP credentials.
- [ ] proxy options - advanced network configuration.
- [ ] `pause-metadata` - magnet/torrent behavior.
- [ ] `bt-max-peers` - torrent tuning.
- [ ] `bt-request-peer-speed-limit` - torrent tuning.
- [ ] `bt-remove-unselected-file` - torrent cleanup behavior.
- [ ] `bt-stop-timeout` - torrent seeding behavior.
- [ ] `bt-tracker` - torrent tracker override.
- [ ] `conditional-get` - advanced HTTP behavior.
- [ ] `parameterized-uri` - advanced URI behavior.
- [ ] `force-save` - advanced save behavior.
- [ ] `save-session` - daemon session persistence.
- [ ] `max-download-result` / `download-result` - daemon result retention.

### Skip for now

- [ ] `webSocketReconnectInterval` - no WebSocket RPC in MVP.
- [ ] `language` - requires localization system.
- [ ] `dragAndDropTasks` - queue reordering is future scope.
- [ ] `rpcListDisplayOrder` - only matters after multiple profiles.
- [ ] `includePrefixWhenCopyingFromTaskDetails` - copy UX detail.
- [ ] import/export settings - wait for stable config schema.
- [ ] proxy credential options - security-sensitive and uncommon.
- [ ] HTTP protocol toggles - too detailed for normal settings.
- [ ] FTP/SFTP options - lower-priority protocol surface.
- [ ] advanced BitTorrent options - torrent-focused future work.
- [ ] Metalink options - lower-priority protocol surface.
- [ ] resume/checksum/control-file advanced options - obscure.

### Do not migrate

- [ ] `theme` - current project should not copy AriaNg theme settings.
- [ ] `title` / `titleRefreshInterval` - browser tab behavior.
- [ ] `browserNotification` / `browserNotificationSound` /
  `browserNotificationFrequency` - browser API-specific.
- [ ] `swipeGesture` - mobile/browser-specific.
- [ ] `afterCreatingNewTask` - SPA route behavior.
- [ ] `afterRetryingTask` - SPA route behavior.
- [ ] `httpMethod` - keep JSON-RPC POST only.
- [ ] `enable-rpc` / `rpc-listen-port` / `rpc-listen-all` / `rpc-secure` -
  daemon-owned RPC server config.
- [ ] `daemon` / `conf-path` / console/system options - daemon startup or
  console behavior.
- [ ] read-only DHT/listen/peer identity options - diagnostics only, not
  frontend settings.

## Open Questions Before Code Implementation

1. Should v1 support one RPC profile only, or include a full multi-profile
   manager?
2. Should RPC secrets be session-only by default, keyring-persisted by default,
   or user-selectable?
3. Should Cottid directly edit aria2 global options in v1, or only store task
   defaults and leave daemon config alone?
4. Should the new-task dialog expose advanced per-task options immediately, or
   start with only directory, filename, and speed limits?
5. Should native desktop notifications replace AriaNg browser notifications, or
   stay out of scope for now?
6. Should daemon session controls like `save-session` and `saveSession` action
   be exposed, or treated as external daemon administration?
7. Should there eventually be a raw advanced aria2 option editor, or should
   every supported option be deliberately modeled?
