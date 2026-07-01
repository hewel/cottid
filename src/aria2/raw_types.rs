use serde::Deserialize;
use serde_json::Value;

use crate::aria2::domain::{
    DownloadDetail, DownloadFile, DownloadItem, DownloadStatus, Gid, GlobalStats, TorrentDetail,
    VersionInfo,
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

#[derive(Debug)]
enum JsonRpcResponse<T> {
    Success { id: RequestId, result: T },
    Error { error: RawRpcError },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MulticallEntryKind {
    GlobalStats,
    DownloadItems,
    SelectedDownloadDetail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MulticallEntry {
    GlobalStats(GlobalStats),
    DownloadItems(Vec<DownloadItem>),
    SelectedDownloadDetail(DownloadDetail),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawMulticallEntry {
    Success(Vec<Value>),
    Fault {
        #[serde(rename = "faultCode")]
        code: i64,
        #[serde(rename = "faultString")]
        message: String,
    },
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
    dir: String,
    #[serde(default)]
    connections: String,
    #[serde(rename = "pieceLength", default)]
    piece_length: String,
    #[serde(rename = "numPieces", default)]
    piece_count: String,
    #[serde(rename = "errorCode", default)]
    error_code: String,
    #[serde(rename = "errorMessage", default)]
    error_message: String,
    #[serde(rename = "infoHash", default)]
    info_hash: String,
    #[serde(default)]
    seeder: String,
    #[serde(rename = "numSeeders", default)]
    num_seeders: String,
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

pub fn parse_download_detail_response(
    body: &str,
    expected_id: RequestId,
) -> Result<DownloadDetail, ClientError> {
    let envelope: JsonRpcEnvelope<RawDownloadItem> = parse_envelope(body, expected_id)?;
    let raw = envelope.result.ok_or_else(|| {
        ClientError::MalformedResponse("missing download detail result".to_owned())
    })?;

    parse_download_detail(raw)
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
    let response = parse_response(body, expected_id)?;

    match response {
        JsonRpcResponse::Success { id, result } => Ok(JsonRpcEnvelope {
            id,
            result: Some(result),
            error: None,
        }),
        JsonRpcResponse::Error { error, .. } => Err(ClientError::Rpc {
            code: error.code,
            message: error.message,
        }),
    }
}

pub fn parse_multicall_response(
    body: &str,
    expected_id: RequestId,
    kinds: &[MulticallEntryKind],
) -> Result<Vec<MulticallEntry>, ClientError> {
    let entries = match parse_response::<Vec<RawMulticallEntry>>(body, expected_id)? {
        JsonRpcResponse::Success { result, .. } => result,
        JsonRpcResponse::Error { error, .. } => {
            return Err(ClientError::Rpc {
                code: error.code,
                message: error.message,
            });
        }
    };

    if entries.len() != kinds.len() {
        return Err(ClientError::MalformedResponse(
            "multicall result count did not match request count".to_owned(),
        ));
    }

    entries
        .into_iter()
        .zip(kinds.iter().copied())
        .map(parse_multicall_entry)
        .collect()
}

fn parse_response<T>(body: &str, expected_id: RequestId) -> Result<JsonRpcResponse<T>, ClientError>
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
        return Ok(JsonRpcResponse::Error { error });
    }

    let result = envelope
        .result
        .ok_or_else(|| ClientError::MalformedResponse("missing result".to_owned()))?;

    Ok(JsonRpcResponse::Success {
        id: envelope.id,
        result,
    })
}

fn parse_multicall_entry(
    entry: (RawMulticallEntry, MulticallEntryKind),
) -> Result<MulticallEntry, ClientError> {
    let (entry, kind) = entry;
    let values = match entry {
        RawMulticallEntry::Success(values) => values,
        RawMulticallEntry::Fault { code, message } => {
            return Err(ClientError::Rpc { code, message });
        }
    };

    let [value]: [Value; 1] = values.try_into().map_err(|_| {
        ClientError::MalformedResponse("multicall entry must contain one result".to_owned())
    })?;

    match kind {
        MulticallEntryKind::GlobalStats => {
            let raw = serde_json::from_value::<RawGlobalStats>(value)
                .map_err(|error| ClientError::MalformedResponse(error.to_string()))?;
            Ok(MulticallEntry::GlobalStats(parse_global_stats(raw)?))
        }
        MulticallEntryKind::DownloadItems => {
            let raw_items = serde_json::from_value::<Vec<RawDownloadItem>>(value)
                .map_err(|error| ClientError::MalformedResponse(error.to_string()))?;
            let items = raw_items
                .into_iter()
                .map(parse_download_item)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(MulticallEntry::DownloadItems(items))
        }
        MulticallEntryKind::SelectedDownloadDetail => {
            let raw = serde_json::from_value::<RawDownloadItem>(value)
                .map_err(|error| ClientError::MalformedResponse(error.to_string()))?;
            Ok(MulticallEntry::SelectedDownloadDetail(
                parse_download_detail(raw)?,
            ))
        }
    }
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

fn parse_download_detail(raw: RawDownloadItem) -> Result<DownloadDetail, ClientError> {
    let directory = optional_string(&raw.dir);
    let connections = parse_optional_u32("connections", &raw.connections)?;
    let piece_length = parse_optional_u64("pieceLength", &raw.piece_length)?;
    let piece_count = parse_optional_u64("numPieces", &raw.piece_count)?;
    let error_code = optional_string(&raw.error_code);
    let error_message = optional_string(&raw.error_message);
    let torrent = parse_torrent_detail(&raw)?;
    let item = parse_download_item(raw)?;
    let mut detail = DownloadDetail::new(item);

    detail.set_directory(directory);
    detail.set_connections(connections);
    detail.set_piece_length_bytes(piece_length);
    detail.set_piece_count(piece_count);
    detail.set_error_code(error_code);
    detail.set_error_message(error_message);
    detail.set_torrent(torrent);

    Ok(detail)
}

fn parse_torrent_detail(raw: &RawDownloadItem) -> Result<Option<TorrentDetail>, ClientError> {
    let info_hash = optional_string(&raw.info_hash);
    let seeder = raw.seeder == "true";
    let num_seeders = parse_optional_u32("numSeeders", &raw.num_seeders)?;

    if info_hash.is_none() && !seeder && num_seeders == 0 {
        return Ok(None);
    }

    Ok(Some(TorrentDetail::new(info_hash, seeder, num_seeders)))
}

fn parse_global_stats(raw: RawGlobalStats) -> Result<GlobalStats, ClientError> {
    Ok(GlobalStats::new(
        parse_u64("downloadSpeed", &raw.download_speed)?,
        parse_u64("uploadSpeed", &raw.upload_speed)?,
        parse_u32("numActive", &raw.active_downloads)?,
        parse_u32("numWaiting", &raw.waiting_downloads)?,
        parse_u32("numStopped", &raw.stopped_downloads)?,
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

fn parse_optional_u32(field: &'static str, value: &str) -> Result<u32, ClientError> {
    if value.is_empty() {
        return Ok(0);
    }

    parse_u32(field, value)
}

fn optional_string(value: &str) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MulticallEntry, MulticallEntryKind, parse_add_uri_response, parse_download_detail_response,
        parse_download_items_response, parse_get_version_response, parse_gid_command_response,
        parse_global_stats_response, parse_multicall_response, parse_ok_response,
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
    fn parses_selected_download_detail_with_torrent_metadata() {
        let detail = parse_download_detail_response(
            r#"{"jsonrpc":"2.0","id":22,"result":{"gid":"abc123","status":"active","totalLength":"4096","completedLength":"2048","downloadSpeed":"1024","uploadSpeed":"128","dir":"/downloads","connections":"4","pieceLength":"262144","numPieces":"32","errorCode":"","errorMessage":"","infoHash":"0123456789abcdef","seeder":"true","numSeeders":"12","files":[{"path":"/downloads/movie.mkv","length":"4096","completedLength":"2048","selected":"true"}]}}"#,
            RequestId::new(22),
        )
        .expect("valid detail response");

        assert_eq!(detail.item().gid().as_str(), "abc123");
        assert_eq!(detail.directory(), Some("/downloads"));
        assert_eq!(detail.connections(), 4);
        assert_eq!(detail.piece_length_bytes(), 262_144);
        assert_eq!(detail.piece_count(), 32);
        let torrent = detail.torrent().expect("torrent metadata");
        assert_eq!(torrent.info_hash(), Some("0123456789abcdef"));
        assert!(torrent.seeder());
        assert_eq!(torrent.num_seeders(), 12);
    }

    #[test]
    fn parses_selected_download_detail_without_optional_metadata() {
        let detail = parse_download_detail_response(
            r#"{"jsonrpc":"2.0","id":23,"result":{"gid":"abc123","status":"waiting","totalLength":"0","completedLength":"0","files":[]}}"#,
            RequestId::new(23),
        )
        .expect("minimal detail response");

        assert_eq!(detail.directory(), None);
        assert_eq!(detail.connections(), 0);
        assert_eq!(detail.torrent(), None);
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
    fn parses_multicall_entries_into_typed_domain_values() {
        let entries = parse_multicall_response(
            r#"{"jsonrpc":"2.0","id":50,"result":[[{"downloadSpeed":"1536","uploadSpeed":"0","numActive":"1","numWaiting":"2","numStopped":"3"}],[[{"gid":"abc123","status":"active","totalLength":"10","completedLength":"5","files":[]}]]]}"#,
            RequestId::new(50),
            &[MulticallEntryKind::GlobalStats, MulticallEntryKind::DownloadItems],
        )
        .expect("valid multicall response");

        assert!(matches!(
            &entries[..],
            [
                MulticallEntry::GlobalStats(_),
                MulticallEntry::DownloadItems(_)
            ]
        ));
    }

    #[test]
    fn parses_selected_download_detail_multicall_entry() {
        let entries = parse_multicall_response(
            r#"{"jsonrpc":"2.0","id":51,"result":[[{"gid":"abc123","status":"active","totalLength":"10","completedLength":"5","connections":"2","files":[]}]]}"#,
            RequestId::new(51),
            &[MulticallEntryKind::SelectedDownloadDetail],
        )
        .expect("valid selected detail multicall");

        assert!(matches!(
            &entries[0],
            MulticallEntry::SelectedDownloadDetail(detail)
                if detail.item().gid().as_str() == "abc123" && detail.connections() == 2
        ));
    }

    #[test]
    fn maps_multicall_nested_fault_to_rpc_error() {
        let error = parse_multicall_response(
            r#"{"jsonrpc":"2.0","id":50,"result":[{"faultCode":1,"faultString":"bad nested call"}]}"#,
            RequestId::new(50),
            &[MulticallEntryKind::GlobalStats],
        )
        .expect_err("nested fault should fail");

        assert!(matches!(error, ClientError::Rpc { code: 1, .. }));
    }

    #[test]
    fn rejects_multicall_result_count_mismatch() {
        let error = parse_multicall_response(
            r#"{"jsonrpc":"2.0","id":50,"result":[]}"#,
            RequestId::new(50),
            &[MulticallEntryKind::GlobalStats],
        )
        .expect_err("count mismatch should fail");

        assert!(matches!(error, ClientError::MalformedResponse(_)));
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
