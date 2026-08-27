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

    /// Serialize as the `result` value of an `initialize` response.
    pub fn into_value(self) -> Value {
        serde_json::to_value(self).expect("handshake serialization is infallible")
    }
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
}
