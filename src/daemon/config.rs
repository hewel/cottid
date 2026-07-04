use std::path::PathBuf;
use std::time::Duration;

use crate::config::{EndpointValidationError, Secret, Settings};
use crate::daemon::paths::ManagedDaemonPaths;

const DEFAULT_READINESS_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_READINESS_INTERVAL: Duration = Duration::from_millis(150);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedDaemonConfig {
    binary_path: Option<PathBuf>,
    paths: ManagedDaemonPaths,
    readiness_timeout: Duration,
    readiness_interval: Duration,
    polling_interval_seconds: u16,
    websocket_enabled: bool,
}

impl ManagedDaemonConfig {
    pub fn new(paths: ManagedDaemonPaths) -> Self {
        Self {
            binary_path: None,
            paths,
            readiness_timeout: DEFAULT_READINESS_TIMEOUT,
            readiness_interval: DEFAULT_READINESS_INTERVAL,
            polling_interval_seconds: 2,
            websocket_enabled: true,
        }
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "configured aria2c path is prepared before the settings UI persists it"
        )
    )]
    pub fn with_binary_path(mut self, binary_path: impl Into<PathBuf>) -> Self {
        self.binary_path = Some(binary_path.into());
        self
    }

    pub fn with_polling_interval_seconds(mut self, seconds: u16) -> Self {
        self.polling_interval_seconds = seconds.max(1);
        self
    }

    pub fn with_websocket_enabled(mut self, enabled: bool) -> Self {
        self.websocket_enabled = enabled;
        self
    }

    #[cfg(test)]
    pub fn with_readiness_timeout(mut self, timeout: Duration) -> Self {
        self.readiness_timeout = timeout;
        self
    }

    #[cfg(test)]
    pub fn with_readiness_interval(mut self, interval: Duration) -> Self {
        self.readiness_interval = interval;
        self
    }

    pub fn binary_path(&self) -> Option<&PathBuf> {
        self.binary_path.as_ref()
    }

    pub fn paths(&self) -> &ManagedDaemonPaths {
        &self.paths
    }

    pub fn readiness_timeout(&self) -> Duration {
        self.readiness_timeout
    }

    pub fn readiness_interval(&self) -> Duration {
        self.readiness_interval
    }

    pub fn polling_interval_seconds(&self) -> u16 {
        self.polling_interval_seconds
    }

    pub fn websocket_enabled(&self) -> bool {
        self.websocket_enabled
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedRuntimeConfig {
    port: u16,
    secret: Secret,
    settings: Settings,
}

impl ManagedRuntimeConfig {
    pub fn new(
        port: u16,
        secret: Secret,
        polling_interval_seconds: u16,
        websocket_enabled: bool,
    ) -> Result<Self, EndpointValidationError> {
        let endpoint = format!("http://127.0.0.1:{port}/jsonrpc");
        let settings = Settings::new_with_session_secret(
            endpoint,
            secret.clone(),
            polling_interval_seconds,
            websocket_enabled,
        )?;

        Ok(Self {
            port,
            secret,
            settings,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn secret(&self) -> &Secret {
        &self.secret
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{ManagedDaemonConfig, ManagedRuntimeConfig};
    use crate::config::{RpcAuth, Secret};
    use crate::daemon::paths::ManagedDaemonPaths;

    #[test]
    fn runtime_config_builds_loopback_settings_with_session_secret() {
        let runtime = ManagedRuntimeConfig::new(68_01, Secret::session("managed-secret"), 3, false)
            .expect("runtime config");

        assert_eq!(
            runtime.settings().endpoint(),
            "http://127.0.0.1:6801/jsonrpc"
        );
        assert_eq!(runtime.settings().polling_interval_seconds(), 3);
        assert!(!runtime.settings().websocket_enabled());
        assert_eq!(
            runtime.settings().auth(),
            &RpcAuth::SessionSecret(Secret::session("managed-secret"))
        );
        assert!(!format!("{runtime:?}").contains("managed-secret"));
    }

    #[test]
    fn daemon_config_records_binary_readiness_and_transport_preferences() {
        let config = ManagedDaemonConfig::new(ManagedDaemonPaths::from_root("/tmp/cottid"))
            .with_binary_path("/usr/bin/aria2c")
            .with_polling_interval_seconds(4)
            .with_websocket_enabled(false)
            .with_readiness_timeout(Duration::from_secs(1))
            .with_readiness_interval(Duration::from_millis(10));

        assert_eq!(
            config.binary_path().map(|path| path.as_path()),
            Some(std::path::Path::new("/usr/bin/aria2c"))
        );
        assert_eq!(config.polling_interval_seconds(), 4);
        assert!(!config.websocket_enabled());
        assert_eq!(config.readiness_timeout(), Duration::from_secs(1));
        assert_eq!(config.readiness_interval(), Duration::from_millis(10));
    }
}
