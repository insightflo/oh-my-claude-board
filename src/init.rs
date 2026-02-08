//! `simple-claude-board init` command implementation.
//!
//! Performs three setup steps:
//! 1. Creates `~/.claude/dashboard/` and `~/.claude/hooks/` directories
//! 2. Deploys the embedded `event-logger.js` to `~/.claude/hooks/`
//! 3. Patches `~/.claude/settings.json` with Pre/PostToolUse hook entries

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde_json::Value;

/// The standalone event-logger.js embedded at compile time.
const EVENT_LOGGER_JS: &str = include_str!("../hooks/event-logger.js");

/// The hook matcher pattern for tool events.
const HOOK_MATCHER: &str = "Task|Edit|Write|Read|Bash|Grep|Glob";

/// The hook command template.
const HOOK_COMMAND: &str = "node \"${HOME}/.claude/hooks/event-logger.js\"";

/// Hook timeout in seconds.
const HOOK_TIMEOUT: u64 = 3;

/// Run the init command: create dirs, deploy hook script, patch settings.
pub fn run_init() -> Result<()> {
    let home = home_dir()?;
    let claude_dir = home.join(".claude");
    let dashboard_dir = claude_dir.join("dashboard");
    let hooks_dir = claude_dir.join("hooks");
    let hook_file = hooks_dir.join("event-logger.js");
    let settings_file = claude_dir.join("settings.json");

    // Step 1: Create directories
    println!("[1/3] Creating directories...");
    create_dir_if_missing(&dashboard_dir)?;
    create_dir_if_missing(&hooks_dir)?;

    // Step 2: Deploy event-logger.js
    println!("[2/3] Deploying event-logger.js...");
    deploy_hook_script(&hook_file)?;

    // Step 3: Patch settings.json
    println!("[3/3] Patching settings.json...");
    patch_settings(&settings_file)?;

    println!();
    println!("Setup complete! Run `simple-claude-board` to start the dashboard.");
    Ok(())
}

/// Get the user's home directory.
fn home_dir() -> Result<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .context("Could not determine home directory (HOME or USERPROFILE)")
}

/// Create a directory if it does not already exist.
fn create_dir_if_missing(path: &PathBuf) -> Result<()> {
    if path.is_dir() {
        println!("  Already exists: {}", path.display());
    } else {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create directory: {}", path.display()))?;
        println!("  Created: {}", path.display());
    }
    Ok(())
}

/// Write the embedded event-logger.js to disk.
fn deploy_hook_script(path: &PathBuf) -> Result<()> {
    if path.is_file() {
        println!("  Overwriting: {}", path.display());
    } else {
        println!("  Writing: {}", path.display());
    }
    fs::write(path, EVENT_LOGGER_JS)
        .with_context(|| format!("Failed to write hook script: {}", path.display()))?;
    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o755);
        fs::set_permissions(path, perms)
            .with_context(|| format!("Failed to set permissions: {}", path.display()))?;
    }
    Ok(())
}

/// Build the hook entry JSON value.
fn build_hook_entry() -> Value {
    serde_json::json!({
        "matcher": HOOK_MATCHER,
        "hooks": [{
            "type": "command",
            "command": HOOK_COMMAND,
            "timeout": HOOK_TIMEOUT
        }]
    })
}

/// Check if a hook array already contains an event-logger entry.
fn has_event_logger_entry(arr: &[Value]) -> bool {
    arr.iter().any(|entry| {
        entry
            .get("hooks")
            .and_then(|h| h.as_array())
            .map(|hooks| {
                hooks.iter().any(|hook| {
                    hook.get("command")
                        .and_then(|c| c.as_str())
                        .is_some_and(|cmd| cmd.contains("event-logger.js"))
                })
            })
            .unwrap_or(false)
    })
}

/// Read, patch, and write settings.json.
fn patch_settings(path: &PathBuf) -> Result<()> {
    // Read existing settings or start with empty object
    let mut settings: Value = if path.is_file() {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read: {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse: {}", path.display()))?
    } else {
        serde_json::json!({})
    };

    let root = settings
        .as_object_mut()
        .context("settings.json root is not an object")?;

    // Ensure "hooks" object exists
    if !root.contains_key("hooks") {
        root.insert("hooks".to_string(), serde_json::json!({}));
    }
    let hooks = root
        .get_mut("hooks")
        .and_then(|v| v.as_object_mut())
        .context("settings.json 'hooks' is not an object")?;

    let entry = build_hook_entry();
    let mut patched = false;

    for key in &["PreToolUse", "PostToolUse"] {
        if !hooks.contains_key(*key) {
            hooks.insert((*key).to_string(), serde_json::json!([]));
        }
        let arr = hooks
            .get_mut(*key)
            .and_then(|v| v.as_array_mut())
            .with_context(|| format!("settings.json 'hooks.{key}' is not an array"))?;

        if has_event_logger_entry(arr) {
            println!("  hooks.{key}: event-logger already registered");
        } else {
            arr.push(entry.clone());
            println!("  hooks.{key}: added event-logger entry");
            patched = true;
        }
    }

    if patched {
        let pretty =
            serde_json::to_string_pretty(&settings).context("Failed to serialize settings.json")?;
        fs::write(path, pretty.as_bytes())
            .with_context(|| format!("Failed to write: {}", path.display()))?;
        println!("  Saved: {}", path.display());
    } else {
        println!("  No changes needed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_hook_entry_shape() {
        let entry = build_hook_entry();
        assert_eq!(entry["matcher"], "Task|Edit|Write|Read|Bash|Grep|Glob");
        let hooks = entry["hooks"].as_array().expect("hooks is array");
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0]["type"], "command");
        assert!(hooks[0]["command"]
            .as_str()
            .expect("command is string")
            .contains("event-logger.js"));
        assert_eq!(hooks[0]["timeout"], 3);
    }

    #[test]
    fn test_has_event_logger_entry_detects_existing() {
        let entry = build_hook_entry();
        assert!(has_event_logger_entry(&[entry]));
    }

    #[test]
    fn test_has_event_logger_entry_empty() {
        assert!(!has_event_logger_entry(&[]));
    }

    #[test]
    fn test_has_event_logger_entry_other_hooks() {
        let other = serde_json::json!({
            "matcher": "Bash",
            "hooks": [{"type": "command", "command": "echo hi", "timeout": 1}]
        });
        assert!(!has_event_logger_entry(&[other]));
    }

    #[test]
    fn test_patch_settings_creates_from_scratch() {
        let dir = tempfile::tempdir().expect("tempdir");
        let settings_path = dir.path().join("settings.json");

        patch_settings(&settings_path).expect("patch succeeds");

        let content = fs::read_to_string(&settings_path).expect("read");
        let val: Value = serde_json::from_str(&content).expect("parse");

        let pre = val["hooks"]["PreToolUse"].as_array().expect("array");
        let post = val["hooks"]["PostToolUse"].as_array().expect("array");
        assert_eq!(pre.len(), 1);
        assert_eq!(post.len(), 1);
        assert!(has_event_logger_entry(pre));
        assert!(has_event_logger_entry(post));
    }

    #[test]
    fn test_patch_settings_preserves_existing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let settings_path = dir.path().join("settings.json");

        let existing = serde_json::json!({
            "model": "opus",
            "hooks": {
                "PreToolUse": [{
                    "matcher": "Bash",
                    "hooks": [{"type": "command", "command": "echo safety", "timeout": 5}]
                }]
            }
        });
        fs::write(
            &settings_path,
            serde_json::to_string_pretty(&existing).unwrap(),
        )
        .expect("write");

        patch_settings(&settings_path).expect("patch succeeds");

        let content = fs::read_to_string(&settings_path).expect("read");
        let val: Value = serde_json::from_str(&content).expect("parse");

        // Existing fields preserved
        assert_eq!(val["model"], "opus");

        // Existing hook preserved + event-logger added
        let pre = val["hooks"]["PreToolUse"].as_array().expect("array");
        assert_eq!(pre.len(), 2);
        assert_eq!(pre[0]["matcher"], "Bash");
        assert!(has_event_logger_entry(pre));
    }

    #[test]
    fn test_patch_settings_idempotent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let settings_path = dir.path().join("settings.json");

        patch_settings(&settings_path).expect("first patch");
        let first = fs::read_to_string(&settings_path).expect("read");

        patch_settings(&settings_path).expect("second patch");
        let second = fs::read_to_string(&settings_path).expect("read");

        // Content should be identical (no duplicate entries)
        assert_eq!(first, second);
    }
}
