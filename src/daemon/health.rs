use std::process::Child;
use std::time::{Duration, Instant};

use crate::config::Settings;
use crate::daemon::error::{DaemonError, DaemonErrorKind};

pub async fn wait_for_rpc_ready(
    child: &mut Child,
    settings: Settings,
    timeout: Duration,
    interval: Duration,
) -> Result<crate::aria2::client::ConnectionTest, DaemonError> {
    let deadline = Instant::now() + timeout;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Err(DaemonError::new(
                    DaemonErrorKind::ExitedEarly,
                    format!("child exited with {status}"),
                ));
            }
            Ok(None) => {}
            Err(error) => {
                return Err(DaemonError::new(
                    DaemonErrorKind::ExitedEarly,
                    error.to_string(),
                ));
            }
        }

        match crate::aria2::client::test_connection(settings.clone()).await {
            Ok(result) => return Ok(result),
            Err(crate::aria2::errors::ClientError::Rpc { code, message }) => {
                return Err(DaemonError::from_client_error(
                    crate::aria2::errors::ClientError::Rpc { code, message },
                ));
            }
            Err(error) => {
                let _ = error;
            }
        }

        if Instant::now() >= deadline {
            return Err(readiness_timeout_error());
        }

        tokio::time::sleep(interval).await;
    }
}

fn readiness_timeout_error() -> DaemonError {
    DaemonError::new(DaemonErrorKind::ReadinessTimeout, "readiness timed out")
}

#[cfg(test)]
mod tests {
    use crate::daemon::error::DaemonErrorKind;

    #[test]
    fn readiness_timeout_error_is_classified_as_timeout() {
        let error = super::readiness_timeout_error();

        assert_eq!(error.kind(), DaemonErrorKind::ReadinessTimeout);
    }
}
