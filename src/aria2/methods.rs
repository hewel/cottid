use std::fmt;

use serde::{Deserialize, Serialize};

use crate::config::Secret;

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
    params: Vec<String>,
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
    pub fn params(&self) -> &[String] {
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

fn token_params(secret: Option<&Secret>) -> Vec<String> {
    secret
        .map(|secret| format!("token:{}", secret.expose_for_session()))
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::aria2::methods::{
        RequestId, build_get_global_stat_request, build_get_version_request,
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

        assert_eq!(request.params(), &["token:secret-value"]);
        assert!(!format!("{request:?}").contains("secret-value"));
    }

    #[test]
    fn builds_get_global_stat_request() {
        let request = build_get_global_stat_request(RequestId::new(11), None);

        assert_eq!(request.id(), RequestId::new(11));
        assert_eq!(request.method(), "aria2.getGlobalStat");
        assert!(request.params().is_empty());
    }
}
