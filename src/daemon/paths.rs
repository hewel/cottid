use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedDaemonPaths {
    root_dir: PathBuf,
    config_file: PathBuf,
    session_file: PathBuf,
    log_file: PathBuf,
    download_dir: PathBuf,
}

impl ManagedDaemonPaths {
    pub fn from_root(root_dir: impl Into<PathBuf>) -> Self {
        let root_dir = root_dir.into();
        Self {
            config_file: root_dir.join("aria2.conf"),
            session_file: root_dir.join("aria2.session"),
            log_file: root_dir.join("aria2.log"),
            download_dir: default_download_dir(),
            root_dir,
        }
    }

    pub fn with_download_dir(mut self, download_dir: impl Into<PathBuf>) -> Self {
        self.download_dir = download_dir.into();
        self
    }

    pub fn prepare(&self) -> io::Result<()> {
        fs::create_dir_all(&self.root_dir)?;
        fs::create_dir_all(&self.download_dir)?;
        create_file_if_missing(&self.config_file)?;
        create_file_if_missing(&self.session_file)?;
        Ok(())
    }

    pub fn root_dir(&self) -> &Path {
        &self.root_dir
    }

    pub fn config_file(&self) -> &Path {
        &self.config_file
    }

    pub fn session_file(&self) -> &Path {
        &self.session_file
    }

    pub fn log_file(&self) -> &Path {
        &self.log_file
    }

    pub fn download_dir(&self) -> &Path {
        &self.download_dir
    }
}

pub fn default_managed_root_dir() -> PathBuf {
    if let Some(xdg_state_home) = std::env::var_os("XDG_STATE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return xdg_state_home.join("cottid").join("aria2");
    }

    if let Some(home) = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return home
            .join(".local")
            .join("state")
            .join("cottid")
            .join("aria2");
    }

    PathBuf::from("cottid").join("aria2")
}

pub fn default_download_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return home.join("Downloads");
    }

    PathBuf::from("Downloads")
}

fn create_file_if_missing(path: &Path) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }

    fs::write(path, "")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::ManagedDaemonPaths;

    #[test]
    fn prepare_creates_config_session_log_parent_and_download_dirs() {
        let root = temp_dir("paths");
        let downloads = temp_dir("downloads");
        let paths = ManagedDaemonPaths::from_root(&root).with_download_dir(&downloads);

        paths.prepare().expect("paths prepared");

        assert!(paths.root_dir().is_dir());
        assert!(paths.config_file().is_file());
        assert!(paths.session_file().is_file());
        assert!(paths.download_dir().is_dir());
        assert_eq!(paths.log_file(), root.join("aria2.log"));
    }

    #[test]
    fn prepare_preserves_existing_config_contents() {
        let root = temp_dir("preserve");
        fs::create_dir_all(&root).expect("root");
        let config = root.join("aria2.conf");
        fs::write(&config, "max-concurrent-downloads=3").expect("config");
        let paths = ManagedDaemonPaths::from_root(&root);

        paths.prepare().expect("paths prepared");

        assert_eq!(
            fs::read_to_string(config).expect("config"),
            "max-concurrent-downloads=3"
        );
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("cottid-daemon-{name}-{unique}"))
    }
}
