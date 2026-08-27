//! Minimal example plugin: echoes text back via the `echo` method.
//!
//! Run it and type a request:
//!
//! ```text
//! cargo run --example echo
//! {"id":1,"method":"echo","params":{"text":"hi"}}
//! ```

use resq_plugin_sdk::{serve_stdio, Handler, PluginInfo};
use serde_json::Value;

struct Echo;

impl Handler for Echo {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "echo".into(),
            version: env!("CARGO_PKG_VERSION").into(),
        }
    }

    fn call(&mut self, method: &str, params: &Value) -> Result<Value, resq_plugin_sdk::RpcError> {
        match method {
            "echo" => {
                let text = params
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                Ok(Value::String(text.to_string()))
            }
            other => Err(resq_plugin_sdk::RpcError::method_not_found(other)),
        }
    }
}

fn main() {
    let mut h = Echo;
    if let Err(e) = serve_stdio(&mut h) {
        eprintln!("[resq] echo exited: {e}");
        std::process::exit(1);
    }
}
