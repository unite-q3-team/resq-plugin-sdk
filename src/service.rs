//! The plugin server loop.
//!
//! A plugin implements [`Handler`] and hands control to [`serve_stdio`].
//! The loop is deliberately minimal: it parses transport messages, forwards
//! `method`/`params` to the handler, and maps results back. There is no
//! built-in RESQ handshake — MCP plugins handle `initialize` inside their
//! handler; plain RESQ plugins can use the [`crate::handshake`] helpers the
//! same way.

use crate::rpc::{Message, RpcError};
use crate::transport::{StdioTransport, TransportError};
use serde::Serialize;
use serde_json::Value;
use std::io::{BufRead, Write};

/// Plugin identity reported to hosts and diagnostics.
#[derive(Debug, Clone, Serialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
}

/// The full method surface of a plugin.
pub trait Handler {
    /// Plugin identity (used in logs and the optional RESQ handshake).
    fn info(&self) -> PluginInfo;

    /// Handle one request; called only for requests, never notifications.
    /// Return [`RpcError::method_not_found`] for unknown methods.
    fn call(&mut self, method: &str, params: &Value) -> Result<Value, RpcError>;

    /// Handle a one-way notification. Default: ignore.
    fn notify(&mut self, _method: &str, _params: &Value) {}
}

/// Run the server loop over arbitrary reader/writer until EOF. Returns the
/// number of requests served (useful for tests).
pub fn serve<R: BufRead, W: Write>(
    handler: &mut dyn Handler,
    reader: R,
    writer: W,
) -> Result<usize, TransportError> {
    let mut t = StdioTransport::new(reader, writer);
    let mut served = 0usize;
    loop {
        match t.read_message()? {
            None => return Ok(served),
            Some(Message::Request { id, method, params }) => {
                let reply = match handler.call(&method, &params) {
                    Ok(v) => Message::ok(id, v),
                    Err(e) => Message::err(id, e),
                };
                t.write_message(&reply)?;
                served += 1;
            }
            Some(Message::Notification { method, params }) => {
                handler.notify(&method, &params);
            }
            Some(Message::Response { .. }) => {
                // Plugins are servers; stray responses are ignored.
            }
        }
    }
}

/// [`serve`] over the process stdio: requests on stdin, responses on stdout,
/// plugin logs on stderr.
pub fn serve_stdio(handler: &mut dyn Handler) -> Result<usize, TransportError> {
    let info = handler.info();
    eprintln!("[resq] {} v{} serving on stdio", info.name, info.version);
    serve(handler, std::io::stdin().lock(), std::io::stdout().lock())
}
