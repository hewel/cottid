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

    #[cfg(test)]
    pub fn length_bytes(&self) -> u64 {
        self.length_bytes
    }

    #[cfg(test)]
    pub fn completed_length_bytes(&self) -> u64 {
        self.completed_length_bytes
    }

    pub fn selected(&self) -> bool {
        self.selected
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadSnapshot {
    global_stats: GlobalStats,
    items: Vec<DownloadItem>,
}

impl DownloadSnapshot {
    pub fn new(global_stats: GlobalStats, items: Vec<DownloadItem>) -> Self {
        Self {
            global_stats,
            items,
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

    pub fn into_parts(self) -> (GlobalStats, Vec<DownloadItem>) {
        (self.global_stats, self.items)
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
