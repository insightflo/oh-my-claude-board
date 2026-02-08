//! Hook event parser (serde_json)
//!
//! Parses JSONL (JSON Lines) hook event streams from Claude Code agents.
//! Handles: agent_start, agent_end, tool_start, tool_end, error events.
//! Gracefully skips malformed lines.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::path::Path;

/// Raw event as deserialized from JSON Lines
#[derive(Debug, Clone, Deserialize)]
pub struct HookEvent {
    pub event_type: EventType,
    pub timestamp: DateTime<Utc>,
    pub agent_id: String,
    pub task_id: String,
    pub session_id: String,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
}

/// Known event types from Claude Code hooks
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    AgentStart,
    AgentEnd,
    ToolStart,
    ToolEnd,
    Error,
}

/// Result of parsing a JSONL file: events + any parse errors
#[derive(Debug)]
pub struct ParseResult {
    pub events: Vec<HookEvent>,
    pub errors: Vec<ParseError>,
}

/// A single line parse error
#[derive(Debug)]
pub struct ParseError {
    pub line_number: usize,
    pub line_content: String,
    pub error: String,
}

/// Parse a JSONL string into hook events, collecting errors for malformed lines
pub fn parse_hook_events(input: &str) -> ParseResult {
    let mut events = Vec::new();
    let mut errors = Vec::new();

    for (idx, line) in input.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<HookEvent>(trimmed) {
            Ok(event) => events.push(event),
            Err(e) => errors.push(ParseError {
                line_number: idx + 1,
                line_content: trimmed.to_string(),
                error: e.to_string(),
            }),
        }
    }

    ParseResult { events, errors }
}

/// Parse a JSONL file from disk
pub fn parse_hook_file(path: &Path) -> Result<ParseResult, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    Ok(parse_hook_events(&content))
}

/// Filter events by agent ID
pub fn events_for_agent(events: &[HookEvent], agent_id: &str) -> Vec<HookEvent> {
    events
        .iter()
        .filter(|e| e.agent_id == agent_id)
        .cloned()
        .collect()
}

/// Filter events by session ID
pub fn events_for_session(events: &[HookEvent], session_id: &str) -> Vec<HookEvent> {
    events
        .iter()
        .filter(|e| e.session_id == session_id)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_agent_events() {
        let input = include_str!("../../tests/fixtures/sample_hooks/agent_events.jsonl");
        let result = parse_hook_events(input);
        assert_eq!(result.events.len(), 6);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn parse_agent_event_types() {
        let input = include_str!("../../tests/fixtures/sample_hooks/agent_events.jsonl");
        let result = parse_hook_events(input);
        assert_eq!(result.events[0].event_type, EventType::AgentStart);
        assert_eq!(result.events[1].event_type, EventType::ToolStart);
        assert_eq!(result.events[2].event_type, EventType::ToolEnd);
        assert_eq!(result.events[5].event_type, EventType::AgentEnd);
    }

    #[test]
    fn parse_agent_event_fields() {
        let input = include_str!("../../tests/fixtures/sample_hooks/agent_events.jsonl");
        let result = parse_hook_events(input);
        let first = &result.events[0];
        assert_eq!(first.agent_id, "backend-specialist-1");
        assert_eq!(first.task_id, "P1-R1-T1");
        assert_eq!(first.session_id, "sess-001");
        assert!(first.tool_name.is_none());
    }

    #[test]
    fn parse_tool_event_has_tool_name() {
        let input = include_str!("../../tests/fixtures/sample_hooks/agent_events.jsonl");
        let result = parse_hook_events(input);
        let tool_start = &result.events[1];
        assert_eq!(tool_start.tool_name.as_deref(), Some("Read"));
    }

    #[test]
    fn parse_error_events() {
        let input = include_str!("../../tests/fixtures/sample_hooks/error_events.jsonl");
        let result = parse_hook_events(input);
        assert_eq!(result.events.len(), 4);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn parse_error_event_message() {
        let input = include_str!("../../tests/fixtures/sample_hooks/error_events.jsonl");
        let result = parse_hook_events(input);
        let err_event = &result.events[1];
        assert_eq!(err_event.event_type, EventType::Error);
        assert_eq!(
            err_event.error_message.as_deref(),
            Some("permission denied: /etc/shadow")
        );
    }

    #[test]
    fn parse_malformed_gracefully() {
        let input = include_str!("../../tests/fixtures/sample_hooks/malformed.jsonl");
        let result = parse_hook_events(input);
        assert_eq!(result.events.len(), 3);
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn malformed_error_has_line_info() {
        let input = include_str!("../../tests/fixtures/sample_hooks/malformed.jsonl");
        let result = parse_hook_events(input);
        assert_eq!(result.errors[0].line_number, 2);
        assert!(result.errors[0].line_content.contains("not valid json"));
    }

    #[test]
    fn empty_input() {
        let result = parse_hook_events("");
        assert!(result.events.is_empty());
        assert!(result.errors.is_empty());
    }

    #[test]
    fn filter_by_agent() {
        let input = include_str!("../../tests/fixtures/sample_hooks/agent_events.jsonl");
        let result = parse_hook_events(input);
        let filtered = events_for_agent(&result.events, "backend-specialist-1");
        assert_eq!(filtered.len(), 6);
        let filtered_none = events_for_agent(&result.events, "nonexistent");
        assert!(filtered_none.is_empty());
    }

    #[test]
    fn filter_by_session() {
        let input = include_str!("../../tests/fixtures/sample_hooks/agent_events.jsonl");
        let result = parse_hook_events(input);
        let filtered = events_for_session(&result.events, "sess-001");
        assert_eq!(filtered.len(), 6);
        let filtered_none = events_for_session(&result.events, "sess-999");
        assert!(filtered_none.is_empty());
    }

    #[test]
    fn timestamps_are_ordered() {
        let input = include_str!("../../tests/fixtures/sample_hooks/agent_events.jsonl");
        let result = parse_hook_events(input);
        for window in result.events.windows(2) {
            assert!(window[0].timestamp <= window[1].timestamp);
        }
    }

    #[test]
    fn parse_file_from_disk() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/sample_hooks/agent_events.jsonl");
        let result = parse_hook_file(&path).expect("should read file");
        assert_eq!(result.events.len(), 6);
    }

    #[test]
    fn parse_file_nonexistent() {
        let result = parse_hook_file(Path::new("/nonexistent/path.jsonl"));
        assert!(result.is_err());
    }
}
