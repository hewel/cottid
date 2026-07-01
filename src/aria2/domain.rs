#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionInfo {
    version: String,
    enabled_features: Vec<String>,
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
