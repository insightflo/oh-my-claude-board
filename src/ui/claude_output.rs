//! Claude Code output panel
//!
//! Shows live agent activity: which agents are running, their current tools,
//! and recent errors.

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
}

impl<'a> AgentPanel<'a> {
    pub fn new(state: &'a DashboardState) -> Self {
        Self { state }
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        if self.state.agents.is_empty() {
            return vec![Line::styled(
                "No agent activity",
                Style::default().fg(Color::DarkGray),
            )];
        }

        let mut lines = Vec::new();
        let mut agents: Vec<&AgentState> = self.state.agents.values().collect();
        agents.sort_by_key(|a| &a.agent_id);

        for agent in agents {
            let (status_icon, status_color) = match agent.status {
                AgentStatus::Running => (">>", Color::Green),
                AgentStatus::Error => ("!!", Color::Red),
                AgentStatus::Idle => ("--", Color::DarkGray),
            };

            let mut spans = vec![
                Span::styled(
                    format!(" {status_icon} "),
                    Style::default().fg(status_color),
                ),
                Span::styled(
                    agent.agent_id.clone(),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
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

            lines.push(Line::from(spans));
        }

        lines
    }
}

impl<'a> Widget for AgentPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Agents ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

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
}
