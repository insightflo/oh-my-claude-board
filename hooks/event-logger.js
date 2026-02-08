#!/usr/bin/env node
/**
 * PreToolUse/PostToolUse Hook: Event Logger for Dashboard
 *
 * Appends JSONL lines to ~/.claude/dashboard/events.jsonl on every tool use.
 * The simple-claude-board dashboard watches this file for live agent activity.
 *
 * This is a standalone version with no external dependencies.
 *
 * Hook events captured:
 *   PreToolUse[Task]              -> agent_start
 *   PostToolUse[Task]             -> agent_end
 *   PreToolUse[Edit|Write|...]    -> tool_start
 *   PostToolUse[Edit|Write|...]   -> tool_end
 */

const fs = require('fs');
const path = require('path');
const os = require('os');

const EVENTS_DIR = path.join(os.homedir(), '.claude', 'dashboard');
const EVENTS_FILE = path.join(EVENTS_DIR, 'events.jsonl');
const SESSION_ID_FILE = path.join(os.tmpdir(), 'claude-dashboard-session-id');

// Tools we track (Task is handled separately as agent events)
const TRACKED_TOOLS = new Set([
  'Edit', 'Write', 'Read', 'Bash', 'Grep', 'Glob',
  'NotebookEdit', 'WebFetch', 'WebSearch'
]);

/**
 * Read and parse JSON from stdin (Claude hook input).
 * @returns {Promise<object>} Parsed JSON object
 */
function readStdin() {
  return new Promise((resolve, reject) => {
    let data = '';
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (chunk) => { data += chunk; });
    process.stdin.on('end', () => {
      try {
        const parsed = data.trim() ? JSON.parse(data) : {};
        resolve(parsed);
      } catch (e) {
        resolve({});
      }
    });
    process.stdin.on('error', reject);
  });
}

/**
 * Get or create a stable session ID for the current process tree.
 */
function getSessionId() {
  try {
    if (fs.existsSync(SESSION_ID_FILE)) {
      const content = fs.readFileSync(SESSION_ID_FILE, 'utf8').trim();
      if (content) return content;
    }
  } catch {
    // Fall through to generate
  }

  const id = `sess-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;
  try {
    fs.writeFileSync(SESSION_ID_FILE, id, 'utf8');
  } catch {
    // Non-fatal
  }
  return id;
}

/**
 * Append a single JSONL line to the events file.
 */
function appendEvent(event) {
  try {
    if (!fs.existsSync(EVENTS_DIR)) {
      fs.mkdirSync(EVENTS_DIR, { recursive: true });
    }
    fs.appendFileSync(EVENTS_FILE, JSON.stringify(event) + '\n', 'utf8');
  } catch {
    // Silent - hooks must never break the session
  }
}

/**
 * Determine if this is a Pre or Post hook from the hook_event_name field.
 */
function isPreHook(hookEventName) {
  return hookEventName && hookEventName.startsWith('PreToolUse');
}

/**
 * Try to extract a short task identifier from a Task prompt.
 * Looks for patterns like "P1-T1", "P1-R1-T1", etc.
 */
function extractTaskId(prompt) {
  if (!prompt) return null;
  // Match "P1-T1" or "P1-R1-T1" style IDs
  const match = prompt.match(/\b(P\d+-(?:R\d+-)?T\d+)\b/i);
  if (match) return match[1];
  // Match "task #N" or "Task N"
  const taskMatch = prompt.match(/task\s*#?(\d+)/i);
  if (taskMatch) return `task-${taskMatch[1]}`;
  return null;
}

async function main() {
  const input = await readStdin();
  const hookEventName = input.hook_event_name || '';
  const toolInput = input.tool_input || {};
  const toolName = toolInput.tool_name || input.tool_name || '';

  // Determine agent_id from env or default
  const agentId = process.env.CLAUDE_AGENT_ROLE || 'main';
  const sessionId = getSessionId();
  const timestamp = new Date().toISOString();

  const pre = isPreHook(hookEventName);

  // Task tool -> agent_start / agent_end
  if (toolName === 'Task' || hookEventName.includes('Task')) {
    const subagentType = toolInput.subagent_type || 'unknown';
    const taskId = extractTaskId(toolInput.prompt) || 'unknown';

    appendEvent({
      event_type: pre ? 'agent_start' : 'agent_end',
      timestamp,
      agent_id: subagentType,
      task_id: taskId,
      session_id: sessionId,
      tool_name: subagentType,
    });
    return;
  }

  // Other tracked tools -> tool_start / tool_end
  if (TRACKED_TOOLS.has(toolName)) {
    appendEvent({
      event_type: pre ? 'tool_start' : 'tool_end',
      timestamp,
      agent_id: agentId,
      task_id: 'unknown',
      session_id: sessionId,
      tool_name: toolName,
    });
    return;
  }
}

main().catch(() => {
  // Silent exit - hooks must never break the session
});
