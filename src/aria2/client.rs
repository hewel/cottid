use serde_json::to_string;

use crate::aria2::domain::{GlobalStats, VersionInfo};
use crate::aria2::errors::ClientError;
use crate::aria2::methods::{
    JsonRpcRequest, RequestId, build_get_global_stat_request, build_get_version_request,
};
use crate::aria2::raw_types::{parse_get_version_response, parse_global_stats_response};
use crate::config::{RpcAuth, Settings};

const CONNECTION_TEST_REQUEST_ID: RequestId = RequestId::new(1);
const GLOBAL_STATS_REQUEST_ID: RequestId = RequestId::new(2);

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

pub fn fetch_global_stats(settings: Settings) -> Result<GlobalStats, ClientError> {
    let transport = ReqwestTransport::new();
    fetch_global_stats_with_transport(&settings, &transport)
}

pub fn test_connection_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<ConnectionTest, ClientError> {
    let secret = match settings.auth() {
        RpcAuth::NoSecret => None,
        RpcAuth::SessionSecret(secret) => Some(secret),
    };
    let request = build_get_version_request(CONNECTION_TEST_REQUEST_ID, secret);
    let body = send_rpc_request(settings, transport, request)?;
    let version = parse_get_version_response(&body, CONNECTION_TEST_REQUEST_ID)?;

    Ok(ConnectionTest { version })
}

pub fn fetch_global_stats_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<GlobalStats, ClientError> {
    let secret = match settings.auth() {
        RpcAuth::NoSecret => None,
        RpcAuth::SessionSecret(secret) => Some(secret),
    };
    let request = build_get_global_stat_request(GLOBAL_STATS_REQUEST_ID, secret);
    let body = send_rpc_request(settings, transport, request)?;

    parse_global_stats_response(&body, GLOBAL_STATS_REQUEST_ID)
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
        HttpPost, HttpResponse, Transport, fetch_global_stats_with_transport,
        test_connection_with_transport,
    };
    use crate::aria2::errors::ClientError;
    use crate::config::{RpcAuth, Secret, Settings, SettingsDraft};

    #[derive(Debug)]
    struct FakeTransport {
        response: Result<HttpResponse, ClientError>,
        posts: RefCell<Vec<HttpPost>>,
    }

    impl FakeTransport {
        fn returning(response: Result<HttpResponse, ClientError>) -> Self {
            Self {
                response,
                posts: RefCell::new(Vec::new()),
            }
        }
    }

    impl Transport for FakeTransport {
        fn post(&self, request: HttpPost) -> Result<HttpResponse, ClientError> {
            self.posts.borrow_mut().push(request);
            self.response.clone()
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
