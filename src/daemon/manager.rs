use std::fmt;
use std::process::{Child, ExitStatus};
use std::sync::{Arc, Mutex};

use crate::config::Secret;
use crate::daemon::config::{ManagedDaemonConfig, ManagedRuntimeConfig};
use crate::daemon::error::{DaemonError, DaemonErrorKind};
use crate::daemon::health::wait_for_rpc_ready;
use crate::daemon::paths::ManagedDaemonPaths;
use crate::daemon::process::{
    ManagedDaemonArgs, reserve_loopback_port, resolve_binary, spawn_child,
};

#[derive(Clone)]
pub struct DaemonManager {
    child: Option<Arc<Mutex<Child>>>,
    paths: ManagedDaemonPaths,
    runtime: ManagedRuntimeConfig,
}

impl DaemonManager {
    #[cfg(test)]
    pub fn test(runtime: ManagedRuntimeConfig, paths: ManagedDaemonPaths) -> Self {
        Self {
            child: None,
            paths,
            runtime,
        }
    }

    pub fn paths(&self) -> &ManagedDaemonPaths {
        &self.paths
    }

    pub fn runtime(&self) -> &ManagedRuntimeConfig {
        &self.runtime
    }

    pub fn try_wait(&self) -> Result<Option<ExitStatus>, DaemonError> {
        let Some(child) = self.child.as_ref() else {
            return Ok(None);
        };
        let mut child = child
            .lock()
            .map_err(|_| DaemonError::new(DaemonErrorKind::Crash, "child lock poisoned"))?;
        child
            .try_wait()
            .map_err(|error| DaemonError::new(DaemonErrorKind::Crash, error.to_string()))
    }
}

impl fmt::Debug for DaemonManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DaemonManager")
            .field("has_child", &self.child.is_some())
            .field("paths", &self.paths)
            .field("runtime", &self.runtime)
            .finish()
    }
}

impl PartialEq for DaemonManager {
    fn eq(&self, other: &Self) -> bool {
        self.paths == other.paths && self.runtime == other.runtime
    }
}

impl Eq for DaemonManager {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedDaemonStart {
    manager: DaemonManager,
    version: crate::aria2::client::ConnectionTest,
}

impl ManagedDaemonStart {
    #[cfg(test)]
    pub fn test(manager: DaemonManager, version: crate::aria2::client::ConnectionTest) -> Self {
        Self { manager, version }
    }

    pub fn manager(&self) -> &DaemonManager {
        &self.manager
    }

    pub fn into_parts(self) -> (DaemonManager, crate::aria2::client::ConnectionTest) {
        (self.manager, self.version)
    }
}

pub async fn start_managed_daemon(
    config: ManagedDaemonConfig,
) -> Result<ManagedDaemonStart, DaemonError> {
    config.paths().prepare().map_err(|error| {
        DaemonError::new(DaemonErrorKind::ConfigIo, error.to_string())
            .with_log_path(config.paths().log_file())
    })?;

    let binary = resolve_binary(config.binary_path().map(std::path::PathBuf::as_path))?;
    let port = reserve_loopback_port()?;
    let secret = generate_secret()?;
    let runtime = ManagedRuntimeConfig::new(
        port,
        secret,
        config.polling_interval_seconds(),
        config.websocket_enabled(),
    )
    .map_err(|error| DaemonError::new(DaemonErrorKind::PortUnavailable, error.message()))?;
    let args = ManagedDaemonArgs::build(config.paths(), runtime.port(), runtime.secret());
    let mut child = spawn_child(&binary, &args, config.paths().log_file())?;
    let version = match wait_for_rpc_ready(
        &mut child,
        runtime.settings().clone(),
        config.readiness_timeout(),
        config.readiness_interval(),
    )
    .await
    {
        Ok(version) => version,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error.with_log_path(config.paths().log_file()));
        }
    };

    Ok(ManagedDaemonStart {
        manager: DaemonManager {
            child: Some(Arc::new(Mutex::new(child))),
            paths: config.paths().clone(),
            runtime,
        },
        version,
    })
}

fn generate_secret() -> Result<Secret, DaemonError> {
    let mut bytes = [0_u8; 32];
    getrandom::fill(&mut bytes)
        .map_err(|error| DaemonError::new(DaemonErrorKind::SecretGeneration, error.to_string()))?;
    Ok(Secret::session(hex_encode(&bytes)))
}

fn hex_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(TABLE[(byte >> 4) as usize] as char);
        output.push(TABLE[(byte & 0x0f) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::hex_encode;

    #[test]
    fn hex_encode_uses_lowercase_pairs() {
        assert_eq!(hex_encode(&[0, 15, 16, 255]), "000f10ff");
    }
}
