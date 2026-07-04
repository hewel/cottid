use std::fmt;
use std::path::PathBuf;

use crate::aria2::errors::ClientError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonErrorKind {
    BinaryNotFound,
    PermissionDenied,
    PortUnavailable,
    SpawnFailed,
    ExitedEarly,
    ReadinessTimeout,
    AuthFailure,
    ConnectionFailed,
    ConfigIo,
    SecretGeneration,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DaemonError {
    kind: DaemonErrorKind,
    detail: String,
    log_path: Option<PathBuf>,
}

impl DaemonError {
    pub fn new(kind: DaemonErrorKind, detail: impl Into<String>) -> Self {
        Self {
            kind,
            detail: detail.into(),
            log_path: None,
        }
    }

    pub fn with_log_path(mut self, log_path: impl Into<PathBuf>) -> Self {
        self.log_path = Some(log_path.into());
        self
    }

    pub fn from_client_error(error: ClientError) -> Self {
        match error {
            ClientError::Rpc { code, .. } => {
                Self::new(DaemonErrorKind::AuthFailure, format!("rpc error {code}"))
            }
            ClientError::Transport(_)
            | ClientError::HttpStatus(_)
            | ClientError::MalformedResponse(_)
            | ClientError::ResponseIdMismatch { .. } => {
                Self::new(DaemonErrorKind::ConnectionFailed, "readiness check failed")
            }
        }
    }

    pub fn kind(&self) -> DaemonErrorKind {
        self.kind
    }

    pub fn display_message(&self) -> &'static str {
        match self.kind {
            DaemonErrorKind::BinaryNotFound => "aria2c could not be found.",
            DaemonErrorKind::PermissionDenied => "aria2c could not be started due to permissions.",
            DaemonErrorKind::PortUnavailable => "A local RPC port could not be reserved.",
            DaemonErrorKind::SpawnFailed => "aria2c could not be started.",
            DaemonErrorKind::ExitedEarly => "aria2c exited before RPC became ready.",
            DaemonErrorKind::ReadinessTimeout => "aria2c did not become ready in time.",
            DaemonErrorKind::AuthFailure => "Managed aria2 authentication failed.",
            DaemonErrorKind::ConnectionFailed => "Managed aria2 readiness check failed.",
            DaemonErrorKind::ConfigIo => "Managed aria2 paths could not be prepared.",
            DaemonErrorKind::SecretGeneration => "Managed aria2 secret could not be generated.",
        }
    }

    pub fn log_path(&self) -> Option<&PathBuf> {
        self.log_path.as_ref()
    }
}

impl fmt::Debug for DaemonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DaemonError")
            .field("kind", &self.kind)
            .field("detail", &"<redacted>")
            .field("log_path", &self.log_path)
            .finish()
    }
}

impl fmt::Display for DaemonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.display_message())
    }
}

impl std::error::Error for DaemonError {}

#[cfg(test)]
mod tests {
    use super::{DaemonError, DaemonErrorKind};

    #[test]
    fn debug_redacts_error_detail() {
        let error = DaemonError::new(DaemonErrorKind::SpawnFailed, "token:super-secret");

        assert!(!format!("{error:?}").contains("super-secret"));
    }
}
