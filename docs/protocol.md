# Wire protocol

## Transport

Newline-delimited JSON-RPC 2.0 over the plugin's stdio:

- each message is one line, UTF-8, terminated by `\n`;
- blank lines are skipped;
- **stdout carries protocol messages only** — logs go to stderr;
- the writer flushes after every message (interactive sessions).

## Messages

Request (expects exactly one response):

```json
{"id": 7, "method": "tools/call", "params": {"name": "open_qvm"}}
```

Notification (no response ever):

```json
{"method": "notifications/initialized"}
```

Response:

```json
{"id": 7, "result": {"content": []}}
{"id": 7, "error": {"code": -32000, "message": "no QVM loaded - call open_qvm first"}}
```

`id` is a non-negative integer or a string (the SDK type `Id` accepts both;
untyped `null` ids from the JSON-RPC spec are not used).

## Error codes

| Code    | Meaning                                                      |
|---------|--------------------------------------------------------------|
| `-32700`| parse error (bad JSON line)                                   |
| `-32601`| method not found                                              |
| `-32602`| invalid params (schema violations, missing fields)            |
| `-32603`| internal error                                                |
| `-32000`| plugin/domain error — "file not found", "no session", etc.    |

Discipline: a request that reached the handler never dies as a protocol
error if the failure is the *task's* (bad path, empty filter results).
MCP servers additionally wrap task failures as `isError: true` tool
results — see the resq-mcp docs.

## RESQ handshake (optional, non-MCP plugins)

```text
-> {"id":1,"method":"initialize","params":{"protocolVersion":1,"client":"resq-gui"}}
<- {"id":1,"result":{"protocolVersion":1,"plugin":{"name":"x","version":"0.1.0"}}}
```

The plugin echoes the highest protocol version it supports that is `<=`
the requested one. `handshake::InitializeResult` builds the payload;
`service::Handshake::accepts` validates an incoming request.

## MCP handshake (MCP servers built on this SDK)

MCP servers do not use the RESQ handshake: the Model Context Protocol
defines its own `initialize` with capability negotiation, plus
`notifications/initialized`, `ping`, `tools/list`, `tools/call`. The SDK
provides transport and envelope types; the MCP method surface belongs to
the server (see `resq-mcp/src/main.rs` for a complete example).

## Framing notes

- There is intentionally no `Content-Length` headering (LSP style): line
  framing is trivially implementable in every language and debuggable with
  `cat` and a pipe.
- A malformed line is reported as a parse error (or `TransportError::
  BadMessage` for in-memory use) — the receiver may reply
  `{"id":null,...}` per JSON-RPC, but the SDK loop simply surfaces the
  error to the host process.
