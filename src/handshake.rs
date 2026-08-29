//! Optional RESQ-level `initialize` exchange.
//!
//! MCP plugins do NOT use this — the Model Context Protocol has its own
//! `initialize` with capabilities negotiation, handled inside the plugin's
//! [`crate::service::Handler`]. Use this module for minimal non-MCP RESQ
//! plugins (custom GUI tools, batch scripts) where a tiny fixed contract is
//! enough:
//!
//! ```json
//! -> {"id":1,"method":"initialize","params":{"protocolVersion":1,"client":"resq-gui"}}
//! <- {"id":1,"result":{"protocolVersion":1,"plugin":{"name":"x","version":"0.1.0"}}}
//! ```
//!
//! [`accepts`] validates an incoming request; [`InitializeResult`] builds
//! the reply (`.from_info(&info)` fills the plugin identity,
//! `.with_protocol_version(v)` echoes the negotiated version).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Client -> plugin `initialize` params.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub protocol_version: u32,
    #[serde(default)]
    pub client: String,
}

/// Plugin -> client `initialize` result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: u32,
    pub plugin: PluginDescriptor,
}

/// Flat identity block inside [`InitializeResult`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDescriptor {
    pub name: String,
    pub version: String,
}

impl InitializeResult {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> InitializeResult {
        InitializeResult {
            protocol_version: crate::PROTOCOL_VERSION,
            plugin: PluginDescriptor {
                name: name.into(),
                version: version.into(),
            },
        }
    }

    /// Build from a handler's [`crate::service::PluginInfo`].
    pub fn from_info(info: &crate::service::PluginInfo) -> InitializeResult {
        InitializeResult::new(info.name.clone(), info.version.clone())
    }

    /// Override the reported `protocolVersion`, e.g. to echo the highest
    /// version the client requested that the plugin supports.
    pub fn with_protocol_version(mut self, protocol_version: u32) -> Self {
        self.protocol_version = protocol_version;
        self
    }

    /// Serialize as the `result` value of an `initialize` response.
    pub fn into_value(self) -> Value {
        serde_json::to_value(self).expect("handshake serialization is infallible")
    }
}

/// True when an incoming `initialize` request carries a `protocolVersion`
/// this SDK can speak (`<=` [`crate::PROTOCOL_VERSION`]).
pub fn accepts(params: &Value) -> bool {
    matches!(
        params.get("protocolVersion").and_then(Value::as_u64),
        Some(v) if v as u32 <= crate::PROTOCOL_VERSION
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_shape_is_stable() {
        let v = InitializeResult::new("echo", "0.2.0").into_value();
        assert_eq!(v["protocolVersion"], crate::PROTOCOL_VERSION);
        assert_eq!(v["plugin"]["name"], "echo");
        assert_eq!(v["plugin"]["version"], "0.2.0");
    }

    #[test]
    fn accepts_supported_versions_only() {
        assert!(accepts(&serde_json::json!({
            "protocolVersion": crate::PROTOCOL_VERSION
        })));
        assert!(!accepts(&serde_json::json!({
            "protocolVersion": crate::PROTOCOL_VERSION + 1
        })));
        assert!(!accepts(&serde_json::json!({})));
    }

    #[test]
    fn from_info_and_echo() {
        let info = crate::service::PluginInfo {
            name: "echo".into(),
            version: "0.1.0".into(),
        };
        let v = InitializeResult::from_info(&info)
            .with_protocol_version(1)
            .into_value();
        assert_eq!(v["protocolVersion"], 1);
        assert_eq!(v["plugin"]["name"], "echo");
        assert_eq!(v["plugin"]["version"], "0.1.0");
    }
}
