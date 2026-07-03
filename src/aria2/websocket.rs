use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::to_string;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WebSocketMessage;

use crate::aria2::errors::ClientError;
use crate::aria2::methods::JsonRpcRequest;
use crate::aria2::notifications::{Aria2Notification, WebSocketFrame, parse_websocket_frame};
use crate::config::Settings;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WebSocketEvent {
    Connected,
    Degraded,
    Reconnecting,
    Notification(Aria2Notification),
}

pub fn websocket_endpoint(endpoint: &str) -> Result<String, ClientError> {
    let trimmed = endpoint.trim();
    if let Some(rest) = trimmed.strip_prefix("http://") {
        Ok(format!("ws://{rest}"))
    } else if let Some(rest) = trimmed.strip_prefix("https://") {
        Ok(format!("wss://{rest}"))
    } else {
        Err(ClientError::Transport(
            "endpoint does not support websocket derivation".to_owned(),
        ))
    }
}

pub async fn send_rpc_request(
    settings: &Settings,
    request: JsonRpcRequest,
) -> Result<String, ClientError> {
    let endpoint = websocket_endpoint(settings.endpoint())?;
    let body =
        to_string(&request).map_err(|error| ClientError::MalformedResponse(error.to_string()))?;

    timeout(REQUEST_TIMEOUT, send_rpc_request_inner(&endpoint, body))
        .await
        .map_err(|_| ClientError::Transport("websocket request timed out".to_owned()))?
}

async fn send_rpc_request_inner(endpoint: &str, body: String) -> Result<String, ClientError> {
    let (mut socket, _) = connect_async(endpoint)
        .await
        .map_err(|error| ClientError::Transport(error.to_string()))?;

    socket
        .send(WebSocketMessage::Text(body.into()))
        .await
        .map_err(|error| ClientError::Transport(error.to_string()))?;

    while let Some(message) = socket.next().await {
        let message = message.map_err(|error| ClientError::Transport(error.to_string()))?;
        let Some(text) = text_message(message)? else {
            continue;
        };

        match parse_websocket_frame(&text)? {
            WebSocketFrame::Response => return Ok(text),
            WebSocketFrame::Notification(_) | WebSocketFrame::Request | WebSocketFrame::Unknown => {
            }
        }
    }

    Err(ClientError::Transport("websocket closed".to_owned()))
}

pub async fn listen_notifications<F, Fut>(endpoint: String, mut emit: F)
where
    F: FnMut(WebSocketEvent) -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    loop {
        let websocket_endpoint = match websocket_endpoint(&endpoint) {
            Ok(endpoint) => endpoint,
            Err(_) => {
                if !emit(WebSocketEvent::Degraded).await {
                    return;
                }
                return;
            }
        };

        match connect_async(&websocket_endpoint).await {
            Ok((mut socket, _)) => {
                if !emit(WebSocketEvent::Connected).await {
                    return;
                }

                while let Some(message) = socket.next().await {
                    let message = match message {
                        Ok(message) => message,
                        Err(_) => break,
                    };
                    let text = match text_message(message) {
                        Ok(Some(text)) => text,
                        Ok(None) => continue,
                        Err(_) => break,
                    };

                    if let Ok(WebSocketFrame::Notification(notification)) =
                        parse_websocket_frame(&text)
                        && !emit(WebSocketEvent::Notification(notification)).await
                    {
                        return;
                    }
                }
            }
            Err(_) => {
                if !emit(WebSocketEvent::Degraded).await {
                    return;
                }
            }
        }

        if !emit(WebSocketEvent::Reconnecting).await {
            return;
        }
        sleep(RECONNECT_DELAY).await;
    }
}

fn text_message(message: WebSocketMessage) -> Result<Option<String>, ClientError> {
    match message {
        WebSocketMessage::Text(text) => Ok(Some(text.to_string())),
        WebSocketMessage::Binary(bytes) => String::from_utf8(bytes.to_vec())
            .map(Some)
            .map_err(|error| ClientError::MalformedResponse(error.to_string())),
        WebSocketMessage::Ping(_) | WebSocketMessage::Pong(_) | WebSocketMessage::Frame(_) => {
            Ok(None)
        }
        WebSocketMessage::Close(_) => Err(ClientError::Transport("websocket closed".to_owned())),
    }
}

#[cfg(test)]
mod tests {
    use super::websocket_endpoint;

    #[test]
    fn websocket_endpoint_derives_ws_from_http_endpoint() {
        let endpoint =
            websocket_endpoint("http://aria2.local:6800/jsonrpc").expect("valid endpoint");

        assert_eq!(endpoint, "ws://aria2.local:6800/jsonrpc");
    }

    #[test]
    fn websocket_endpoint_derives_wss_from_https_endpoint() {
        let endpoint =
            websocket_endpoint("https://aria2.local:6800/jsonrpc").expect("valid endpoint");

        assert_eq!(endpoint, "wss://aria2.local:6800/jsonrpc");
    }

    #[test]
    fn websocket_endpoint_rejects_non_http_endpoint() {
        let error =
            websocket_endpoint("ftp://aria2.local:6800/jsonrpc").expect_err("unsupported endpoint");

        assert!(matches!(
            error,
            crate::aria2::errors::ClientError::Transport(_)
        ));
    }
}
