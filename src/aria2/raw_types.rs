use serde::Deserialize;

use crate::aria2::domain::{
    DownloadFile, DownloadItem, DownloadStatus, Gid, GlobalStats, VersionInfo,
};
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
struct RawDownloadItem {
    gid: String,
    status: String,
    #[serde(rename = "totalLength")]
    total_length: String,
    #[serde(rename = "completedLength")]
    completed_length: String,
    #[serde(rename = "downloadSpeed", default)]
    download_speed: String,
    #[serde(rename = "uploadSpeed", default)]
    upload_speed: String,
    #[serde(default)]
    files: Vec<RawDownloadFile>,
}

#[derive(Debug, Deserialize)]
struct RawDownloadFile {
    path: String,
    length: String,
    #[serde(rename = "completedLength")]
    completed_length: String,
    #[serde(default)]
    selected: String,
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

pub fn parse_download_items_response(
    body: &str,
    expected_id: RequestId,
) -> Result<Vec<DownloadItem>, ClientError> {
    let envelope: JsonRpcEnvelope<Vec<RawDownloadItem>> = parse_envelope(body, expected_id)?;
    let raw_items = envelope
        .result
        .ok_or_else(|| ClientError::MalformedResponse("missing downloads result".to_owned()))?;

    raw_items.into_iter().map(parse_download_item).collect()
}

pub fn parse_add_uri_response(body: &str, expected_id: RequestId) -> Result<Gid, ClientError> {
    let envelope: JsonRpcEnvelope<String> = parse_envelope(body, expected_id)?;
    let gid = envelope
        .result
        .ok_or_else(|| ClientError::MalformedResponse("missing addUri result".to_owned()))?;

    Gid::new(gid).map_err(|error| ClientError::MalformedResponse(error.message().to_owned()))
}

pub fn parse_gid_command_response(body: &str, expected_id: RequestId) -> Result<Gid, ClientError> {
    parse_add_uri_response(body, expected_id)
}

pub fn parse_ok_response(body: &str, expected_id: RequestId) -> Result<(), ClientError> {
    let envelope: JsonRpcEnvelope<String> = parse_envelope(body, expected_id)?;
    let result = envelope
        .result
        .ok_or_else(|| ClientError::MalformedResponse("missing command result".to_owned()))?;

    if result == "OK" {
        Ok(())
    } else {
        Err(ClientError::MalformedResponse(
            "command result must be OK".to_owned(),
        ))
    }
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

fn parse_download_item(raw: RawDownloadItem) -> Result<DownloadItem, ClientError> {
    let gid = Gid::new(raw.gid)
        .map_err(|error| ClientError::MalformedResponse(error.message().to_owned()))?;
    let files = raw
        .files
        .into_iter()
        .map(parse_download_file)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(DownloadItem::new(
        gid,
        DownloadStatus::from_aria2(raw.status),
        parse_u64("totalLength", &raw.total_length)?,
        parse_u64("completedLength", &raw.completed_length)?,
        parse_optional_u64("downloadSpeed", &raw.download_speed)?,
        parse_optional_u64("uploadSpeed", &raw.upload_speed)?,
        files,
    ))
}

fn parse_download_file(raw: RawDownloadFile) -> Result<DownloadFile, ClientError> {
    Ok(DownloadFile::new(
        raw.path,
        parse_u64("file.length", &raw.length)?,
        parse_u64("file.completedLength", &raw.completed_length)?,
        raw.selected == "true",
    ))
}

fn parse_u64(field: &'static str, value: &str) -> Result<u64, ClientError> {
    value.parse::<u64>().map_err(|error| {
        ClientError::MalformedResponse(format!("{field} must be an unsigned integer: {error}"))
    })
}

fn parse_optional_u64(field: &'static str, value: &str) -> Result<u64, ClientError> {
    if value.is_empty() {
        return Ok(0);
    }

    parse_u64(field, value)
}

fn parse_u32(field: &'static str, value: &str) -> Result<u32, ClientError> {
    value.parse::<u32>().map_err(|error| {
        ClientError::MalformedResponse(format!("{field} must be an unsigned integer: {error}"))
    })
}

#[cfg(test)]
mod tests {
    use super::{
        parse_add_uri_response, parse_download_items_response, parse_get_version_response,
        parse_gid_command_response, parse_global_stats_response, parse_ok_response,
    };
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

    #[test]
    fn parses_download_items_with_typed_gid_status_files_and_numbers() {
        let items = parse_download_items_response(
            r#"{"jsonrpc":"2.0","id":21,"result":[{"gid":"abc123","status":"active","totalLength":"2048","completedLength":"1024","downloadSpeed":"512","uploadSpeed":"0","files":[{"path":"/tmp/file.iso","length":"2048","completedLength":"1024","selected":"true"}]}]}"#,
            RequestId::new(21),
        )
        .expect("valid downloads");

        assert_eq!(items.len(), 1);
        assert_eq!(items[0].gid().as_str(), "abc123");
        assert_eq!(
            items[0].status(),
            &crate::aria2::domain::DownloadStatus::Active
        );
        assert_eq!(items[0].total_length_bytes(), 2048);
        assert_eq!(items[0].completed_length_bytes(), 1024);
        assert_eq!(items[0].download_speed_bytes_per_second(), 512);
        assert_eq!(items[0].files()[0].path(), "/tmp/file.iso");
        assert_eq!(items[0].files()[0].length_bytes(), 2048);
        assert_eq!(items[0].files()[0].completed_length_bytes(), 1024);
        assert!(items[0].files()[0].selected());
    }

    #[test]
    fn keeps_unknown_download_status_as_typed_domain_value() {
        let items = parse_download_items_response(
            r#"{"jsonrpc":"2.0","id":21,"result":[{"gid":"abc123","status":"mystery","totalLength":"0","completedLength":"0","files":[]}]}"#,
            RequestId::new(21),
        )
        .expect("unknown status is still data");

        assert_eq!(
            items[0].status(),
            &crate::aria2::domain::DownloadStatus::Unknown("mystery".to_owned())
        );
    }

    #[test]
    fn rejects_download_items_with_malformed_numeric_fields() {
        let error = parse_download_items_response(
            r#"{"jsonrpc":"2.0","id":21,"result":[{"gid":"abc123","status":"active","totalLength":"large","completedLength":"0","files":[]}]}"#,
            RequestId::new(21),
        )
        .expect_err("malformed item should fail");

        assert!(matches!(error, ClientError::MalformedResponse(_)));
    }

    #[test]
    fn parses_add_uri_result_into_typed_gid() {
        let gid = parse_add_uri_response(
            r#"{"jsonrpc":"2.0","id":31,"result":"new-gid"}"#,
            RequestId::new(31),
        )
        .expect("valid addUri result");

        assert_eq!(gid.as_str(), "new-gid");
    }

    #[test]
    fn rejects_empty_add_uri_gid() {
        let error = parse_add_uri_response(
            r#"{"jsonrpc":"2.0","id":31,"result":""}"#,
            RequestId::new(31),
        )
        .expect_err("empty gid should fail");

        assert!(matches!(error, ClientError::MalformedResponse(_)));
    }

    #[test]
    fn parses_gid_command_response() {
        let gid = parse_gid_command_response(
            r#"{"jsonrpc":"2.0","id":41,"result":"abc123"}"#,
            RequestId::new(41),
        )
        .expect("valid gid command result");

        assert_eq!(gid.as_str(), "abc123");
    }

    #[test]
    fn parses_ok_command_response() {
        parse_ok_response(
            r#"{"jsonrpc":"2.0","id":44,"result":"OK"}"#,
            RequestId::new(44),
        )
        .expect("valid OK response");
    }

    #[test]
    fn rejects_non_ok_command_response() {
        let error = parse_ok_response(
            r#"{"jsonrpc":"2.0","id":44,"result":"NO"}"#,
            RequestId::new(44),
        )
        .expect_err("non OK should fail");

        assert!(matches!(error, ClientError::MalformedResponse(_)));
    }
}
