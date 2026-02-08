//! Fixture validation tests - ensures test fixtures are valid and loadable

mod helpers;

use helpers::*;

#[test]
fn fixtures_dir_exists() {
    assert!(fixtures_dir().exists(), "fixtures directory should exist");
}

#[test]
fn sample_tasks_file_exists_and_valid() {
    assert!(sample_tasks_path().exists(), "sample_tasks.md should exist");
    let content = read_sample_tasks();
    assert!(!content.is_empty());
    assert!(content.contains("Phase 0"));
    assert!(content.contains("Phase 1"));
    assert!(content.contains("Phase 2"));
    // Verify different task statuses exist
    assert!(content.contains("[x]"), "should have completed tasks");
    assert!(content.contains("[ ]"), "should have pending tasks");
    assert!(
        content.contains("[InProgress]"),
        "should have in-progress tasks"
    );
    assert!(content.contains("[Failed]"), "should have failed tasks");
    assert!(content.contains("[Blocked]"), "should have blocked tasks");
}

#[test]
fn sample_hooks_dir_exists() {
    assert!(
        sample_hooks_dir().exists(),
        "sample_hooks directory should exist"
    );
}

#[test]
fn agent_events_fixture_valid() {
    let content = read_hook_events("agent_events.jsonl");
    assert!(content.contains("agent_start"));
    assert!(content.contains("agent_end"));
    assert!(content.contains("tool_start"));
    assert!(content.contains("tool_end"));
    // Each line should be parseable as JSON (except empty lines)
    for line in content.lines() {
        if !line.trim().is_empty() {
            assert!(
                serde_json::from_str::<serde_json::Value>(line).is_ok(),
                "Line should be valid JSON: {line}"
            );
        }
    }
}

#[test]
fn error_events_fixture_valid() {
    let content = read_hook_events("error_events.jsonl");
    assert!(content.contains("error"));
    assert!(content.contains("permission denied"));
    assert!(content.contains("connection refused"));
}

#[test]
fn malformed_fixture_has_invalid_lines() {
    let content = read_hook_events("malformed.jsonl");
    let mut valid_count = 0;
    let mut invalid_count = 0;
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if serde_json::from_str::<serde_json::Value>(line).is_ok() {
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
    }
    assert!(valid_count >= 2, "should have at least 2 valid JSON lines");
    assert!(
        invalid_count >= 1,
        "should have at least 1 invalid JSON line"
    );
}
