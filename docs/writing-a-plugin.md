# Writing a plugin

A plugin is: one executable + one `resq-plugin.toml` + a `Handler`.

## 1. Scaffold

```bash
cargo new --bin resq-hello
cd resq-hello
cargo add serde serde_json
cargo add --path ../resq-plugin-sdk resq-plugin-sdk
```

## 2. Manifest

`resq-plugin.toml` next to the built executable (hosts scan for it; for a
plain binary put a copy in the crate root and copy it in your packaging
step):

```toml
name = "resq-hello"
version = "0.1.0"
description = "Greets QVM addresses"
protocol = 1
```

## 3. Handler

`src/main.rs`:

```rust,ignore
use resq_plugin_sdk::{serve_stdio, Handler, PluginInfo, RpcError};
use serde_json::{json, Value};

struct Hello;

impl Handler for Hello {
    fn info(&self) -> PluginInfo {
        PluginInfo { name: "resq-hello".into(), version: "0.1.0".into() }
    }

    fn call(&mut self, method: &str, params: &Value) -> Result<Value, RpcError> {
        match method {
            "hello" => {
                let who = params.get("who").and_then(Value::as_str).unwrap_or("world");
                Ok(json!({ "greeting": format!("hello, {who}") }))
            }
            other => Err(RpcError::method_not_found(other)),
        }
    }
}

fn main() {
    let mut p = Hello;
    if let Err(e) = serve_stdio(&mut p) {
        eprintln!("[resq-hello] exited: {e}");
        std::process::exit(1);
    }
}
```

## 4. Try it by hand

```text
$ cargo run
{"id":1,"method":"hello","params":{"who":"qvm"}}
{"id":1,"result":{"greeting":"hello, qvm"}}
```

Type the request line, press Enter, read the reply line. Ctrl-D (EOF)
exits the loop.

## 5. Test it in-process

The loop is generic over reader/writer, so tests need no processes:

```rust,ignore
let input = "{\"id\":1,\"method\":\"hello\"}\n";
let mut out = Vec::new();
resq_plugin_sdk::serve(&mut Hello, input.as_bytes(), &mut out).unwrap();
assert!(String::from_utf8(out).unwrap().contains("hello, world"));
```

## Conventions

- Method names: `snake_case` (RESQ plugins) or the MCP surface
  (`initialize`, `tools/list`, `tools/call`, `ping`) for MCP servers.
- Params/results are plain JSON objects; prefer explicit fields over
  positional arrays so the schema can evolve additively.
- Heavy output (whole-file listings) belongs behind filters/offset/limit
  pagination — tools that answer agents should not dump megabytes.
- Never `println!` outside a response: stdout is the protocol channel.

## Shipping

- Static binaries are the distribution unit (one exe, no runtime deps
  beyond the OS).
- Hosts look for `resq-plugin.toml` beside the executable; keep the
  manifest in the release archive.
- Document your methods (or MCP tools) in the repo `docs/` — the manifest
  `description` is one line, not the reference.
