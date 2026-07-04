use serde_json::to_string;

#[cfg(test)]
use crate::aria2::domain::GlobalStats;
use crate::aria2::domain::{
    AddUriOptions, DownloadSnapshot, Gid, RuntimeGlobalOptions, VersionInfo,
};
use crate::aria2::errors::ClientError;
#[cfg(test)]
use crate::aria2::methods::build_get_global_stat_request;
use crate::aria2::methods::{
    JsonRpcRequest, RequestId, build_add_uri_request, build_change_global_option_request,
    build_get_global_option_request, build_get_global_stat_call, build_get_version_request,
    build_list_notifications_request, build_multicall_request, build_pause_request,
    build_purge_stopped_request, build_remove_request, build_save_session_request,
    build_shutdown_request, build_tell_active_call, build_tell_status_call,
    build_tell_stopped_call, build_tell_waiting_call, build_unpause_request,
};
#[cfg(test)]
use crate::aria2::raw_types::parse_global_stats_response;
use crate::aria2::raw_types::{
    MulticallEntry, MulticallEntryKind, parse_add_uri_response, parse_get_version_response,
    parse_gid_command_response, parse_multicall_response, parse_ok_response,
};
use crate::config::{RpcAuth, Settings};

pub const DEFAULT_STOPPED_REFRESH_LIMIT: u64 = 50;

const CONNECTION_TEST_REQUEST_ID: RequestId = RequestId::new(1);
const GLOBAL_STATS_REQUEST_ID: RequestId = RequestId::new(2);
const ADD_URI_REQUEST_ID: RequestId = RequestId::new(6);
const PAUSE_REQUEST_ID: RequestId = RequestId::new(7);
const UNPAUSE_REQUEST_ID: RequestId = RequestId::new(8);
const REMOVE_REQUEST_ID: RequestId = RequestId::new(9);
const PURGE_STOPPED_REQUEST_ID: RequestId = RequestId::new(10);
const SNAPSHOT_MULTICALL_REQUEST_ID: RequestId = RequestId::new(11);
const GET_GLOBAL_OPTION_REQUEST_ID: RequestId = RequestId::new(12);
const CHANGE_GLOBAL_OPTION_REQUEST_ID: RequestId = RequestId::new(13);
const LIST_NOTIFICATIONS_REQUEST_ID: RequestId = RequestId::new(14);
const SAVE_SESSION_REQUEST_ID: RequestId = RequestId::new(15);
const SHUTDOWN_REQUEST_ID: RequestId = RequestId::new(16);

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchRefreshRequest {
    include_active: bool,
    include_waiting: bool,
    include_stopped: bool,
    stopped_limit: u64,
    selected_gid: Option<Gid>,
}

impl Default for BatchRefreshRequest {
    fn default() -> Self {
        Self {
            include_active: true,
            include_waiting: true,
            include_stopped: true,
            stopped_limit: DEFAULT_STOPPED_REFRESH_LIMIT,
            selected_gid: None,
        }
    }
}

impl BatchRefreshRequest {
    pub fn stats_only() -> Self {
        Self {
            include_active: false,
            include_waiting: false,
            include_stopped: false,
            stopped_limit: DEFAULT_STOPPED_REFRESH_LIMIT,
            selected_gid: None,
        }
    }

    pub fn include_all_summaries(&mut self) {
        self.include_active = true;
        self.include_waiting = true;
        self.include_stopped = true;
    }

    pub fn include_active(&self) -> bool {
        self.include_active
    }

    pub fn set_include_active(&mut self, include_active: bool) {
        self.include_active = include_active;
    }

    pub fn include_waiting(&self) -> bool {
        self.include_waiting
    }

    pub fn set_include_waiting(&mut self, include_waiting: bool) {
        self.include_waiting = include_waiting;
    }

    pub fn include_stopped(&self) -> bool {
        self.include_stopped
    }

    pub fn set_include_stopped(&mut self, include_stopped: bool) {
        self.include_stopped = include_stopped;
    }

    pub fn stopped_limit(&self) -> u64 {
        self.stopped_limit
    }

    pub fn selected_gid(&self) -> Option<&Gid> {
        self.selected_gid.as_ref()
    }

    pub fn set_selected_gid(&mut self, selected_gid: Option<Gid>) {
        self.selected_gid = selected_gid;
    }

    pub fn refreshes_anything(&self) -> bool {
        self.include_active
            || self.include_waiting
            || self.include_stopped
            || self.selected_gid.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct Aria2Client {
    settings: Settings,
}

impl Aria2Client {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub async fn test_connection(&self) -> Result<ConnectionTest, ClientError> {
        let transport = ReqwestTransport::new();
        let request = build_get_version_request(CONNECTION_TEST_REQUEST_ID, self.secret());
        let body = send_rpc_request_async(&self.settings, &transport, request).await?;
        let version = parse_get_version_response(&body, CONNECTION_TEST_REQUEST_ID)?;

        Ok(ConnectionTest { version })
    }

    pub async fn fetch_download_snapshot(
        &self,
        request: BatchRefreshRequest,
    ) -> Result<DownloadSnapshot, ClientError> {
        let (rpc_request, entry_kinds) = self.build_snapshot_multicall(&request);
        let body = send_rpc_request_preferred(&self.settings, rpc_request).await?;

        parse_snapshot_multicall(&body, &entry_kinds)
    }

    #[cfg(test)]
    fn test_connection_with_transport(
        &self,
        transport: &impl Transport,
    ) -> Result<ConnectionTest, ClientError> {
        let request = build_get_version_request(CONNECTION_TEST_REQUEST_ID, self.secret());
        let body = send_rpc_request(&self.settings, transport, request)?;
        let version = parse_get_version_response(&body, CONNECTION_TEST_REQUEST_ID)?;

        Ok(ConnectionTest { version })
    }

    #[cfg(test)]
    fn fetch_global_stats_with_transport(
        &self,
        transport: &impl Transport,
    ) -> Result<GlobalStats, ClientError> {
        let request = build_get_global_stat_request(GLOBAL_STATS_REQUEST_ID, self.secret());
        let body = send_rpc_request(&self.settings, transport, request)?;

        parse_global_stats_response(&body, GLOBAL_STATS_REQUEST_ID)
    }

    #[cfg(test)]
    fn fetch_download_snapshot_with_transport(
        &self,
        transport: &impl Transport,
        request: &BatchRefreshRequest,
    ) -> Result<DownloadSnapshot, ClientError> {
        let (rpc_request, entry_kinds) = self.build_snapshot_multicall(request);
        let body = send_rpc_request(&self.settings, transport, rpc_request)?;

        parse_snapshot_multicall(&body, &entry_kinds)
    }

    fn build_snapshot_multicall(
        &self,
        request: &BatchRefreshRequest,
    ) -> (JsonRpcRequest, Vec<MulticallEntryKind>) {
        let secret = self.secret();
        let mut calls = vec![build_get_global_stat_call(secret)];
        let mut entry_kinds = vec![MulticallEntryKind::GlobalStats];

        if request.include_active {
            calls.push(build_tell_active_call(secret));
            entry_kinds.push(MulticallEntryKind::DownloadItems);
        }
        if request.include_waiting {
            calls.push(build_tell_waiting_call(secret));
            entry_kinds.push(MulticallEntryKind::DownloadItems);
        }
        if request.include_stopped {
            calls.push(build_tell_stopped_call(secret, request.stopped_limit()));
            entry_kinds.push(MulticallEntryKind::DownloadItems);
        }
        if let Some(gid) = request.selected_gid() {
            calls.push(build_tell_status_call(secret, gid));
            entry_kinds.push(MulticallEntryKind::SelectedDownloadDetail);
        }

        (
            build_multicall_request(SNAPSHOT_MULTICALL_REQUEST_ID, calls),
            entry_kinds,
        )
    }

    fn secret(&self) -> Option<&crate::config::Secret> {
        secret(&self.settings)
    }
}

#[cfg(test)]
pub trait Transport {
    fn post(&self, request: HttpPost) -> Result<HttpResponse, ClientError>;
}

pub async fn test_connection(settings: Settings) -> Result<ConnectionTest, ClientError> {
    Aria2Client::new(settings).test_connection().await
}

pub async fn test_websocket_notifications(settings: Settings) -> Result<(), ClientError> {
    if !settings.websocket_enabled() {
        return Ok(());
    }

    let request =
        build_list_notifications_request(LIST_NOTIFICATIONS_REQUEST_ID, secret(&settings));
    crate::aria2::websocket::send_rpc_request(&settings, request)
        .await
        .map(|_| ())
}

#[expect(dead_code, reason = "kept as the default app-facing snapshot wrapper")]
pub async fn fetch_download_snapshot(settings: Settings) -> Result<DownloadSnapshot, ClientError> {
    fetch_download_snapshot_with_request(settings, BatchRefreshRequest::default()).await
}

pub async fn fetch_download_snapshot_with_request(
    settings: Settings,
    request: BatchRefreshRequest,
) -> Result<DownloadSnapshot, ClientError> {
    Aria2Client::new(settings)
        .fetch_download_snapshot(request)
        .await
}

pub async fn get_runtime_global_options(
    settings: Settings,
) -> Result<RuntimeGlobalOptions, ClientError> {
    let transport = ReqwestTransport::new();
    let request = build_get_global_option_request(GET_GLOBAL_OPTION_REQUEST_ID, secret(&settings));
    let body = send_rpc_request_async(&settings, &transport, request).await?;

    crate::aria2::raw_types::parse_runtime_global_options_response(
        &body,
        GET_GLOBAL_OPTION_REQUEST_ID,
    )
}

pub async fn change_runtime_global_options(
    settings: Settings,
    options: RuntimeGlobalOptions,
) -> Result<(), ClientError> {
    let transport = ReqwestTransport::new();
    let request = build_change_global_option_request(
        CHANGE_GLOBAL_OPTION_REQUEST_ID,
        secret(&settings),
        options.into_rpc_options(),
    );
    let body = send_rpc_request_async(&settings, &transport, request).await?;

    parse_ok_response(&body, CHANGE_GLOBAL_OPTION_REQUEST_ID)
}

pub async fn add_uri(
    settings: Settings,
    uri: String,
    options: AddUriOptions,
) -> Result<Gid, ClientError> {
    let request = build_add_uri_request(
        ADD_URI_REQUEST_ID,
        secret(&settings),
        &uri,
        options.into_rpc_options(),
    );
    let body = send_rpc_request_preferred(&settings, request).await?;

    parse_add_uri_response(&body, ADD_URI_REQUEST_ID)
}

pub async fn pause(settings: Settings, gid: Gid) -> Result<Gid, ClientError> {
    let request = build_pause_request(PAUSE_REQUEST_ID, secret(&settings), &gid);
    let body = send_rpc_request_preferred(&settings, request).await?;

    parse_gid_command_response(&body, PAUSE_REQUEST_ID)
}

pub async fn unpause(settings: Settings, gid: Gid) -> Result<Gid, ClientError> {
    let request = build_unpause_request(UNPAUSE_REQUEST_ID, secret(&settings), &gid);
    let body = send_rpc_request_preferred(&settings, request).await?;

    parse_gid_command_response(&body, UNPAUSE_REQUEST_ID)
}

pub async fn remove(settings: Settings, gid: Gid) -> Result<Gid, ClientError> {
    let request = build_remove_request(REMOVE_REQUEST_ID, secret(&settings), &gid);
    let body = send_rpc_request_preferred(&settings, request).await?;

    parse_gid_command_response(&body, REMOVE_REQUEST_ID)
}

pub async fn purge_stopped(settings: Settings) -> Result<(), ClientError> {
    let request = build_purge_stopped_request(PURGE_STOPPED_REQUEST_ID, secret(&settings));
    let body = send_rpc_request_preferred(&settings, request).await?;

    parse_ok_response(&body, PURGE_STOPPED_REQUEST_ID)
}

pub async fn save_session(settings: Settings) -> Result<(), ClientError> {
    let request = build_save_session_request(SAVE_SESSION_REQUEST_ID, secret(&settings));
    let body = send_rpc_request_preferred(&settings, request).await?;

    parse_ok_response(&body, SAVE_SESSION_REQUEST_ID)
}

pub async fn shutdown(settings: Settings) -> Result<(), ClientError> {
    let request = build_shutdown_request(SHUTDOWN_REQUEST_ID, secret(&settings));
    let body = send_rpc_request_preferred(&settings, request).await?;

    parse_ok_response(&body, SHUTDOWN_REQUEST_ID)
}

#[cfg(test)]
pub fn test_connection_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<ConnectionTest, ClientError> {
    Aria2Client::new(settings.clone()).test_connection_with_transport(transport)
}

#[cfg(test)]
pub fn fetch_global_stats_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<GlobalStats, ClientError> {
    Aria2Client::new(settings.clone()).fetch_global_stats_with_transport(transport)
}

#[cfg(test)]
pub fn fetch_download_snapshot_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<DownloadSnapshot, ClientError> {
    fetch_download_snapshot_with_transport_and_request(
        settings,
        transport,
        &BatchRefreshRequest::default(),
    )
}

#[cfg(test)]
pub fn fetch_download_snapshot_with_transport_and_request(
    settings: &Settings,
    transport: &impl Transport,
    request: &BatchRefreshRequest,
) -> Result<DownloadSnapshot, ClientError> {
    Aria2Client::new(settings.clone()).fetch_download_snapshot_with_transport(transport, request)
}

#[cfg(test)]
pub fn add_uri_with_transport(
    settings: &Settings,
    transport: &impl Transport,
    uri: &str,
    options: AddUriOptions,
) -> Result<Gid, ClientError> {
    let request = build_add_uri_request(
        ADD_URI_REQUEST_ID,
        secret(settings),
        uri,
        options.into_rpc_options(),
    );
    let body = send_rpc_request(settings, transport, request)?;

    parse_add_uri_response(&body, ADD_URI_REQUEST_ID)
}

#[cfg(test)]
pub fn get_runtime_global_options_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<RuntimeGlobalOptions, ClientError> {
    let request = build_get_global_option_request(GET_GLOBAL_OPTION_REQUEST_ID, secret(settings));
    let body = send_rpc_request(settings, transport, request)?;

    crate::aria2::raw_types::parse_runtime_global_options_response(
        &body,
        GET_GLOBAL_OPTION_REQUEST_ID,
    )
}

#[cfg(test)]
pub fn change_runtime_global_options_with_transport(
    settings: &Settings,
    transport: &impl Transport,
    options: RuntimeGlobalOptions,
) -> Result<(), ClientError> {
    let request = build_change_global_option_request(
        CHANGE_GLOBAL_OPTION_REQUEST_ID,
        secret(settings),
        options.into_rpc_options(),
    );
    let body = send_rpc_request(settings, transport, request)?;

    parse_ok_response(&body, CHANGE_GLOBAL_OPTION_REQUEST_ID)
}

#[cfg(test)]
pub fn pause_with_transport(
    settings: &Settings,
    transport: &impl Transport,
    gid: &Gid,
) -> Result<Gid, ClientError> {
    let request = build_pause_request(PAUSE_REQUEST_ID, secret(settings), gid);
    let body = send_rpc_request(settings, transport, request)?;

    parse_gid_command_response(&body, PAUSE_REQUEST_ID)
}

#[cfg(test)]
pub fn unpause_with_transport(
    settings: &Settings,
    transport: &impl Transport,
    gid: &Gid,
) -> Result<Gid, ClientError> {
    let request = build_unpause_request(UNPAUSE_REQUEST_ID, secret(settings), gid);
    let body = send_rpc_request(settings, transport, request)?;

    parse_gid_command_response(&body, UNPAUSE_REQUEST_ID)
}

#[cfg(test)]
pub fn remove_with_transport(
    settings: &Settings,
    transport: &impl Transport,
    gid: &Gid,
) -> Result<Gid, ClientError> {
    let request = build_remove_request(REMOVE_REQUEST_ID, secret(settings), gid);
    let body = send_rpc_request(settings, transport, request)?;

    parse_gid_command_response(&body, REMOVE_REQUEST_ID)
}

#[cfg(test)]
pub fn purge_stopped_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<(), ClientError> {
    let request = build_purge_stopped_request(PURGE_STOPPED_REQUEST_ID, secret(settings));
    let body = send_rpc_request(settings, transport, request)?;

    parse_ok_response(&body, PURGE_STOPPED_REQUEST_ID)
}

#[cfg(test)]
pub fn save_session_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<(), ClientError> {
    let request = build_save_session_request(SAVE_SESSION_REQUEST_ID, secret(settings));
    let body = send_rpc_request(settings, transport, request)?;

    parse_ok_response(&body, SAVE_SESSION_REQUEST_ID)
}

#[cfg(test)]
pub fn shutdown_with_transport(
    settings: &Settings,
    transport: &impl Transport,
) -> Result<(), ClientError> {
    let request = build_shutdown_request(SHUTDOWN_REQUEST_ID, secret(settings));
    let body = send_rpc_request(settings, transport, request)?;

    parse_ok_response(&body, SHUTDOWN_REQUEST_ID)
}

fn parse_snapshot_multicall(
    body: &str,
    entry_kinds: &[MulticallEntryKind],
) -> Result<DownloadSnapshot, ClientError> {
    let entries = parse_multicall_response(body, SNAPSHOT_MULTICALL_REQUEST_ID, entry_kinds)?;
    let mut global_stats = None;
    let mut items = Vec::new();
    let mut selected_detail = None;

    for entry in entries {
        match entry {
            MulticallEntry::GlobalStats(stats) => global_stats = Some(stats),
            MulticallEntry::DownloadItems(mut download_items) => items.append(&mut download_items),
            MulticallEntry::SelectedDownloadDetail(detail) => selected_detail = Some(detail),
        }
    }

    let global_stats = global_stats.ok_or_else(|| {
        ClientError::MalformedResponse("multicall snapshot missing global stats".to_owned())
    })?;

    Ok(DownloadSnapshot::with_selected_detail(
        global_stats,
        items,
        selected_detail,
    ))
}

#[cfg(test)]
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

async fn send_rpc_request_async(
    settings: &Settings,
    transport: &ReqwestTransport,
    request: JsonRpcRequest,
) -> Result<String, ClientError> {
    let body =
        to_string(&request).map_err(|error| ClientError::MalformedResponse(error.to_string()))?;
    let response = transport
        .post(HttpPost {
            endpoint: settings.endpoint().to_owned(),
            body,
        })
        .await?;

    if !(200..=299).contains(&response.status) {
        return Err(ClientError::HttpStatus(response.status));
    }

    Ok(response.body)
}

async fn send_rpc_request_preferred(
    settings: &Settings,
    request: JsonRpcRequest,
) -> Result<String, ClientError> {
    if settings.websocket_enabled()
        && let Ok(body) = crate::aria2::websocket::send_rpc_request(settings, request.clone()).await
    {
        return Ok(body);
    }

    let transport = ReqwestTransport::new();
    send_rpc_request_async(settings, &transport, request).await
}

fn secret(settings: &Settings) -> Option<&crate::config::Secret> {
    match settings.auth() {
        RpcAuth::NoSecret => None,
        RpcAuth::SessionSecret(secret) => Some(secret),
    }
}

struct ReqwestTransport {
    client: reqwest::Client,
}

impl ReqwestTransport {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    async fn post(&self, request: HttpPost) -> Result<HttpResponse, ClientError> {
        let response = self
            .client
            .post(request.endpoint())
            .header("content-type", "application/json")
            .body(request.body().to_owned())
            .send()
            .await
            .map_err(|error| ClientError::Transport(error.to_string()))?;

        let status = response.status().as_u16();
        let body = response
            .text()
            .await
            .map_err(|error| ClientError::Transport(error.to_string()))?;

        Ok(HttpResponse { status, body })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use serde_json::Value;

    use super::{
        BatchRefreshRequest, DEFAULT_STOPPED_REFRESH_LIMIT, HttpPost, HttpResponse, Transport,
        add_uri_with_transport, change_runtime_global_options_with_transport,
        fetch_download_snapshot_with_transport, fetch_download_snapshot_with_transport_and_request,
        fetch_global_stats_with_transport, get_runtime_global_options_with_transport,
        pause_with_transport, purge_stopped_with_transport, remove_with_transport,
        save_session_with_transport, shutdown_with_transport, test_connection_with_transport,
        unpause_with_transport,
    };
    use crate::aria2::domain::{AddUriOptions, Gid, RuntimeGlobalOptions};
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
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":11,"result":[[{"downloadSpeed":"1536","uploadSpeed":"512","numActive":"1","numWaiting":"1","numStopped":"1"}],[[{"gid":"active-gid","status":"active","totalLength":"2000","completedLength":"1000","downloadSpeed":"500","uploadSpeed":"0","files":[]}]],[[{"gid":"waiting-gid","status":"waiting","totalLength":"3000","completedLength":"0","files":[]}]],[[{"gid":"stopped-gid","status":"complete","totalLength":"4000","completedLength":"4000","files":[]}]]]}"#,
        )));

        let snapshot = fetch_download_snapshot_with_transport(&settings, &transport)
            .expect("snapshot should parse");

        assert_eq!(snapshot.global_stats().active_downloads(), 1);
        assert_eq!(snapshot.items().len(), 3);
        assert_eq!(snapshot.items()[0].gid().as_str(), "active-gid");
        assert_eq!(snapshot.items()[1].gid().as_str(), "waiting-gid");
        assert_eq!(snapshot.items()[2].gid().as_str(), "stopped-gid");

        let posts = transport.posts.borrow();
        assert_eq!(posts.len(), 1);

        let body: Value = serde_json::from_str(posts[0].body()).expect("request body is JSON");
        assert_eq!(body["jsonrpc"], "2.0");
        assert_eq!(body["method"], "system.multicall");
        assert_eq!(body["id"], 11);

        let calls = &body["params"][0];
        assert_eq!(calls[0]["methodName"], "aria2.getGlobalStat");
        assert_eq!(calls[1]["methodName"], "aria2.tellActive");
        assert_eq!(calls[2]["methodName"], "aria2.tellWaiting");
        assert_eq!(calls[3]["methodName"], "aria2.tellStopped");
        assert_eq!(calls[1]["params"][0][0], "gid");
        assert_eq!(calls[1]["params"][0][6], "dir");
        assert_eq!(calls[1]["params"][0][7], "files");
        assert_eq!(calls[2]["params"][0], 0);
        assert_eq!(calls[2]["params"][1], 1000);
        assert_eq!(calls[2]["params"][2][0], "gid");
        assert_eq!(calls[2]["params"][2][6], "dir");
        assert_eq!(calls[2]["params"][2][7], "files");
        assert_eq!(calls[3]["params"][0], 0);
        assert_eq!(calls[3]["params"][1], DEFAULT_STOPPED_REFRESH_LIMIT);
        assert_eq!(calls[3]["params"][2][0], "gid");
        assert_eq!(calls[3]["params"][2][6], "dir");
        assert_eq!(calls[3]["params"][2][7], "files");
    }

    #[test]
    fn download_snapshot_fetches_selected_detail_when_requested() {
        let settings = Settings::default();
        let mut request = BatchRefreshRequest::stats_only();
        request.set_selected_gid(Some(Gid::new("selected-gid").expect("valid gid")));
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":11,"result":[[{"downloadSpeed":"1536","uploadSpeed":"512","numActive":"1","numWaiting":"1","numStopped":"1"}],[{"gid":"selected-gid","status":"active","totalLength":"2000","completedLength":"1000","downloadSpeed":"500","uploadSpeed":"0","dir":"/downloads","connections":"3","infoHash":"abcdef","numSeeders":"9","files":[]}]]}"#,
        )));

        let snapshot =
            fetch_download_snapshot_with_transport_and_request(&settings, &transport, &request)
                .expect("snapshot should parse selected detail");

        let detail = snapshot.selected_detail().expect("selected detail");
        assert_eq!(detail.item().gid().as_str(), "selected-gid");
        assert_eq!(detail.directory(), Some("/downloads"));
        assert_eq!(detail.connections(), 3);
        assert_eq!(
            detail.torrent().and_then(|torrent| torrent.info_hash()),
            Some("abcdef")
        );

        let posts = transport.posts.borrow();
        let body: Value = serde_json::from_str(posts[0].body()).expect("request body is JSON");
        let calls = &body["params"][0];
        assert_eq!(calls[1]["methodName"], "aria2.tellStatus");
        assert_eq!(calls[1]["params"][0], "selected-gid");
        assert_eq!(calls[1]["params"][1][15], "files");
    }

    #[test]
    fn download_snapshot_returns_error_when_any_multicall_entry_fails() {
        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":11,"result":[[{"downloadSpeed":"0","uploadSpeed":"0","numActive":"0","numWaiting":"0","numStopped":"0"}],{"faultCode":1,"faultString":"bad nested call"},[[]],[[]]]}"#,
        )));

        assert!(matches!(
            fetch_download_snapshot_with_transport(&settings, &transport),
            Err(ClientError::Rpc { code: 1, .. })
        ));
    }

    #[test]
    fn add_uri_posts_json_rpc_add_uri_and_returns_gid() {
        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":6,"result":"new-gid"}"#,
        )));

        let gid = add_uri_with_transport(
            &settings,
            &transport,
            "https://example.test/file",
            AddUriOptions::default(),
        )
        .expect("addUri should return gid");

        assert_eq!(gid.as_str(), "new-gid");

        let posts = transport.posts.borrow();
        let body: Value = serde_json::from_str(posts[0].body()).expect("request body is JSON");
        assert_eq!(body["method"], "aria2.addUri");
        assert_eq!(body["id"], 6);
        assert_eq!(body["params"][0][0], "https://example.test/file");
        assert_eq!(body["params"].as_array().expect("params").len(), 1);
    }

    #[test]
    fn add_uri_posts_modeled_options_when_present() {
        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":6,"result":"new-gid"}"#,
        )));

        add_uri_with_transport(
            &settings,
            &transport,
            "https://example.test/file",
            AddUriOptions::new(
                Some("/downloads".to_owned()),
                Some("file.iso".to_owned()),
                Some("1024".to_owned()),
                Some("2048".to_owned()),
            ),
        )
        .expect("addUri should return gid");

        let posts = transport.posts.borrow();
        let body: Value = serde_json::from_str(posts[0].body()).expect("request body is JSON");
        assert_eq!(body["method"], "aria2.addUri");
        assert_eq!(body["params"][0][0], "https://example.test/file");
        assert_eq!(body["params"][1]["dir"], "/downloads");
        assert_eq!(body["params"][1]["out"], "file.iso");
        assert_eq!(body["params"][1]["max-download-limit"], "1024");
        assert_eq!(body["params"][1]["max-upload-limit"], "2048");
    }

    #[test]
    fn runtime_global_options_fetch_and_change_are_modeled() {
        let settings = Settings::default();
        let fetch_transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":12,"result":{"dir":"/downloads","max-concurrent-downloads":"6","max-overall-download-limit":"4096","max-overall-upload-limit":"512","save-session":"/tmp/session"}}"#,
        )));

        let options = get_runtime_global_options_with_transport(&settings, &fetch_transport)
            .expect("global options should parse");

        assert_eq!(options.directory(), Some("/downloads"));
        assert_eq!(options.max_concurrent_downloads(), Some("6"));
        let fetch_body: Value =
            serde_json::from_str(fetch_transport.posts.borrow()[0].body()).expect("json");
        assert_eq!(fetch_body["method"], "aria2.getGlobalOption");

        let change_transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":13,"result":"OK"}"#,
        )));
        change_runtime_global_options_with_transport(
            &settings,
            &change_transport,
            RuntimeGlobalOptions::with_values(
                Some("/new-downloads".to_owned()),
                Some("8".to_owned()),
                Some("8192".to_owned()),
                Some("1024".to_owned()),
            ),
        )
        .expect("global options should change");

        let change_body: Value =
            serde_json::from_str(change_transport.posts.borrow()[0].body()).expect("json");
        assert_eq!(change_body["method"], "aria2.changeGlobalOption");
        assert_eq!(change_body["params"][0]["dir"], "/new-downloads");
        assert_eq!(change_body["params"][0]["max-concurrent-downloads"], "8");
        assert_eq!(
            change_body["params"][0]["max-overall-download-limit"],
            "8192"
        );
        assert_eq!(change_body["params"][0]["max-overall-upload-limit"], "1024");
    }

    #[test]
    fn add_uri_maps_rpc_and_malformed_responses() {
        let settings = Settings::default();
        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":6,"error":{"code":1,"message":"bad uri"}}"#,
        )));

        assert!(matches!(
            add_uri_with_transport(
                &settings,
                &transport,
                "https://example.test/file",
                AddUriOptions::default(),
            ),
            Err(ClientError::Rpc { code: 1, .. })
        ));

        let transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":6,"result":""}"#,
        )));

        assert!(matches!(
            add_uri_with_transport(
                &settings,
                &transport,
                "https://example.test/file",
                AddUriOptions::default(),
            ),
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
    fn shutdown_commands_post_save_session_and_shutdown() {
        let mut draft = SettingsDraft::from_settings(&Settings::default());
        draft.set_secret("session-secret");
        let settings = draft.apply().expect("valid settings");

        let save_transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":15,"result":"OK"}"#,
        )));
        save_session_with_transport(&settings, &save_transport).expect("save session succeeds");

        let shutdown_transport = FakeTransport::returning(Ok(HttpResponse::ok(
            r#"{"jsonrpc":"2.0","id":16,"result":"OK"}"#,
        )));
        shutdown_with_transport(&settings, &shutdown_transport).expect("shutdown succeeds");

        let save_body: Value =
            serde_json::from_str(save_transport.posts.borrow()[0].body()).expect("json");
        let shutdown_body: Value =
            serde_json::from_str(shutdown_transport.posts.borrow()[0].body()).expect("json");

        assert_eq!(save_body["method"], "aria2.saveSession");
        assert_eq!(save_body["id"], 15);
        assert_eq!(save_body["params"][0], "token:session-secret");
        assert_eq!(shutdown_body["method"], "aria2.shutdown");
        assert_eq!(shutdown_body["id"], 16);
        assert_eq!(shutdown_body["params"][0], "token:session-secret");
    }

    #[test]
    fn connection_test_inserts_secret_token_without_debug_leak() {
        let mut draft = SettingsDraft::from_settings(&Settings::default());
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
