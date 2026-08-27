//! Plugin descriptor: `resq-plugin.toml` next to the plugin executable.
//!
//! ```toml
//! name = "resq-mcp"
//! version = "0.1.0"
//! description = "MCP server exposing QVM analysis tools to AI agents"
//! protocol = 1
//! ```

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Parsed `resq-plugin.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Unique plugin id, e.g. `resq-mcp`.
    pub name: String,
    /// Semver string, informational.
    pub version: String,
    /// One-line human description.
    #[serde(default)]
    pub description: String,
    /// Wire protocol version the plugin was built against
    /// ([`crate::PROTOCOL_VERSION`]).
    #[serde(default)]
    pub protocol: u32,
}

/// Manifest problems (unreadable file, bad TOML, missing fields).
#[derive(Debug)]
pub enum ManifestError {
    Io(std::io::Error),
    Parse(String),
    Missing(&'static str),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Io(e) => write!(f, "manifest io: {e}"),
            ManifestError::Parse(e) => write!(f, "manifest parse: {e}"),
            ManifestError::Missing(k) => write!(f, "manifest missing field: {k}"),
        }
    }
}

impl std::error::Error for ManifestError {}

impl Manifest {
    /// Parse from raw TOML text.
    pub fn parse(text: &str) -> Result<Manifest, ManifestError> {
        let m: Manifest = toml::from_str(text).map_err(|e| ManifestError::Parse(e.to_string()))?;
        if m.name.is_empty() {
            return Err(ManifestError::Missing("name"));
        }
        if m.version.is_empty() {
            return Err(ManifestError::Missing("version"));
        }
        Ok(m)
    }

    /// Load `resq-plugin.toml` from a directory.
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Manifest, ManifestError> {
        let p = dir.as_ref().join("resq-plugin.toml");
        let text = std::fs::read_to_string(&p).map_err(ManifestError::Io)?;
        Manifest::parse(&text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_manifest() {
        let m = Manifest::parse("name = \"resq-mcp\"\nversion = \"0.1.0\"\nprotocol = 1\n")
            .expect("parse");
        assert_eq!(m.name, "resq-mcp");
        assert_eq!(m.version, "0.1.0");
        assert_eq!(m.protocol, 1);
        assert_eq!(m.description, "");
    }

    #[test]
    fn rejects_missing_name() {
        assert!(Manifest::parse("version = \"1\"\n").is_err());
    }
}
