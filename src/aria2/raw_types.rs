use serde::Deserialize;

use crate::aria2::domain::VersionInfo;
use crate::aria2::errors::ClientError;
use crate::aria2::methods::RequestId;

#[derive(Debug, Deserialize)]
struct JsonRpcEnvelope {
    id: RequestId,
    #[serde(default)]
    result: Option<RawVersionInfo>,
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
struct RawRpcError {
    code: i64,
    message: String,
}

pub fn parse_get_version_response(
    body: &str,
    expected_id: RequestId,
) -> Result<VersionInfo, ClientError> {
    let envelope: JsonRpcEnvelope = serde_json::from_str(body)
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

    let result = envelope
        .result
        .ok_or_else(|| ClientError::MalformedResponse("missing result".to_owned()))?;

    Ok(VersionInfo::new(result.version, result.enabled_features))
}

#[cfg(test)]
mod tests {
    use super::parse_get_version_response;
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
}
