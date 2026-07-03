# Modeled runtime global options

Cottid may edit a small allowlist of live aria2 runtime global options through
`aria2.getGlobalOption` and `aria2.changeGlobalOption`, starting with `dir`,
`max-concurrent-downloads`, `max-overall-download-limit`, and
`max-overall-upload-limit`. This gives users practical download-manager controls
without making Cottid an aria2.conf editor, daemon session administrator, or raw
advanced option editor.
