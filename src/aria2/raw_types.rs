use serde::Deserialize;

use crate::aria2::domain::{GlobalStats, VersionInfo};
use crate::aria2::errors::ClientError;
use crate::aria2::methods::RequestId;

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct JsonRpcEnvelope<T> {
    id: RequestId,
    #[serde(default)]
    result: Option<T>,
    #[serde(default)]
    error: Option<RawRpcError>,
}

#[derive(Debug, Deserialize)]
struct RawVersionInfo {
    version: String,
    #[serde(rename = "enabledFeatures", default)]
    enabled_features: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawGlobalStats {
    #[serde(rename = "downloadSpeed")]
    download_speed: String,
    #[serde(rename = "uploadSpeed")]
    upload_speed: String,
    #[serde(rename = "numActive")]
    active_downloads: String,
    #[serde(rename = "numWaiting")]
    waiting_downloads: String,
    #[serde(rename = "numStopped")]
    stopped_downloads: String,
}

#[derive(Debug, Deserialize)]
struct RawRpcError {
    code: i64,
    message: String,
}

pub fn parse_get_version_response(
    body: &str,
    expected_id: RequestId,
) -> Result<VersionInfo, ClientError> {
    let envelope: JsonRpcEnvelope<RawVersionInfo> = parse_envelope(body, expected_id)?;

    let result = envelope
        .result
        .ok_or_else(|| ClientError::MalformedResponse("missing result".to_owned()))?;

    Ok(VersionInfo::new(result.version, result.enabled_features))
}

pub fn parse_global_stats_response(
    body: &str,
    expected_id: RequestId,
) -> Result<GlobalStats, ClientError> {
    let envelope: JsonRpcEnvelope<RawGlobalStats> = parse_envelope(body, expected_id)?;

    let raw = envelope
        .result
        .ok_or_else(|| ClientError::MalformedResponse("missing global stats result".to_owned()))?;

    Ok(GlobalStats::new(
        parse_u64("downloadSpeed", &raw.download_speed)?,
        parse_u64("uploadSpeed", &raw.upload_speed)?,
        parse_u32("numActive", &raw.active_downloads)?,
        parse_u32("numWaiting", &raw.waiting_downloads)?,
        parse_u32("numStopped", &raw.stopped_downloads)?,
    ))
}

fn parse_envelope<T>(body: &str, expected_id: RequestId) -> Result<JsonRpcEnvelope<T>, ClientError>
where
    T: for<'de> Deserialize<'de>,
{
    let envelope: JsonRpcEnvelope<T> = serde_json::from_str(body)
        .map_err(|error| ClientError::MalformedResponse(error.to_string()))?;

    if envelope.id != expected_id {
        return Err(ClientError::ResponseIdMismatch {
            expected: expected_id,
            actual: envelope.id,
        });
    }

    if let Some(error) = envelope.error {
        return Err(ClientError::Rpc {
            code: error.code,
            message: error.message,
        });
    }

    Ok(envelope)
}

fn parse_u64(field: &'static str, value: &str) -> Result<u64, ClientError> {
    value.parse::<u64>().map_err(|error| {
        ClientError::MalformedResponse(format!("{field} must be an unsigned integer: {error}"))
    })
}

fn parse_u32(field: &'static str, value: &str) -> Result<u32, ClientError> {
    value.parse::<u32>().map_err(|error| {
        ClientError::MalformedResponse(format!("{field} must be an unsigned integer: {error}"))
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_get_version_response, parse_global_stats_response};
    use crate::aria2::errors::ClientError;
    use crate::aria2::methods::RequestId;

    #[test]
    fn maps_get_version_success_into_domain_data() {
        let version = parse_get_version_response(
            r#"{"jsonrpc":"2.0","id":7,"result":{"version":"1.37.0","enabledFeatures":["Async DNS"]}}"#,
            RequestId::new(7),
        )
        .expect("valid version response");

        assert_eq!(version.version(), "1.37.0");
        assert_eq!(version.enabled_features(), &["Async DNS".to_owned()]);
    }

    #[test]
    fn rejects_mismatched_response_ids() {
        let error = parse_get_version_response(
            r#"{"jsonrpc":"2.0","id":9,"result":{"version":"1.37.0"}}"#,
            RequestId::new(7),
        )
        .expect_err("id mismatch should fail");

        assert_eq!(
            error,
            ClientError::ResponseIdMismatch {
                expected: RequestId::new(7),
                actual: RequestId::new(9),
            }
        );
    }

    #[test]
    fn maps_rpc_errors_without_leaking_messages_in_debug() {
        let error = parse_get_version_response(
            r#"{"jsonrpc":"2.0","id":7,"error":{"code":1,"message":"token:super-secret rejected"}}"#,
            RequestId::new(7),
        )
        .expect_err("rpc error should fail");

        assert_eq!(
            error,
            ClientError::Rpc {
                code: 1,
                message: "token:super-secret rejected".to_owned(),
            }
        );
        assert!(!format!("{error:?}").contains("super-secret"));
    }

    #[test]
    fn maps_malformed_responses_to_typed_errors() {
        let error = parse_get_version_response("not json", RequestId::new(7))
            .expect_err("malformed response should fail");

        assert!(matches!(error, ClientError::MalformedResponse(_)));
    }

    #[test]
    fn parses_global_stats_numeric_strings_before_domain_state() {
        let stats = parse_global_stats_response(
            r#"{"jsonrpc":"2.0","id":11,"result":{"downloadSpeed":"1536","uploadSpeed":"0","numActive":"2","numWaiting":"3","numStopped":"4"}}"#,
            RequestId::new(11),
        )
        .expect("valid stats");

        assert_eq!(stats.download_speed_bytes_per_second(), 1536);
        assert_eq!(stats.upload_speed_bytes_per_second(), 0);
        assert_eq!(stats.active_downloads(), 2);
        assert_eq!(stats.waiting_downloads(), 3);
        assert_eq!(stats.stopped_downloads(), 4);
    }

    #[test]
    fn rejects_malformed_global_stats_numeric_fields() {
        let error = parse_global_stats_response(
            r#"{"jsonrpc":"2.0","id":11,"result":{"downloadSpeed":"fast","uploadSpeed":"0","numActive":"2","numWaiting":"3","numStopped":"4"}}"#,
            RequestId::new(11),
        )
        .expect_err("malformed numeric field should fail");

        assert!(matches!(error, ClientError::MalformedResponse(_)));
    }
}
