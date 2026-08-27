//! JSON-RPC 2.0 envelope types.
//!
//! Only the subset plugins need: requests with numeric ids, notifications,
//! responses, and the standard error codes. Serialization is derived, so the
//! wire shape is exactly the JSON-RPC examples (snake_case keys via
//! `#[serde(rename_all)]` where needed).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standard JSON-RPC error codes plus the RESQ plugin range.
pub mod codes {
    pub const PARSE_ERROR: i64 = -32700;
    pub const METHOD_NOT_FOUND: i64 = -32601;
    pub const INVALID_PARAMS: i64 = -32602;
    pub const INTERNAL_ERROR: i64 = -32603;
    /// Server-defined errors start here; plugins report domain failures
    /// (e.g. "no QVM loaded") as code -32000 with a human message.
    pub const PLUGIN_ERROR: i64 = -32000;
}

/// A JSON-RPC error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcError {
    pub fn new(code: i64, message: impl Into<String>) -> RpcError {
        RpcError {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn method_not_found(method: &str) -> RpcError {
        RpcError::new(
            codes::METHOD_NOT_FOUND,
            format!("method not found: {method}"),
        )
    }

    pub fn invalid_params(message: impl Into<String>) -> RpcError {
        RpcError::new(codes::INVALID_PARAMS, message)
    }

    pub fn plugin(message: impl Into<String>) -> RpcError {
        RpcError::new(codes::PLUGIN_ERROR, message)
    }
}

/// JSON-RPC id: the spec allows numbers and strings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Id {
    Num(u64),
    Str(String),
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Id::Num(n) => write!(f, "{n}"),
            Id::Str(s) => write!(f, "{s}"),
        }
    }
}

/// One inbound or outbound JSON-RPC message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    /// A call expecting a response.
    Request {
        id: Id,
        method: String,
        #[serde(default)]
        params: Value,
    },
    /// A one-way event; never answered.
    Notification {
        method: String,
        #[serde(default)]
        params: Value,
    },
    /// A reply to a request.
    Response {
        id: Id,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<RpcError>,
    },
}

impl Message {
    /// Convenience constructor for an `Ok` response.
    pub fn ok(id: Id, result: Value) -> Message {
        Message::Response {
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Convenience constructor for an error response.
    pub fn err(id: Id, error: RpcError) -> Message {
        Message::Response {
            id,
            result: None,
            error: Some(error),
        }
    }

    /// Serialize to one JSON line (no trailing newline).
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).expect("message serialization is infallible")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_request() {
        let m: Message =
            serde_json::from_str(r#"{"id":7,"method":"tools/call","params":{"name":"x"}}"#)
                .expect("parse");
        match m {
            Message::Request { id, method, params } => {
                assert_eq!(id, Id::Num(7));
                assert_eq!(method, "tools/call");
                assert_eq!(params["name"], "x");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn string_ids_are_accepted() {
        let m: Message = serde_json::from_str(r#"{"id":"abc-1","method":"ping"}"#).expect("parse");
        match m {
            Message::Request { id, method, .. } => {
                assert_eq!(id, Id::Str("abc-1".into()));
                assert_eq!(method, "ping");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn notification_has_no_id() {
        let m: Message =
            serde_json::from_str(r#"{"method":"notifications/initialized"}"#).expect("parse");
        assert!(
            matches!(m, Message::Notification { ref method, .. } if method == "notifications/initialized")
        );
    }

    #[test]
    fn error_response_omits_result() {
        let m = Message::err(Id::Num(3), RpcError::plugin("no qvm"));
        let line = m.to_line();
        assert!(!line.contains("\"result\""), "{line}");
        assert!(line.contains("\"code\":-32000"), "{line}");
    }
}
