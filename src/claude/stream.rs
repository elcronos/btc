use serde::{Deserialize, Serialize};

use crate::error::{BtcError, BtcResult};

/// A single event parsed from Claude Code's NDJSON stream output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEvent {
    AssistantMessage {
        text: String,
    },
    ToolUse {
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        output: String,
    },
    HookEvent {
        hook_type: String,
        data: serde_json::Value,
    },
    SystemMessage {
        text: String,
    },
    #[serde(rename = "result")]
    ResultMessage {
        text: String,
        #[serde(default)]
        cost_usd: f64,
    },
    Error {
        message: String,
    },
}

impl StreamEvent {
    /// Extract cost from a result event, if present.
    pub fn cost(&self) -> Option<f64> {
        match self {
            Self::ResultMessage { cost_usd, .. } if *cost_usd > 0.0 => Some(*cost_usd),
            _ => None,
        }
    }

    /// Return the text content for display, if this event carries text.
    pub fn text(&self) -> Option<&str> {
        match self {
            Self::AssistantMessage { text }
            | Self::SystemMessage { text }
            | Self::ResultMessage { text, .. }
            | Self::Error { message: text } => Some(text),
            _ => None,
        }
    }
}

/// Buffered parser that accumulates partial lines and emits complete
/// [`StreamEvent`]s from Claude Code's NDJSON output.
pub struct StreamParser {
    buffer: String,
}

impl StreamParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    /// Feed raw bytes into the parser and return any complete events.
    pub fn feed(&mut self, data: &str) -> Vec<StreamEvent> {
        self.buffer.push_str(data);
        let mut events = Vec::new();

        while let Some(newline_pos) = self.buffer.find('\n') {
            let line: String = self.buffer.drain(..=newline_pos).collect();
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            match parse_line(line) {
                Ok(event) => events.push(event),
                Err(e) => {
                    tracing::warn!("Skipping malformed NDJSON line: {e}");
                }
            }
        }

        events
    }
}

impl Default for StreamParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a single NDJSON line into a [`StreamEvent`].
pub fn parse_line(line: &str) -> BtcResult<StreamEvent> {
    serde_json::from_str(line).map_err(|e| {
        BtcError::StreamParse(format!("failed to parse stream event: {e} — line: {line}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_assistant_message() {
        let line = r#"{"type":"assistant_message","text":"Hello"}"#;
        let event = parse_line(line).unwrap();
        assert!(matches!(event, StreamEvent::AssistantMessage { text } if text == "Hello"));
    }

    #[test]
    fn parse_result_with_cost() {
        let line = r#"{"type":"result","text":"Done","cost_usd":0.05}"#;
        let event = parse_line(line).unwrap();
        assert_eq!(event.cost(), Some(0.05));
    }

    #[test]
    fn malformed_json_returns_error() {
        let line = "not json at all";
        assert!(parse_line(line).is_err());
    }

    #[test]
    fn stream_parser_buffers_partial_lines() {
        let mut parser = StreamParser::new();
        let events = parser.feed(r#"{"type":"assistant_message","te"#);
        assert!(events.is_empty());

        let events = parser.feed(r#"xt":"Hello"}
"#);
        assert_eq!(events.len(), 1);
    }
}
