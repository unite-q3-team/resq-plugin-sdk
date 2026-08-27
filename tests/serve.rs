//! End-to-end test of the serve loop over in-memory transports.

use resq_plugin_sdk::{serve, Handler, PluginInfo, RpcError};
use serde_json::{json, Value};

struct Demo;

impl Handler for Demo {
    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: "demo".into(),
            version: "0.0.1".into(),
        }
    }

    fn call(&mut self, method: &str, params: &Value) -> Result<Value, RpcError> {
        match method {
            "add" => {
                let a = params.get("a").and_then(Value::as_i64).unwrap_or(0);
                let b = params.get("b").and_then(Value::as_i64).unwrap_or(0);
                Ok(json!({ "sum": a + b }))
            }
            "fail" => Err(RpcError::plugin("expected failure")),
            other => Err(RpcError::method_not_found(other)),
        }
    }

    fn notify(&mut self, method: &str, _params: &Value) {
        if method == "bye" {
            // Notifications are ignored by the loop; nothing to assert here.
        }
    }
}

#[test]
fn serve_loop_requests_notifications_and_errors() {
    let mut input = String::new();
    input.push_str("{\"id\":1,\"method\":\"add\",\"params\":{\"a\":2,\"b\":40}}\n");
    input.push_str("{\"method\":\"bye\"}\n");
    input.push_str("{\"id\":2,\"method\":\"fail\"}\n");
    input.push_str("{\"id\":3,\"method\":\"nope\"}\n");

    let mut out: Vec<u8> = Vec::new();
    let served = serve(&mut Demo, input.as_bytes(), &mut out).expect("serve");
    assert_eq!(served, 3);

    let text = String::from_utf8(out).expect("utf8");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 3);

    let r1: Value = serde_json::from_str(lines[0]).unwrap();
    assert_eq!(r1["id"], 1);
    assert_eq!(r1["result"]["sum"], 42);

    let r2: Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(r2["id"], 2);
    assert_eq!(r2["error"]["code"], -32000);

    let r3: Value = serde_json::from_str(lines[2]).unwrap();
    assert_eq!(r3["error"]["code"], -32601);
}
