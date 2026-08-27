# resq-plugin-sdk

SDK for [RESQ-kit](https://github.com/unite-q3-team/RESQ-kit) plugins.

RESQ-kit plugins are **out-of-process programs** that speak newline-delimited
JSON-RPC 2.0 over stdio — the same transport model as LSP and MCP. Rust has
no stable plugin ABI, so loading `.dll`/`.so` plugins into a Rust GUI is
fragile; a protocol process boundary gives crash isolation, versioned
evolution, and lets plugins be written in any language.

First plugin built on this SDK: [resq-mcp](https://github.com/unite-q3-team/resq-mcp)
— an MCP server that exposes QVM analysis tools to AI agents.

## What's in the box

| Module        | Purpose                                                        |
|---------------|----------------------------------------------------------------|
| `manifest`    | `resq-plugin.toml` plugin descriptor                            |
| `rpc`         | JSON-RPC 2.0 envelope types (`Message`, `RpcError`, codes)      |
| `transport`   | newline-delimited framing over any reader/writer                |
| `service`     | the server loop (`serve`, `serve_stdio`) + `Handler` trait      |
| `handshake`   | optional RESQ-level `initialize` for non-MCP plugins            |

```toml
[dependencies]
resq-plugin-sdk = { path = "../resq-plugin-sdk" }
```

## Quick start

```rust,ignore
use resq_plugin_sdk::{serve_stdio, Handler, PluginInfo, RpcError};
use serde_json::Value;

struct MyPlugin;

impl Handler for MyPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo { name: "my-plugin".into(), version: "0.1.0".into() }
    }
    fn call(&mut self, method: &str, _params: &Value) -> Result<Value, RpcError> {
        Err(RpcError::method_not_found(method))
    }
}

fn main() {
    let mut p = MyPlugin;
    let _ = serve_stdio(&mut p); // reads stdin, writes stdout; logs -> stderr
}
```

See `examples/echo.rs` for a runnable minimal plugin.

## Rules of the transport

- **stdout is the protocol channel.** All diagnostics go to stderr.
- One JSON-RPC message per line, UTF-8, `\n`-terminated.
- Requests carry a numeric or string `id`; notifications never get replies.
- Unknown methods answer `-32601` (method not found); plugin-domain failures
  use `-32000` with a human-readable message.

## Docs

- [docs/architecture.md](docs/architecture.md) — why out-of-process, lifecycle, host embedding
- [docs/protocol.md](docs/protocol.md) — wire format, error codes, handshake
- [docs/writing-a-plugin.md](docs/writing-a-plugin.md) — step-by-step walkthrough

Русская версия: [docs-ru/](docs-ru/) и [README-ru.md](README-ru.md).

## License

MIT — see [LICENSE](LICENSE).
