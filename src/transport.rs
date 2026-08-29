//! Newline-delimited JSON-RPC framing over any reader/writer.
//!
//! stdout is the protocol channel only: plugin logs must go to stderr
//! (`eprintln!`/`log`-to-stderr), otherwise they corrupt the stream.

use crate::rpc::Message;
use std::io::{BufRead, Write};

/// Transport failures (broken pipe, bad UTF-8, line too long).
#[derive(Debug)]
pub enum TransportError {
    Io(std::io::Error),
    /// A line was received but is not valid JSON for [`Message`].
    BadMessage {
        line: String,
        reason: String,
    },
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::Io(e) => write!(f, "transport io: {e}"),
            TransportError::BadMessage { line, reason } => {
                write!(f, "bad message {line:?}: {reason}")
            }
        }
    }
}

impl std::error::Error for TransportError {}

/// Newline-delimited JSON-RPC over a buffered reader + writer.
pub struct StdioTransport<R: BufRead, W: Write> {
    reader: R,
    writer: W,
}

impl<R: BufRead, W: Write> StdioTransport<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        StdioTransport { reader, writer }
    }

    /// Read one message. Returns `Ok(None)` on clean EOF.
    pub fn read_message(&mut self) -> Result<Option<Message>, TransportError> {
        let mut line = String::new();
        loop {
            line.clear();
            let n = self
                .reader
                .read_line(&mut line)
                .map_err(TransportError::Io)?;
            if n == 0 {
                return Ok(None);
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue; // tolerate stray blank lines
            }
            return match serde_json::from_str(trimmed) {
                Ok(m) => Ok(Some(m)),
                Err(e) => Err(TransportError::BadMessage {
                    line: trimmed.to_string(),
                    reason: e.to_string(),
                }),
            };
        }
    }

    /// Write one message followed by a newline and flush (callers depend on
    /// immediate delivery for interactive sessions).
    pub fn write_message(&mut self, msg: &Message) -> Result<(), TransportError> {
        writeln!(self.writer, "{}", msg.to_line()).map_err(TransportError::Io)?;
        self.writer.flush().map_err(TransportError::Io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rpc::{Id, RpcError};
    use std::io::BufWriter;

    #[test]
    fn roundtrip_over_memory_buffer() {
        let mut input = String::new();
        input.push_str("{\"id\":1,\"method\":\"ping\"}\n");
        input.push('\n'); // blank lines are skipped
        input.push_str("{\"method\":\"notifications/x\"}\n");
        let mut t = StdioTransport::new(input.as_bytes(), BufWriter::new(Vec::new()));

        let m1 = t.read_message().expect("read").expect("some");
        assert!(
            matches!(m1, Message::Request { id: Id::Num(1), ref method, .. } if method == "ping")
        );
        let m2 = t.read_message().expect("read").expect("some");
        assert!(matches!(m2, Message::Notification { .. }));
        assert!(t.read_message().expect("read").is_none(), "EOF");

        t.write_message(&Message::err(Id::Num(9), RpcError::plugin("boom")))
            .expect("write");
    }

    #[test]
    fn bad_json_is_reported_not_swallowed() {
        let mut t = StdioTransport::new("not json\n".as_bytes(), BufWriter::new(Vec::new()));
        match t.read_message() {
            Err(TransportError::BadMessage { line, .. }) => assert_eq!(line, "not json"),
            other => panic!("expected BadMessage, got {other:?}"),
        }
    }
}
