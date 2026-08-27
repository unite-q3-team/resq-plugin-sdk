# Architecture

## Why out-of-process plugins

1. **No stable Rust ABI.** A GUI compiled with rustc A cannot load a `dylib`
   built with rustc B — layout and vtable assumptions differ. Every plugin
   would have to be rebuilt for every RESQ-kit build. A wire protocol has no
   such coupling.
2. **Crash isolation.** A misbehaving analysis pass (say, a decompiler
   experiment that panics on a weird CFG) must not take the GUI down. A
   child process dying is an error message, not data loss.
3. **Language freedom.** The wire format is one JSON object per line. A
   plugin can be Rust, Python, Node — anything that can read stdin and
   write stdout.
4. **Agents are first-class hosts.** The same protocol that a GUI speaks is
   the protocol MCP clients (Claude and friends) already speak. A plugin
   written once serves both.

## Topology

```text
+--------------------+      newline-delimited JSON-RPC 2.0      +-----------------+
| host               | <--------------------------------------> | plugin process  |
| (resq-gui, agent,  |            stdin / stdout                 | (resq-mcp, ...) |
|  human with a pipe)|            logs -> stderr                 |                 |
+--------------------+                                           +-----------------+
```

A host discovers plugins by scanning a plugin directory for
`resq-plugin.toml` descriptors (`manifest` module), spawns the executable
the descriptor sits next to, and drives the protocol. The SDK repo ships
the contract; embedding into resq-gui is a host-side concern (planned).

## Lifecycle

1. **spawn** — host starts the plugin executable.
2. **handshake** — either the MCP `initialize` (capability negotiation) or
   the minimal RESQ `initialize` (protocol version echo, see
   `handshake` module).
3. **session** — request/response traffic; notifications for one-way events.
4. **shutdown** — the host closes stdin; the plugin sees EOF on
   `serve_stdio`, flushes and exits. stdout is closed by the plugin before
   process exit; no extra "shutdown" method is required.

## Versioning

- `resq-plugin-sdk::PROTOCOL_VERSION` (currently `1`) covers RESQ-level
  conventions: framing, manifest, error-code discipline, handshake shape.
- Plugin-specific method schemas (e.g. MCP tools) are versioned by the
  plugin itself (semver in the manifest + the MCP `protocolVersion` for
  MCP servers).
- Additive changes (new methods, new optional fields) never bump
  `PROTOCOL_VERSION`.

## What the SDK does NOT do

- No plugin discovery/scanning — hosts implement it with `manifest::Manifest::load_dir`.
- No GUI embedding — see above.
- No persistence — sessions are owned by the plugin process; keep state in
  memory or sidecar files next to the analyzed artifacts.
