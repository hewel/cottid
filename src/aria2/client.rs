use serde_json::to_string;

use crate::aria2::domain::{DownloadItem, DownloadSnapshot, Gid, GlobalStats, VersionInfo};
use crate::aria2::errors::ClientError;
use crate::aria2::methods::{
    JsonRpcRequest, RequestId, build_add_uri_request, build_get_global_stat_request,
    build_get_version_request, build_pause_request, build_purge_stopped_request,
    build_remove_request, build_tell_active_request, build_tell_stopped_request,
    build_tell_waiting_request, build_unpause_request,
};
use crate::aria2::raw_types::{
    parse_add_uri_response, parse_download_items_response, parse_get_version_response,
    parse_gid_command_response, parse_global_stats_response, parse_ok_response,
};
use crate::config::{RpcAuth, Settings};

const CONNECTION_TEST_REQUEST_ID: RequestId = RequestId::new(1);
const GLOBAL_STATS_REQUEST_ID: RequestId = RequestId::new(2);
const TELL_ACTIVE_REQUEST_ID: RequestId = RequestId::new(3);
const TELL_WAITING_REQUEST_ID: RequestId = RequestId::new(4);
const TELL_STOPPED_REQUEST_ID: RequestId = RequestId::new(5);
const ADD_URI_REQUEST_ID: RequestId = RequestId::new(6);
const PAUSE_REQUEST_ID: RequestId = RequestId::new(7);
const UNPAUSE_REQUEST_ID: RequestId = RequestId::new(8);
const REMOVE_REQUEST_ID: RequestId = RequestId::new(9);
const PURGE_STOPPED_REQUEST_ID: RequestId = RequestId::new(10);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionTest {
    version: VersionInfo,
}

impl ConnectionTest {
    #[cfg(test)]
    pub fn new(version: VersionInfo) -> Self {
        Self { version }
    }

    pub fn version(&self) -> &VersionInfo {
        &self.version
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpPost {
    endpoint: String,
    body: String,
}

impl HttpPost {
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn body(&self) -> &str {
        &self.body
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    status: u16,
    body: String,
}

impl HttpResponse {
    #[cfg(test)]
    pub fn ok(body: impl Into<String>) -> Self {
        Self {
            status: 200,
            body: body.into(),
        }
    }

    #[cfg(test)]
    pub fn with_status(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }
}

pub trait Transport {
    fn post(&self, request: HttpPost) -> Result<HttpResponse, ClientError>;
}

pub fn test_connection(settings: Settings) -> Result<ConnectionTest, ClientError> {
    let transport = ReqwestTransport::new();
    test_connection_with_transport(&settings, &transport)
}

pub fn fetch_download_snapshot(settings: Settings) -> Result<DownloadSnapshot, ClientError> {
    let transport = ReqwestTransport::new();
    fetch_download_snapshot_with_transport(&settings, &transport)
}

pub fn add_uri(settings: Settings, uri: String) -> Result<Gid, ClientError> {
    let transport = ReqwestTransport::new();
    add_uri_with_transport(&settings, &transport, &uri)
}

pub fn pause(settings: Settings, gid: Gid) -> Result<Gid, ClientError> {
    let transport = ReqwestTransport::new();
    pause_with_transport(&settings, &transport, &gid)
}

pub fn unpause(settings: Settings, gid: Gid) -> Result<Gid, ClientError> {
    let transport = ReqwestTransport::new();
    unpause_with_transport(&settings, &transport, &gid)
}

pub fn remove(settings: Settings, gid: Gid) -> Result<Gid, ClientError> {
    let transport = ReqwestTransport::new();
    remove_with_transport(&settings, &transport, &gid)
}

pub fn purge_stopped(settings: Settings) -> Result<(), ClientError> {
    let transport = ReqwestTransport::new();
    purge_stopped_with_transport(&settings, &transport)
}

pub fn test_connection_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<ConnectionTest, ClientError> {
    let request = build_get_version_request(CONNECTION_TEST_REQUEST_ID, secret(settings));
    let body = send_rpc_request(settings, transport, request)?;
    let version = parse_get_version_response(&body, CONNECTION_TEST_REQUEST_ID)?;

    Ok(ConnectionTest { version })
}

pub fn fetch_global_stats_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<GlobalStats, ClientError> {
    let request = build_get_global_stat_request(GLOBAL_STATS_REQUEST_ID, secret(settings));
    let body = send_rpc_request(settings, transport, request)?;

    parse_global_stats_response(&body, GLOBAL_STATS_REQUEST_ID)
}

pub fn fetch_download_snapshot_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<DownloadSnapshot, ClientError> {
    let global_stats = fetch_global_stats_with_transport(settings, transport)?;
    let secret = secret(settings);

    let active = fetch_download_items(
        settings,
        transport,
        build_tell_active_request(TELL_ACTIVE_REQUEST_ID, secret),
        TELL_ACTIVE_REQUEST_ID,
    )?;
    let waiting = fetch_download_items(
        settings,
        transport,
        build_tell_waiting_request(TELL_WAITING_REQUEST_ID, secret),
        TELL_WAITING_REQUEST_ID,
    )?;
    let stopped = fetch_download_items(
        settings,
        transport,
        build_tell_stopped_request(TELL_STOPPED_REQUEST_ID, secret),
        TELL_STOPPED_REQUEST_ID,
    )?;

    let mut items = active;
    items.extend(waiting);
    items.extend(stopped);

    Ok(DownloadSnapshot::new(global_stats, items))
}

pub fn add_uri_with_transport(
    settings: &Settings,
    transport: &impl Transport,
    uri: &str,
) -> Result<Gid, ClientError> {
    let request = build_add_uri_request(ADD_URI_REQUEST_ID, secret(settings), uri);
    let body = send_rpc_request(settings, transport, request)?;

    parse_add_uri_response(&body, ADD_URI_REQUEST_ID)
}

pub fn pause_with_transport(
    settings: &Settings,
    transport: &impl Transport,
    gid: &Gid,
) -> Result<Gid, ClientError> {
    let request = build_pause_request(PAUSE_REQUEST_ID, secret(settings), gid);
    let body = send_rpc_request(settings, transport, request)?;

    parse_gid_command_response(&body, PAUSE_REQUEST_ID)
}

pub fn unpause_with_transport(
    settings: &Settings,
    transport: &impl Transport,
    gid: &Gid,
) -> Result<Gid, ClientError> {
    let request = build_unpause_request(UNPAUSE_REQUEST_ID, secret(settings), gid);
    let body = send_rpc_request(settings, transport, request)?;

    parse_gid_command_response(&body, UNPAUSE_REQUEST_ID)
}

pub fn remove_with_transport(
    settings: &Settings,
    transport: &impl Transport,
    gid: &Gid,
) -> Result<Gid, ClientError> {
    let request = build_remove_request(REMOVE_REQUEST_ID, secret(settings), gid);
    let body = send_rpc_request(settings, transport, request)?;

    parse_gid_command_response(&body, REMOVE_REQUEST_ID)
}

pub fn purge_stopped_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<(), ClientError> {
    let request = build_purge_stopped_request(PURGE_STOPPED_REQUEST_ID, secret(settings));
    let body = send_rpc_request(settings, transport, request)?;

    parse_ok_response(&body, PURGE_STOPPED_REQUEST_ID)
}

fn fetch_download_items(
    settings: &Settings,
    transport: &impl Transport,
    request: JsonRpcRequest,
    request_id: RequestId,
) -> Result<Vec<DownloadItem>, ClientError> {
    let body = send_rpc_request(settings, transport, request)?;

    parse_download_items_response(&body, request_id)
}

fn send_rpc_request(
    settings: &Settings,
    transport: &impl Transport,
    request: JsonRpcRequest,
) -> Result<String, ClientError> {
    let body =
        to_string(&request).map_err(|error| ClientError::MalformedResponse(error.to_string()))?;
    let response = transport.post(HttpPost {
        endpoint: settings.endpoint().to_owned(),
        body,
    })?;

    if !(200..=299).contains(&response.status) {
        return Err(ClientError::HttpStatus(response.status));
    }

    Ok(response.body)
}

fn secret(settings: &Settings) -> Option<&crate::config::Secret> {
    match settings.auth() {
        RpcAuth::NoSecret => None,
        RpcAuth::SessionSecret(secret) => Some(secret),
    }
}

struct ReqwestTransport {
    client: reqwest::blocking::Client,
}

impl ReqwestTransport {
    fn new() -> Self {
        Self {
            client: reqwest::blocking::Client::new(),
        }
    }
}

impl Transport for ReqwestTransport {
    fn post(&self, request: HttpPost) -> Result<HttpResponse, ClientError> {
        let response = self
            .client
            .post(request.endpoint())
            .header("content-type", "application/json")
            .body(request.body().to_owned())
            .send()
            .map_err(|error| ClientError::Transport(error.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .map_err(|error| ClientError::Transport(error.to_string()))?;

        Ok(HttpResponse { status, body })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use serde_json::Value;

    use super::{
        HttpPost, HttpResponse, Transport, add_uri_with_transport,
        fetch_download_snapshot_with_transport, fetch_global_stats_with_transport,
        pause_with_transport, purge_stopped_with_transport, remove_with_transport,
        test_connection_with_transport, unpause_with_transport,
    };
    use crate::aria2::domain::Gid;
    use crate::aria2::errors::ClientError;
    use crate::config::{RpcAuth, Secret, Settings, SettingsDraft};

    #[derive(Debug)]
    struct FakeTransport {
        responses: RefCell<Vec<Result<HttpResponse, ClientError>>>,
        posts: RefCell<Vec<HttpPost>>,
    }

    impl FakeTransport {
        fn returning(response: Result<HttpResponse, ClientError>) -> Self {
            Self {
                responses: RefCell::new(vec![response]),
                posts: RefCell::new(Vec::new()),
            }
        }

        fn returning_sequence(responses: Vec<Result<HttpResponse, ClientError>>) -> Self {
            Self {
                responses: RefCell::new(responses),
                posts: RefCell::new(Vec::new()),
            }
        }
    }

    impl Transport for FakeTransport {
        fn post(&self, request: HttpPost) -> Result<HttpResponse, ClientError> {
            self.posts.borrow_mut().push(request);
            let mut responses = self.responses.borrow_mut();
            if responses.len() > 1 {
                responses.remove(0)
            } else {
                responses[0].clone()
            }
        }
    }

    #[test]
    fn connection_test_posts_json_rpc_get_version() {
        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":1,"result":{"version":"1.37.0"}}"#,
        )));

        let result = test_connection_with_transport(&settings, &transport)
            .expect("connection test should pass");

        assert_eq!(result.version().version(), "1.37.0");

        let posts = transport.posts.borrow();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].endpoint(), "http://localhost:6800/jsonrpc");

        let body: Value = serde_json::from_str(posts[0].body()).expect("request body is JSON");
        assert_eq!(body["jsonrpc"], "2.0");
        assert_eq!(body["method"], "aria2.getVersion");
        assert_eq!(body["id"], 1);
    }

    #[test]
    fn global_stats_posts_json_rpc_get_global_stat() {
        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":2,"result":{"downloadSpeed":"1536","uploadSpeed":"512","numActive":"2","numWaiting":"3","numStopped":"4"}}"#,
        )));

        let stats = fetch_global_stats_with_transport(&settings, &transport)
            .expect("global stats should parse");

        assert_eq!(stats.download_speed_bytes_per_second(), 1536);
        assert_eq!(stats.upload_speed_bytes_per_second(), 512);
        assert_eq!(stats.active_downloads(), 2);

        let posts = transport.posts.borrow();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].endpoint(), "http://localhost:6800/jsonrpc");

        let body: Value = serde_json::from_str(posts[0].body()).expect("request body is JSON");
        assert_eq!(body["jsonrpc"], "2.0");
        assert_eq!(body["method"], "aria2.getGlobalStat");
        assert_eq!(body["id"], 2);
    }

    #[test]
    fn global_stats_maps_http_rpc_and_malformed_responses() {
        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::with_status(503, "busy")));

        assert_eq!(
            fetch_global_stats_with_transport(&settings, &transport),
            Err(ClientError::HttpStatus(503))
        );

        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":2,"error":{"code":1,"message":"bad token"}}"#,
        )));

        assert!(matches!(
            fetch_global_stats_with_transport(&settings, &transport),
            Err(ClientError::Rpc { code: 1, .. })
        ));

        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":2,"result":{"downloadSpeed":"fast","uploadSpeed":"0","numActive":"0","numWaiting":"0","numStopped":"0"}}"#,
        )));

        assert!(matches!(
            fetch_global_stats_with_transport(&settings, &transport),
            Err(ClientError::MalformedResponse(_))
        ));
    }

    #[test]
    fn download_snapshot_fetches_stats_active_waiting_and_stopped() {
        let settings = Settings::default();
        let transport = FakeTransport::returning_sequence(vec![
            Ok(HttpResponse::ok(
                r#"{"jsonrpc":"2.0","id":2,"result":{"downloadSpeed":"1536","uploadSpeed":"512","numActive":"1","numWaiting":"1","numStopped":"1"}}"#,
            )),
            Ok(HttpResponse::ok(
                r#"{"jsonrpc":"2.0","id":3,"result":[{"gid":"active-gid","status":"active","totalLength":"2000","completedLength":"1000","downloadSpeed":"500","uploadSpeed":"0","files":[]}]}"#,
            )),
            Ok(HttpResponse::ok(
                r#"{"jsonrpc":"2.0","id":4,"result":[{"gid":"waiting-gid","status":"waiting","totalLength":"3000","completedLength":"0","files":[]}]}"#,
            )),
            Ok(HttpResponse::ok(
                r#"{"jsonrpc":"2.0","id":5,"result":[{"gid":"stopped-gid","status":"complete","totalLength":"4000","completedLength":"4000","files":[]}]}"#,
            )),
        ]);

        let snapshot = fetch_download_snapshot_with_transport(&settings, &transport)
            .expect("snapshot should parse");

        assert_eq!(snapshot.global_stats().active_downloads(), 1);
        assert_eq!(snapshot.items().len(), 3);
        assert_eq!(snapshot.items()[0].gid().as_str(), "active-gid");
        assert_eq!(snapshot.items()[1].gid().as_str(), "waiting-gid");
        assert_eq!(snapshot.items()[2].gid().as_str(), "stopped-gid");

        let methods = transport
            .posts
            .borrow()
            .iter()
            .map(|post| {
                let body: Value = serde_json::from_str(post.body()).expect("request body is JSON");
                body["method"].as_str().expect("method").to_owned()
            })
            .collect::<Vec<_>>();

        assert_eq!(
            methods,
            [
                "aria2.getGlobalStat",
                "aria2.tellActive",
                "aria2.tellWaiting",
                "aria2.tellStopped"
            ]
        );
    }

    #[test]
    fn download_snapshot_returns_error_when_any_refresh_call_fails() {
        let settings = Settings::default();
        let transport = FakeTransport::returning_sequence(vec![
            Ok(HttpResponse::ok(
                r#"{"jsonrpc":"2.0","id":2,"result":{"downloadSpeed":"0","uploadSpeed":"0","numActive":"0","numWaiting":"0","numStopped":"0"}}"#,
            )),
            Err(ClientError::Transport("connection refused".to_owned())),
        ]);

        assert!(matches!(
            fetch_download_snapshot_with_transport(&settings, &transport),
            Err(ClientError::Transport(_))
        ));
    }

    #[test]
    fn add_uri_posts_json_rpc_add_uri_and_returns_gid() {
        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":6,"result":"new-gid"}"#,
        )));

        let gid = add_uri_with_transport(&settings, &transport, "https://example.test/file")
            .expect("addUri should return gid");

        assert_eq!(gid.as_str(), "new-gid");

        let posts = transport.posts.borrow();
        let body: Value = serde_json::from_str(posts[0].body()).expect("request body is JSON");
        assert_eq!(body["method"], "aria2.addUri");
        assert_eq!(body["id"], 6);
        assert_eq!(body["params"][0][0], "https://example.test/file");
    }

    #[test]
    fn add_uri_maps_rpc_and_malformed_responses() {
        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":6,"error":{"code":1,"message":"bad uri"}}"#,
        )));

        assert!(matches!(
            add_uri_with_transport(&settings, &transport, "https://example.test/file"),
            Err(ClientError::Rpc { code: 1, .. })
        ));

        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":6,"result":""}"#,
        )));

        assert!(matches!(
            add_uri_with_transport(&settings, &transport, "https://example.test/file"),
            Err(ClientError::MalformedResponse(_))
        ));
    }

    #[test]
    fn pause_unpause_and_remove_post_gid_commands() {
        let settings = Settings::default();
        let gid = Gid::new("abc123").expect("valid gid");

        let pause_transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":7,"result":"abc123"}"#,
        )));
        pause_with_transport(&settings, &pause_transport, &gid).expect("pause succeeds");

        let unpause_transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":8,"result":"abc123"}"#,
        )));
        unpause_with_transport(&settings, &unpause_transport, &gid).expect("unpause succeeds");

        let remove_transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":9,"result":"abc123"}"#,
        )));
        remove_with_transport(&settings, &remove_transport, &gid).expect("remove succeeds");

        let pause_body: Value =
            serde_json::from_str(pause_transport.posts.borrow()[0].body()).expect("json");
        let unpause_body: Value =
            serde_json::from_str(unpause_transport.posts.borrow()[0].body()).expect("json");
        let remove_body: Value =
            serde_json::from_str(remove_transport.posts.borrow()[0].body()).expect("json");

        assert_eq!(pause_body["method"], "aria2.pause");
        assert_eq!(unpause_body["method"], "aria2.unpause");
        assert_eq!(remove_body["method"], "aria2.remove");
        assert_eq!(pause_body["params"][0], "abc123");
    }

    #[test]
    fn purge_stopped_posts_command_and_requires_ok() {
        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":10,"result":"OK"}"#,
        )));

        purge_stopped_with_transport(&settings, &transport).expect("purge succeeds");

        let body: Value = serde_json::from_str(transport.posts.borrow()[0].body()).expect("json");
        assert_eq!(body["method"], "aria2.purgeDownloadResult");
    }

    #[test]
    fn connection_test_inserts_secret_token_without_debug_leak() {
        let mut draft = SettingsDraft::from_settings(&Settings::default());
        draft.set_auth(crate::config::RpcAuthDraft::SessionSecret);
        draft.set_secret("super-secret");
        let settings = draft.apply().expect("valid settings");
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":1,"result":{"version":"1.37.0"}}"#,
        )));

        test_connection_with_transport(&settings, &transport).expect("connection test should pass");

        let posts = transport.posts.borrow();
        let body: Value = serde_json::from_str(posts[0].body()).expect("request body is JSON");
        assert_eq!(body["params"][0], "token:super-secret");
        assert!(!format!("{settings:?}").contains("super-secret"));
    }

    #[test]
    fn connection_test_maps_transport_and_http_errors() {
        let settings = Settings::default();
        let transport =
            FakeTransport::returning(Err(ClientError::Transport("connection refused".to_owned())));

        assert!(matches!(
            test_connection_with_transport(&settings, &transport),
            Err(ClientError::Transport(_))
        ));

        let transport = FakeTransport::returning(Ok(HttpResponse::with_status(500, "oops")));

        assert_eq!(
            test_connection_with_transport(&settings, &transport),
            Err(ClientError::HttpStatus(500))
        );
    }

    #[test]
    fn connection_test_maps_rpc_and_malformed_responses() {
        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":1,"message":"bad token"}}"#,
        )));

        assert!(matches!(
            test_connection_with_transport(&settings, &transport),
            Err(ClientError::Rpc { code: 1, .. })
        ));

        let transport = FakeTransport::returning(Ok(HttpResponse::ok("not json")));

        assert!(matches!(
            test_connection_with_transport(&settings, &transport),
            Err(ClientError::MalformedResponse(_))
        ));
    }

    #[test]
    fn no_raw_json_types_are_needed_by_callers() {
        fn assert_typed_result(result: Result<super::ConnectionTest, ClientError>) {
            let _ = result;
        }

        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":1,"result":{"version":"1.37.0"}}"#,
        )));

        assert_typed_result(test_connection_with_transport(&settings, &transport));
    }

    #[test]
    fn auth_debug_redacts_session_secret() {
        let auth = RpcAuth::SessionSecret(Secret::session("super-secret"));

        assert!(!format!("{auth:?}").contains("super-secret"));
    }
}
