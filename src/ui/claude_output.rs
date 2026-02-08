//! Claude Code output panel
//!
//! Shows live agent activity: which agents are running, their current tools,
//! and recent errors. Highlights the agent assigned to the currently selected task.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use crate::data::state::{AgentState, AgentStatus, DashboardState};

/// Agent activity panel widget
pub struct AgentPanel<'a> {
    state: &'a DashboardState,
    /// Agent name assigned to the currently selected task (from TASKS.md `@agent`)
    selected_agent: Option<&'a str>,
    focused: bool,
    selected_index: usize,
}

impl<'a> AgentPanel<'a> {
    pub fn new(state: &'a DashboardState) -> Self {
        Self {
            state,
            selected_agent: None,
            focused: false,
            selected_index: 0,
        }
    }

    pub fn with_selected_agent(mut self, agent: Option<&'a str>) -> Self {
        self.selected_agent = agent;
        self
    }

    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn with_selected_index(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        if self.state.agents.is_empty() && self.selected_agent.is_none() {
            return vec![Line::styled(
                " No agent activity",
                Style::default().fg(Color::DarkGray),
            )];
        }

        let mut lines = Vec::new();

        // Show selected task's assigned agent header if present
        if let Some(agent_name) = self.selected_agent {
            lines.push(Line::from(vec![
                Span::styled(" Task agent: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    format!("@{agent_name}"),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        let mut agents: Vec<&AgentState> = self.state.agents.values().collect();
        agents.sort_by_key(|a| &a.agent_id);

        for (idx, agent) in agents.iter().enumerate() {
            let is_selected = self.focused && idx == self.selected_index;
            let is_highlighted = is_selected
                || self
                    .selected_agent
                    .is_some_and(|name| agent.agent_id.contains(name));

            let (status_icon, status_color) = match agent.status {
                AgentStatus::Running => (">>", Color::Green),
                AgentStatus::Error => ("!!", Color::Red),
                AgentStatus::Idle => ("--", Color::DarkGray),
            };

            let name_style = if is_highlighted {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            };

            let prefix = if is_selected { ">" } else { " " };
            let mut spans = vec![
                Span::styled(
                    format!("{prefix}{status_icon} "),
                    Style::default().fg(status_color),
                ),
                Span::styled(agent.agent_id.clone(), name_style),
            ];

            if let Some(ref task) = agent.current_task {
                spans.push(Span::styled(
                    format!(" [{task}]"),
                    Style::default().fg(Color::Cyan),
                ));
            }

            if let Some(ref tool) = agent.current_tool {
                spans.push(Span::styled(
                    format!(" -> {tool}"),
                    Style::default().fg(Color::Yellow),
                ));
            }

            if agent.error_count > 0 {
                spans.push(Span::styled(
                    format!(" ({} errs)", agent.error_count),
                    Style::default().fg(Color::Red),
                ));
            }

            spans.push(Span::styled(
                format!(" ({}ev)", agent.event_count),
                Style::default().fg(Color::DarkGray),
            ));

            lines.push(Line::from(spans));

            // Show most recent error for this agent (if any)
            if let Some(err) = self
                .state
                .recent_errors
                .iter()
                .rev()
                .find(|e| e.agent_id == agent.agent_id)
            {
                let retry_str = if err.retryable { "retry" } else { "no retry" };
                let msg_short = if err.message.len() > 40 {
                    format!("{}...", &err.message[..37])
                } else {
                    err.message.clone()
                };
                lines.push(Line::from(vec![
                    Span::styled("    !! ", Style::default().fg(Color::Red)),
                    Span::styled(msg_short, Style::default().fg(Color::Red)),
                    Span::styled(
                        format!(" → {} ({retry_str})", err.category),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]));
            }
        }

        if lines.is_empty() {
            lines.push(Line::styled(
                " No agent activity",
                Style::default().fg(Color::DarkGray),
            ));
        }

        lines
    }
}

impl<'a> Widget for AgentPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_color = if self.focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        let block = Block::default()
            .title(" Agents ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let lines = self.build_lines();
        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });
        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::hook_parser;

    fn state_with_agents() -> DashboardState {
        let input = include_str!("../../tests/fixtures/sample_hooks/agent_events.jsonl");
        let result = hook_parser::parse_hook_events(input);
        let mut state = DashboardState::default();
        state.update_from_events(&result.events);
        state
    }

    #[test]
    fn agent_panel_empty() {
        let state = DashboardState::default();
        let panel = AgentPanel::new(&state);
        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);
        panel.render(area, &mut buf);
    }

    #[test]
    fn agent_panel_with_agents() {
        let state = state_with_agents();
        let panel = AgentPanel::new(&state);
        let area = Rect::new(0, 0, 60, 10);
        let mut buf = Buffer::empty(area);
        panel.render(area, &mut buf);
    }

    #[test]
    fn build_lines_with_agents() {
        let state = state_with_agents();
        let panel = AgentPanel::new(&state);
        let lines = panel.build_lines();
        assert!(!lines.is_empty());
    }

    #[test]
    fn build_lines_empty() {
        let state = DashboardState::default();
        let panel = AgentPanel::new(&state);
        let lines = panel.build_lines();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn with_selected_agent_highlights() {
        let state = state_with_agents();
        let panel = AgentPanel::new(&state).with_selected_agent(Some("backend-specialist"));
        let lines = panel.build_lines();
        // Should have header line + agent lines
        assert!(lines.len() >= 2);
    }

    fn state_with_errors() -> DashboardState {
        let input = include_str!("../../tests/fixtures/sample_hooks/error_events.jsonl");
        let result = hook_parser::parse_hook_events(input);
        let mut state = DashboardState::default();
        state.update_from_events(&result.events);
        state
    }

    #[test]
    fn build_lines_shows_error_summary() {
        let state = state_with_errors();
        let panel = AgentPanel::new(&state);
        let lines = panel.build_lines();
        // Should have agent line + error summary line
        assert!(lines.len() >= 2);
        let error_line = lines
            .iter()
            .find(|l| l.spans.iter().any(|s| s.content.contains("!!")));
        assert!(error_line.is_some(), "should have error summary line");
    }

    #[test]
    fn error_summary_shows_category() {
        let state = state_with_errors();
        let panel = AgentPanel::new(&state);
        let lines = panel.build_lines();
        let has_category = lines.iter().any(|l| {
            l.spans
                .iter()
                .any(|s| s.content.contains("Network") || s.content.contains("Permission"))
        });
        assert!(has_category, "error summary should show category");
    }

    #[test]
    fn focused_panel_highlights_selected() {
        let state = state_with_agents();
        let panel = AgentPanel::new(&state)
            .with_focused(true)
            .with_selected_index(0);
        let lines = panel.build_lines();
        // Should have selection indicator ">" on the first agent line
        let has_selector = lines
            .iter()
            .any(|l| l.spans.iter().any(|s| s.content.starts_with('>')));
        assert!(has_selector, "focused panel should show > selector");
    }

    #[test]
    fn unfocused_panel_no_selector() {
        let state = state_with_agents();
        let panel = AgentPanel::new(&state)
            .with_focused(false)
            .with_selected_index(0);
        let lines = panel.build_lines();
        let has_selector = lines
            .iter()
            .any(|l| l.spans.iter().any(|s| s.content.starts_with('>')));
        assert!(!has_selector, "unfocused panel should not show > selector");
    }

    #[test]
    fn selected_agent_no_match_still_shows_header() {
        let state = DashboardState::default();
        let panel = AgentPanel::new(&state).with_selected_agent(Some("nonexistent"));
        let lines = panel.build_lines();
        // Header line + "No agent activity" would be empty agents but header exists
        assert!(!lines.is_empty());
    }
}
