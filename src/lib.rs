//! RESQ plugin SDK — the contract between RESQ-kit and out-of-process
//! plugins.
//!
//! RESQ-kit plugins are separate programs that speak newline-delimited
//! JSON-RPC 2.0 over stdio (the same transport model as LSP and MCP).
//! Rust has no stable plugin ABI, so dynamic in-process loading is off the
//! table; a protocol process boundary keeps plugins crash-isolated and
//! language-agnostic (the wire format is trivial to implement outside Rust).
//!
//! Modules:
//! - [`manifest`]: `resq-plugin.toml` plugin descriptor.
//! - [`rpc`]: JSON-RPC 2.0 envelope types and error codes.
//! - [`transport`]: newline-delimited framing over any reader/writer.
//! - [`service`]: the server loop ([`service::serve`]) + [`service::Handler`].
//! - [`handshake`]: optional RESQ-level `initialize` exchange for plugins
//!   that are not MCP servers (MCP has its own initialize method).
//!
//! Everything is re-exported at the crate root.

pub mod handshake;
pub mod manifest;
pub mod rpc;
pub mod service;
pub mod transport;

pub use manifest::{Manifest, ManifestError};
pub use rpc::{Message, RpcError};
pub use service::{serve, serve_stdio, Handler, PluginInfo};
pub use transport::{StdioTransport, TransportError};

/// Wire protocol version implemented by this SDK. Clients put their version
/// into `initialize`; a plugin replies with the version it speaks. Bump on
/// breaking changes to the RESQ-level conventions (not to individual tool
/// schemas, which live inside plugin-specific methods).
pub const PROTOCOL_VERSION: u32 = 1;
