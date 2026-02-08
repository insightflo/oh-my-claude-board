//! Test helper utilities

use std::path::{Path, PathBuf};

/// Returns the path to the test fixtures directory
pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

/// Returns the path to the sample TASKS.md fixture
pub fn sample_tasks_path() -> PathBuf {
    fixtures_dir().join("sample_tasks.md")
}

/// Returns the path to the sample hooks directory
pub fn sample_hooks_dir() -> PathBuf {
    fixtures_dir().join("sample_hooks")
}

/// Reads the sample TASKS.md fixture as a string
pub fn read_sample_tasks() -> String {
    std::fs::read_to_string(sample_tasks_path()).expect("Failed to read sample_tasks.md fixture")
}

/// Reads a hook event file as a string
pub fn read_hook_events(filename: &str) -> String {
    std::fs::read_to_string(sample_hooks_dir().join(filename))
        .expect("Failed to read hook event fixture")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixtures_dir_exists() {
        assert!(fixtures_dir().exists(), "fixtures directory should exist");
    }

    #[test]
    fn sample_tasks_file_exists() {
        assert!(sample_tasks_path().exists(), "sample_tasks.md should exist");
    }

    #[test]
    fn sample_hooks_dir_exists() {
        assert!(
            sample_hooks_dir().exists(),
            "sample_hooks directory should exist"
        );
    }

    #[test]
    fn read_sample_tasks_not_empty() {
        let content = read_sample_tasks();
        assert!(!content.is_empty(), "sample_tasks.md should not be empty");
        assert!(
            content.contains("Phase"),
            "sample_tasks.md should contain Phase"
        );
    }

    #[test]
    fn read_hook_events_agent() {
        let content = read_hook_events("agent_events.jsonl");
        assert!(
            !content.is_empty(),
            "agent_events.jsonl should not be empty"
        );
        assert!(
            content.contains("agent_start"),
            "should contain agent_start event"
        );
    }

    #[test]
    fn read_hook_events_error() {
        let content = read_hook_events("error_events.jsonl");
        assert!(
            content.contains("permission denied"),
            "should contain error message"
        );
    }

    #[test]
    fn read_hook_events_malformed() {
        let content = read_hook_events("malformed.jsonl");
        assert!(
            content.contains("not valid json"),
            "should contain malformed line"
        );
    }
}
