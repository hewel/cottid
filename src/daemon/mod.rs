pub mod config;
pub mod error;
pub mod health;
pub mod manager;
pub mod paths;
pub mod process;

pub use config::ManagedDaemonConfig;
#[cfg(test)]
pub use config::ManagedRuntimeConfig;
pub use error::DaemonError;
pub use manager::{DaemonManager, ManagedDaemonStart, ManagedDaemonStop};

pub async fn start_managed_daemon(
    config: ManagedDaemonConfig,
) -> Result<ManagedDaemonStart, DaemonError> {
    manager::start_managed_daemon(config).await
}

pub async fn stop_managed_daemon(manager: DaemonManager) -> Result<ManagedDaemonStop, DaemonError> {
    manager::stop_managed_daemon(manager).await
}
