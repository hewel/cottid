use std::fmt;

use serde::{Deserialize, Serialize};

use crate::aria2::domain::Gid;
use crate::config::Secret;

const DOWNLOAD_ITEM_KEYS: [&str; 7] = [
    "gid",
    "status",
    "totalLength",
    "completedLength",
    "downloadSpeed",
    "uploadSpeed",
    "files",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct RequestId(u64);

impl RequestId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize)]
pub struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: RequestId,
    method: &'static str,
    params: Vec<JsonRpcParam>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum JsonRpcParam {
    String(String),
    Number(u64),
    StringList(Vec<String>),
}

impl JsonRpcRequest {
    #[cfg(test)]
    pub fn id(&self) -> RequestId {
        self.id
    }

    #[cfg(test)]
    pub fn method(&self) -> &'static str {
        self.method
    }

    #[cfg(test)]
    pub fn params(&self) -> &[JsonRpcParam] {
        &self.params
    }
}

impl fmt::Debug for JsonRpcRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let params = if self.params.is_empty() {
            Vec::new()
        } else {
            vec!["<redacted>"]
        };

        f.debug_struct("JsonRpcRequest")
            .field("jsonrpc", &self.jsonrpc)
            .field("id", &self.id)
            .field("method", &self.method)
            .field("params", &params)
            .finish()
    }
}

pub fn build_get_version_request(id: RequestId, secret: Option<&Secret>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0",
        id,
        method: "aria2.getVersion",
        params: token_params(secret),
    }
}

pub fn build_get_global_stat_request(id: RequestId, secret: Option<&Secret>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0",
        id,
        method: "aria2.getGlobalStat",
        params: token_params(secret),
    }
}

pub fn build_tell_active_request(id: RequestId, secret: Option<&Secret>) -> JsonRpcRequest {
    let mut params = token_params(secret);
    params.push(download_item_keys_param());

    JsonRpcRequest {
        jsonrpc: "2.0",
        id,
        method: "aria2.tellActive",
        params,
    }
}

pub fn build_tell_waiting_request(id: RequestId, secret: Option<&Secret>) -> JsonRpcRequest {
    let mut params = token_params(secret);
    params.push(JsonRpcParam::Number(0));
    params.push(JsonRpcParam::Number(1000));
    params.push(download_item_keys_param());

    JsonRpcRequest {
        jsonrpc: "2.0",
        id,
        method: "aria2.tellWaiting",
        params,
    }
}

pub fn build_tell_stopped_request(id: RequestId, secret: Option<&Secret>) -> JsonRpcRequest {
    let mut params = token_params(secret);
    params.push(JsonRpcParam::Number(0));
    params.push(JsonRpcParam::Number(1000));
    params.push(download_item_keys_param());

    JsonRpcRequest {
        jsonrpc: "2.0",
        id,
        method: "aria2.tellStopped",
        params,
    }
}

pub fn build_add_uri_request(id: RequestId, secret: Option<&Secret>, uri: &str) -> JsonRpcRequest {
    let mut params = token_params(secret);
    params.push(JsonRpcParam::StringList(vec![uri.to_owned()]));

    JsonRpcRequest {
        jsonrpc: "2.0",
        id,
        method: "aria2.addUri",
        params,
    }
}

pub fn build_pause_request(id: RequestId, secret: Option<&Secret>, gid: &Gid) -> JsonRpcRequest {
    build_gid_command_request(id, secret, "aria2.pause", gid)
}

pub fn build_unpause_request(id: RequestId, secret: Option<&Secret>, gid: &Gid) -> JsonRpcRequest {
    build_gid_command_request(id, secret, "aria2.unpause", gid)
}

pub fn build_remove_request(id: RequestId, secret: Option<&Secret>, gid: &Gid) -> JsonRpcRequest {
    build_gid_command_request(id, secret, "aria2.remove", gid)
}

pub fn build_purge_stopped_request(id: RequestId, secret: Option<&Secret>) -> JsonRpcRequest {
    JsonRpcRequest {
        jsonrpc: "2.0",
        id,
        method: "aria2.purgeDownloadResult",
        params: token_params(secret),
    }
}

fn build_gid_command_request(
    id: RequestId,
    secret: Option<&Secret>,
    method: &'static str,
    gid: &Gid,
) -> JsonRpcRequest {
    let mut params = token_params(secret);
    params.push(JsonRpcParam::String(gid.as_str().to_owned()));

    JsonRpcRequest {
        jsonrpc: "2.0",
        id,
        method,
        params,
    }
}

fn token_params(secret: Option<&Secret>) -> Vec<JsonRpcParam> {
    secret
        .map(|secret| JsonRpcParam::String(format!("token:{}", secret.expose_for_session())))
        .into_iter()
        .collect()
}

fn download_item_keys_param() -> JsonRpcParam {
    JsonRpcParam::StringList(
        DOWNLOAD_ITEM_KEYS
            .iter()
            .map(|key| (*key).to_owned())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::download_item_keys_param;
    use serde_json::Value;

    use crate::aria2::domain::Gid;
    use crate::aria2::methods::{
        JsonRpcParam, RequestId, build_add_uri_request, build_get_global_stat_request,
        build_get_version_request, build_pause_request, build_purge_stopped_request,
        build_remove_request, build_tell_active_request, build_tell_stopped_request,
        build_tell_waiting_request, build_unpause_request,
    };
    use crate::config::Secret;

    #[test]
    fn builds_get_version_request_without_token() {
        let request = build_get_version_request(RequestId::new(7), None);

        assert_eq!(request.id(), RequestId::new(7));
        assert_eq!(request.method(), "aria2.getVersion");
        assert!(request.params().is_empty());
    }

    #[test]
    fn prepends_session_secret_token_for_get_version() {
        let secret = Secret::session("secret-value");
        let request = build_get_version_request(RequestId::new(8), Some(&secret));

        assert_eq!(
            request.params(),
            &[JsonRpcParam::String("token:secret-value".to_owned())]
        );
        assert!(!format!("{request:?}").contains("secret-value"));
    }

    #[test]
    fn builds_get_global_stat_request() {
        let request = build_get_global_stat_request(RequestId::new(11), None);

        assert_eq!(request.id(), RequestId::new(11));
        assert_eq!(request.method(), "aria2.getGlobalStat");
        assert!(request.params().is_empty());
    }

    #[test]
    fn builds_tell_active_request() {
        let request = build_tell_active_request(RequestId::new(21), None);

        assert_eq!(request.method(), "aria2.tellActive");
        assert_download_item_keys(&request.params()[0]);

        let body = serde_json::to_value(&request).expect("request serializes");
        assert_eq!(body["params"][0][0], "gid");
        assert_eq!(body["params"][0][6], "files");
    }

    #[test]
    fn builds_tell_waiting_request_with_offset_and_count() {
        let request = build_tell_waiting_request(RequestId::new(22), None);

        assert_eq!(request.method(), "aria2.tellWaiting");
        assert_eq!(
            request.params(),
            &[
                JsonRpcParam::Number(0),
                JsonRpcParam::Number(1000),
                download_item_keys_param()
            ]
        );

        let body = serde_json::to_value(&request).expect("request serializes");
        assert_eq!(body["params"][0], 0);
        assert_eq!(body["params"][1], 1000);
        assert_eq!(body["params"][2][0], "gid");
        assert_eq!(body["params"][2][6], "files");
    }

    #[test]
    fn builds_tell_stopped_request_with_secret_offset_and_count() {
        let secret = Secret::session("secret-value");
        let request = build_tell_stopped_request(RequestId::new(23), Some(&secret));

        assert_eq!(request.method(), "aria2.tellStopped");
        assert_eq!(
            request.params(),
            &[
                JsonRpcParam::String("token:secret-value".to_owned()),
                JsonRpcParam::Number(0),
                JsonRpcParam::Number(1000),
                download_item_keys_param()
            ]
        );
        assert!(!format!("{request:?}").contains("secret-value"));

        let body = serde_json::to_value(&request).expect("request serializes");
        assert_eq!(body["params"][0], "token:secret-value");
        assert_eq!(body["params"][1], 0);
        assert_eq!(body["params"][2], 1000);
        assert_eq!(body["params"][3][0], "gid");
        assert_eq!(body["params"][3][6], "files");
    }

    #[test]
    fn builds_add_uri_request_with_uri_array() {
        let request = build_add_uri_request(RequestId::new(31), None, "https://example.test/file");
        let body: Value = serde_json::to_value(&request).expect("request serializes");

        assert_eq!(request.method(), "aria2.addUri");
        assert_eq!(body["params"][0][0], "https://example.test/file");
    }

    #[test]
    fn builds_add_uri_request_with_secret_before_uri_array() {
        let secret = Secret::session("secret-value");
        let request = build_add_uri_request(RequestId::new(31), Some(&secret), "magnet:?xt=abc");
        let body: Value = serde_json::to_value(&request).expect("request serializes");

        assert_eq!(body["params"][0], "token:secret-value");
        assert_eq!(body["params"][1][0], "magnet:?xt=abc");
        assert!(!format!("{request:?}").contains("secret-value"));
    }

    #[test]
    fn builds_gid_command_requests() {
        let gid = Gid::new("abc123").expect("valid gid");
        let pause = build_pause_request(RequestId::new(41), None, &gid);
        let unpause = build_unpause_request(RequestId::new(42), None, &gid);
        let remove = build_remove_request(RequestId::new(43), None, &gid);

        assert_eq!(pause.method(), "aria2.pause");
        assert_eq!(unpause.method(), "aria2.unpause");
        assert_eq!(remove.method(), "aria2.remove");
        assert_eq!(pause.params(), &[JsonRpcParam::String("abc123".to_owned())]);
    }

    #[test]
    fn builds_purge_stopped_request() {
        let request = build_purge_stopped_request(RequestId::new(44), None);

        assert_eq!(request.method(), "aria2.purgeDownloadResult");
        assert!(request.params().is_empty());
    }

    fn assert_download_item_keys(param: &JsonRpcParam) {
        assert_eq!(
            param,
            &JsonRpcParam::StringList(vec![
                "gid".to_owned(),
                "status".to_owned(),
                "totalLength".to_owned(),
                "completedLength".to_owned(),
                "downloadSpeed".to_owned(),
                "uploadSpeed".to_owned(),
                "files".to_owned(),
            ])
        );
    }
}
