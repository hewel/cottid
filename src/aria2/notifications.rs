use serde::Deserialize;
use serde_json::Value;

use crate::aria2::domain::Gid;
use crate::aria2::errors::ClientError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Aria2Notification {
    DownloadStart(Gid),
    DownloadPause(Gid),
    DownloadStop(Gid),
    DownloadComplete(Gid),
    DownloadError(Gid),
    BtDownloadComplete(Gid),
    Unknown { method: String, gid: Option<Gid> },
}

impl Aria2Notification {
    pub fn gid(&self) -> Option<&Gid> {
        match self {
            Self::DownloadStart(gid)
            | Self::DownloadPause(gid)
            | Self::DownloadStop(gid)
            | Self::DownloadComplete(gid)
            | Self::DownloadError(gid)
            | Self::BtDownloadComplete(gid) => Some(gid),
            Self::Unknown { gid, .. } => gid.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketFrame {
    Response,
    Notification(Aria2Notification),
    Request,
    Unknown,
}

#[derive(Debug, Deserialize)]
struct RawWebSocketFrame {
    id: Option<Value>,
    method: Option<String>,
    params: Option<Vec<Value>>,
}

#[derive(Debug, Deserialize)]
struct RawNotificationEvent {
    gid: String,
}

pub fn parse_websocket_frame(body: &str) -> Result<WebSocketFrame, ClientError> {
    let frame: RawWebSocketFrame = serde_json::from_str(body)
        .map_err(|error| ClientError::MalformedResponse(error.to_string()))?;

    match (frame.id.is_some(), frame.method) {
        (true, Some(_)) => Ok(WebSocketFrame::Request),
        (true, None) => Ok(WebSocketFrame::Response),
        (false, Some(method)) => Ok(WebSocketFrame::Notification(parse_notification(
            method,
            frame.params.as_deref(),
        )?)),
        (false, None) => Ok(WebSocketFrame::Unknown),
    }
}

fn parse_notification(
    method: String,
    params: Option<&[Value]>,
) -> Result<Aria2Notification, ClientError> {
    let gid = params
        .and_then(|params| params.first())
        .cloned()
        .map(parse_notification_gid)
        .transpose()?;

    match (method.as_str(), gid) {
        ("aria2.onDownloadStart", Some(gid)) => Ok(Aria2Notification::DownloadStart(gid)),
        ("aria2.onDownloadPause", Some(gid)) => Ok(Aria2Notification::DownloadPause(gid)),
        ("aria2.onDownloadStop", Some(gid)) => Ok(Aria2Notification::DownloadStop(gid)),
        ("aria2.onDownloadComplete", Some(gid)) => Ok(Aria2Notification::DownloadComplete(gid)),
        ("aria2.onDownloadError", Some(gid)) => Ok(Aria2Notification::DownloadError(gid)),
        ("aria2.onBtDownloadComplete", Some(gid)) => Ok(Aria2Notification::BtDownloadComplete(gid)),
        (_, gid) => Ok(Aria2Notification::Unknown { method, gid }),
    }
}

fn parse_notification_gid(value: Value) -> Result<Gid, ClientError> {
    let event = serde_json::from_value::<RawNotificationEvent>(value)
        .map_err(|error| ClientError::MalformedResponse(error.to_string()))?;

    Gid::new(event.gid).map_err(|error| ClientError::MalformedResponse(error.message().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::{Aria2Notification, WebSocketFrame, parse_websocket_frame};
    use crate::aria2::errors::ClientError;

    #[test]
    fn classifies_response_frames_by_id_without_method() {
        let frame = parse_websocket_frame(r#"{"jsonrpc":"2.0","id":7,"result":"OK"}"#)
            .expect("valid response frame");

        assert_eq!(frame, WebSocketFrame::Response);
    }

    #[test]
    fn classifies_request_frames_by_method_and_id() {
        let frame =
            parse_websocket_frame(r#"{"jsonrpc":"2.0","id":7,"method":"aria2.tellActive"}"#)
                .expect("valid request frame");

        assert_eq!(frame, WebSocketFrame::Request);
    }

    #[test]
    fn parses_download_start_notification_with_gid() {
        let frame = parse_websocket_frame(
            r#"{"jsonrpc":"2.0","method":"aria2.onDownloadStart","params":[{"gid":"abc123"}]}"#,
        )
        .expect("valid notification");

        assert_eq!(
            frame,
            WebSocketFrame::Notification(Aria2Notification::DownloadStart(
                crate::aria2::domain::Gid::new("abc123").expect("valid gid")
            ))
        );
    }

    #[test]
    fn keeps_bt_completion_distinct_from_normal_completion() {
        let frame = parse_websocket_frame(
            r#"{"jsonrpc":"2.0","method":"aria2.onBtDownloadComplete","params":[{"gid":"abc123"}]}"#,
        )
        .expect("valid notification");

        assert!(matches!(
            frame,
            WebSocketFrame::Notification(Aria2Notification::BtDownloadComplete(_))
        ));
    }

    #[test]
    fn preserves_unknown_notification_with_gid() {
        let frame = parse_websocket_frame(
            r#"{"jsonrpc":"2.0","method":"aria2.onFutureEvent","params":[{"gid":"abc123"}]}"#,
        )
        .expect("valid unknown notification");

        assert!(matches!(
            frame,
            WebSocketFrame::Notification(Aria2Notification::Unknown { gid: Some(_), .. })
        ));
    }

    #[test]
    fn rejects_notification_with_malformed_gid_payload() {
        let error = parse_websocket_frame(
            r#"{"jsonrpc":"2.0","method":"aria2.onDownloadStart","params":[{"gid":""}]}"#,
        )
        .expect_err("empty gid should fail");

        assert!(matches!(error, ClientError::MalformedResponse(_)));
    }
}
