#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    version: String,
    enabled_features: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Gid(String);

impl Gid {
    pub fn new(value: impl Into<String>) -> Result<Self, DomainError> {
        let value = value.into();

        if value.trim().is_empty() {
            return Err(DomainError::EmptyGid);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    EmptyGid,
}

impl DomainError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::EmptyGid => "gid must not be empty",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadStatus {
    Active,
    Waiting,
    Paused,
    Complete,
    Error,
    Removed,
    Unknown(String),
}

impl DownloadStatus {
    pub fn from_aria2(value: impl Into<String>) -> Self {
        let value = value.into();

        match value.as_str() {
            "active" => Self::Active,
            "waiting" => Self::Waiting,
            "paused" => Self::Paused,
            "complete" => Self::Complete,
            "error" => Self::Error,
            "removed" => Self::Removed,
            _ => Self::Unknown(value),
        }
    }

    pub fn display_label(&self) -> &str {
        match self {
            Self::Active => "Active",
            Self::Waiting => "Waiting",
            Self::Paused => "Paused",
            Self::Complete => "Complete",
            Self::Error => "Error",
            Self::Removed => "Removed",
            Self::Unknown(value) => value.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadFile {
    path: String,
    length_bytes: u64,
    completed_length_bytes: u64,
    selected: bool,
}

impl DownloadFile {
    pub fn new(
        path: impl Into<String>,
        length_bytes: u64,
        completed_length_bytes: u64,
        selected: bool,
    ) -> Self {
        Self {
            path: path.into(),
            length_bytes,
            completed_length_bytes,
            selected,
        }
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn length_bytes(&self) -> u64 {
        self.length_bytes
    }

    pub fn completed_length_bytes(&self) -> u64 {
        self.completed_length_bytes
    }

    pub fn selected(&self) -> bool {
        self.selected
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorrentDetail {
    info_hash: Option<String>,
    seeder: bool,
    num_seeders: u32,
}

impl TorrentDetail {
    pub fn new(info_hash: Option<String>, seeder: bool, num_seeders: u32) -> Self {
        Self {
            info_hash,
            seeder,
            num_seeders,
        }
    }

    pub fn info_hash(&self) -> Option<&str> {
        self.info_hash.as_deref()
    }

    pub fn seeder(&self) -> bool {
        self.seeder
    }

    pub fn num_seeders(&self) -> u32 {
        self.num_seeders
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadItem {
    gid: Gid,
    status: DownloadStatus,
    total_length_bytes: u64,
    completed_length_bytes: u64,
    download_speed_bytes_per_second: u64,
    upload_speed_bytes_per_second: u64,
    files: Vec<DownloadFile>,
    command_error: Option<String>,
}

impl DownloadItem {
    pub fn new(
        gid: Gid,
        status: DownloadStatus,
        total_length_bytes: u64,
        completed_length_bytes: u64,
        download_speed_bytes_per_second: u64,
        upload_speed_bytes_per_second: u64,
        files: Vec<DownloadFile>,
    ) -> Self {
        Self {
            gid,
            status,
            total_length_bytes,
            completed_length_bytes,
            download_speed_bytes_per_second,
            upload_speed_bytes_per_second,
            files,
            command_error: None,
        }
    }

    pub fn gid(&self) -> &Gid {
        &self.gid
    }

    pub fn status(&self) -> &DownloadStatus {
        &self.status
    }

    pub fn total_length_bytes(&self) -> u64 {
        self.total_length_bytes
    }

    pub fn completed_length_bytes(&self) -> u64 {
        self.completed_length_bytes
    }

    pub fn download_speed_bytes_per_second(&self) -> u64 {
        self.download_speed_bytes_per_second
    }

    pub fn upload_speed_bytes_per_second(&self) -> u64 {
        self.upload_speed_bytes_per_second
    }

    pub fn files(&self) -> &[DownloadFile] {
        &self.files
    }

    pub fn command_error(&self) -> Option<&str> {
        self.command_error.as_deref()
    }

    pub fn set_command_error(&mut self, error: Option<String>) {
        self.command_error = error;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadDetail {
    item: DownloadItem,
    directory: Option<String>,
    connections: u32,
    piece_length_bytes: u64,
    piece_count: u64,
    error_code: Option<String>,
    error_message: Option<String>,
    torrent: Option<TorrentDetail>,
}

impl DownloadDetail {
    pub fn new(item: DownloadItem) -> Self {
        Self {
            item,
            directory: None,
            connections: 0,
            piece_length_bytes: 0,
            piece_count: 0,
            error_code: None,
            error_message: None,
            torrent: None,
        }
    }

    pub fn item(&self) -> &DownloadItem {
        &self.item
    }

    pub fn directory(&self) -> Option<&str> {
        self.directory.as_deref()
    }

    pub fn set_directory(&mut self, directory: Option<String>) {
        self.directory = directory.filter(|value| !value.is_empty());
    }

    pub fn connections(&self) -> u32 {
        self.connections
    }

    pub fn set_connections(&mut self, connections: u32) {
        self.connections = connections;
    }

    pub fn piece_length_bytes(&self) -> u64 {
        self.piece_length_bytes
    }

    pub fn set_piece_length_bytes(&mut self, piece_length_bytes: u64) {
        self.piece_length_bytes = piece_length_bytes;
    }

    pub fn piece_count(&self) -> u64 {
        self.piece_count
    }

    pub fn set_piece_count(&mut self, piece_count: u64) {
        self.piece_count = piece_count;
    }

    pub fn error_code(&self) -> Option<&str> {
        self.error_code.as_deref()
    }

    pub fn set_error_code(&mut self, error_code: Option<String>) {
        self.error_code = error_code.filter(|value| !value.is_empty());
    }

    pub fn error_message(&self) -> Option<&str> {
        self.error_message.as_deref()
    }

    pub fn set_error_message(&mut self, error_message: Option<String>) {
        self.error_message = error_message.filter(|value| !value.is_empty());
    }

    pub fn torrent(&self) -> Option<&TorrentDetail> {
        self.torrent.as_ref()
    }

    pub fn set_torrent(&mut self, torrent: Option<TorrentDetail>) {
        self.torrent = torrent;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadSnapshot {
    global_stats: GlobalStats,
    items: Vec<DownloadItem>,
    selected_detail: Option<DownloadDetail>,
}

impl DownloadSnapshot {
    pub fn new(global_stats: GlobalStats, items: Vec<DownloadItem>) -> Self {
        Self {
            global_stats,
            items,
            selected_detail: None,
        }
    }

    pub fn with_selected_detail(
        global_stats: GlobalStats,
        items: Vec<DownloadItem>,
        selected_detail: Option<DownloadDetail>,
    ) -> Self {
        Self {
            global_stats,
            items,
            selected_detail,
        }
    }

    #[cfg(test)]
    pub fn global_stats(&self) -> GlobalStats {
        self.global_stats
    }

    #[cfg(test)]
    pub fn items(&self) -> &[DownloadItem] {
        &self.items
    }

    pub fn selected_detail(&self) -> Option<&DownloadDetail> {
        self.selected_detail.as_ref()
    }

    pub fn into_parts(self) -> (GlobalStats, Vec<DownloadItem>, Option<DownloadDetail>) {
        (self.global_stats, self.items, self.selected_detail)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalStats {
    download_speed_bytes_per_second: u64,
    upload_speed_bytes_per_second: u64,
    active_downloads: u32,
    waiting_downloads: u32,
    stopped_downloads: u32,
}

impl GlobalStats {
    pub fn new(
        download_speed_bytes_per_second: u64,
        upload_speed_bytes_per_second: u64,
        active_downloads: u32,
        waiting_downloads: u32,
        stopped_downloads: u32,
    ) -> Self {
        Self {
            download_speed_bytes_per_second,
            upload_speed_bytes_per_second,
            active_downloads,
            waiting_downloads,
            stopped_downloads,
        }
    }

    pub fn download_speed_bytes_per_second(&self) -> u64 {
        self.download_speed_bytes_per_second
    }

    pub fn upload_speed_bytes_per_second(&self) -> u64 {
        self.upload_speed_bytes_per_second
    }

    pub fn active_downloads(&self) -> u32 {
        self.active_downloads
    }

    pub fn waiting_downloads(&self) -> u32 {
        self.waiting_downloads
    }

    pub fn stopped_downloads(&self) -> u32 {
        self.stopped_downloads
    }
}

impl VersionInfo {
    pub fn new(version: impl Into<String>, enabled_features: Vec<String>) -> Self {
        Self {
            version: version.into(),
            enabled_features,
        }
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    #[cfg(test)]
    pub fn enabled_features(&self) -> &[String] {
        &self.enabled_features
    }
}
