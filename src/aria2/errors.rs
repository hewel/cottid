use std::fmt;

use crate::aria2::methods::RequestId;

#[derive(Clone, PartialEq, Eq)]
pub enum ClientError {
    Transport(String),
    HttpStatus(u16),
    MalformedResponse(String),
    ResponseIdMismatch {
        expected: RequestId,
        actual: RequestId,
    },
    Rpc {
        code: i64,
        message: String,
    },
}

impl ClientError {
    pub fn display_message(&self) -> &'static str {
        match self {
            Self::Transport(_) => "Connection failed. Check the endpoint and secret.",
            Self::HttpStatus(_) => "Connection failed. Check the endpoint and secret.",
            Self::MalformedResponse(_) => "aria2 returned a malformed response.",
            Self::ResponseIdMismatch { .. } => "aria2 returned an unexpected response.",
            Self::Rpc { .. } => "aria2 returned an RPC error.",
        }
    }
}

impl fmt::Debug for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(_) => f.write_str("Transport(<redacted>)"),
            Self::HttpStatus(status) => f.debug_tuple("HttpStatus").field(status).finish(),
            Self::MalformedResponse(_) => f.write_str("MalformedResponse(<redacted>)"),
            Self::ResponseIdMismatch { expected, actual } => f
                .debug_struct("ResponseIdMismatch")
                .field("expected", expected)
                .field("actual", actual)
                .finish(),
            Self::Rpc { code, message: _ } => f
                .debug_struct("Rpc")
                .field("code", code)
                .field("message", &"<redacted>")
                .finish(),
        }
    }
}
