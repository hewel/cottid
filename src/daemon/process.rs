use std::io;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use crate::config::Secret;
use crate::daemon::error::{DaemonError, DaemonErrorKind};
use crate::daemon::paths::ManagedDaemonPaths;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedDaemonArgs {
    args: Vec<String>,
}

impl ManagedDaemonArgs {
    pub fn build(paths: &ManagedDaemonPaths, port: u16, secret: &Secret) -> Self {
        let pid = std::process::id();
        Self {
            args: vec![
                "--enable-rpc=true".to_owned(),
                "--rpc-listen-all=false".to_owned(),
                "--rpc-listen-port".to_owned(),
                port.to_string(),
                "--rpc-secret".to_owned(),
                secret.expose_for_session().to_owned(),
                "--rpc-allow-origin-all=false".to_owned(),
                "--daemon=false".to_owned(),
                "--stop-with-process".to_owned(),
                pid.to_string(),
                "--conf-path".to_owned(),
                paths.config_file().display().to_string(),
                "--input-file".to_owned(),
                paths.session_file().display().to_string(),
                "--save-session".to_owned(),
                paths.session_file().display().to_string(),
                "--log".to_owned(),
                paths.log_file().display().to_string(),
                "--dir".to_owned(),
                paths.download_dir().display().to_string(),
            ],
        }
    }

    pub fn as_slice(&self) -> &[String] {
        &self.args
    }
}

pub fn reserve_loopback_port() -> Result<u16, DaemonError> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|error| DaemonError::new(DaemonErrorKind::PortUnavailable, error.to_string()))?;
    listener
        .local_addr()
        .map(|address| address.port())
        .map_err(|error| DaemonError::new(DaemonErrorKind::PortUnavailable, error.to_string()))
}

pub fn resolve_binary(configured: Option<&Path>) -> Result<PathBuf, DaemonError> {
    if let Some(configured) = configured {
        return executable_path(configured).ok_or_else(|| {
            DaemonError::new(
                DaemonErrorKind::BinaryNotFound,
                format!("configured binary not found: {}", configured.display()),
            )
        });
    }

    let Some(path) = std::env::var_os("PATH") else {
        return Err(DaemonError::new(
            DaemonErrorKind::BinaryNotFound,
            "PATH is not set",
        ));
    };

    for dir in std::env::split_paths(&path) {
        let candidate = dir.join("aria2c");
        if let Some(candidate) = executable_path(&candidate) {
            return Ok(candidate);
        }
    }

    Err(DaemonError::new(
        DaemonErrorKind::BinaryNotFound,
        "aria2c not found in PATH",
    ))
}

pub fn spawn_child(
    binary: &Path,
    args: &ManagedDaemonArgs,
    log_path: &Path,
) -> Result<Child, DaemonError> {
    Command::new(binary)
        .args(args.as_slice())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| spawn_error(error).with_log_path(log_path))
}

fn spawn_error(error: io::Error) -> DaemonError {
    let kind = if error.kind() == io::ErrorKind::PermissionDenied {
        DaemonErrorKind::PermissionDenied
    } else {
        DaemonErrorKind::SpawnFailed
    };

    DaemonError::new(kind, error.to_string())
}

fn executable_path(path: &Path) -> Option<PathBuf> {
    if path.is_file() {
        Some(path.to_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{ManagedDaemonArgs, resolve_binary};
    use crate::config::Secret;
    use crate::daemon::paths::ManagedDaemonPaths;

    #[test]
    fn args_include_required_rpc_session_log_and_parent_process_options() {
        let paths = ManagedDaemonPaths::from_root("/tmp/cottid-aria2")
            .with_download_dir("/tmp/cottid-downloads");
        let args = ManagedDaemonArgs::build(&paths, 68_01, &Secret::session("super-secret"));

        assert!(args.as_slice().contains(&"--enable-rpc=true".to_owned()));
        assert!(
            args.as_slice()
                .contains(&"--rpc-listen-all=false".to_owned())
        );
        assert!(args.as_slice().contains(&"--rpc-listen-port".to_owned()));
        assert!(args.as_slice().contains(&"6801".to_owned()));
        assert!(args.as_slice().contains(&"--rpc-secret".to_owned()));
        assert!(args.as_slice().contains(&"super-secret".to_owned()));
        assert!(args.as_slice().contains(&"--stop-with-process".to_owned()));
        assert!(args.as_slice().contains(&"--conf-path".to_owned()));
        assert!(args.as_slice().contains(&"--input-file".to_owned()));
        assert!(args.as_slice().contains(&"--save-session".to_owned()));
        assert!(args.as_slice().contains(&"--log".to_owned()));
        assert!(args.as_slice().contains(&"--dir".to_owned()));
    }

    #[test]
    fn args_do_not_start_aria2_in_daemon_mode() {
        let paths = ManagedDaemonPaths::from_root("/tmp/cottid-aria2");
        let args = ManagedDaemonArgs::build(&paths, 68_01, &Secret::session("super-secret"));

        assert!(!args.as_slice().contains(&"--daemon=true".to_owned()));
        assert!(args.as_slice().contains(&"--daemon=false".to_owned()));
    }

    #[test]
    fn configured_binary_path_wins_before_path_lookup() {
        let dir = temp_dir("binary");
        fs::create_dir_all(&dir).expect("dir");
        let binary = dir.join("custom-aria2c");
        fs::write(&binary, "").expect("binary");

        let resolved = resolve_binary(Some(&binary)).expect("configured binary");

        assert_eq!(resolved, binary);
    }

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("cottid-daemon-process-{name}-{unique}"))
    }
}
