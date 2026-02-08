//! Status bar widget
//!
//! Shows overall progress, active agent count, and keybinding hints.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Widget,
};

use crate::data::state::{AgentStatus, DashboardState};

/// Status bar at the bottom of the screen
pub struct StatusBar<'a> {
    state: &'a DashboardState,
}

impl<'a> StatusBar<'a> {
    pub fn new(state: &'a DashboardState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for StatusBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let pct = (self.state.overall_progress * 100.0) as u8;
        let running_agents = self
            .state
            .agents
            .values()
            .filter(|a| a.status == AgentStatus::Running)
            .count();
        let error_agents = self
            .state
            .agents
            .values()
            .filter(|a| a.status == AgentStatus::Error)
            .count();

        let mut spans = vec![
            Span::styled(
                format!(
                    " {}/{} tasks ({pct}%) ",
                    self.state.completed_tasks, self.state.total_tasks
                ),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(" {running_agents} active "),
                Style::default().fg(Color::Black).bg(Color::Yellow),
            ),
        ];

        if error_agents > 0 {
            spans.push(Span::styled(
                format!(" {error_agents} errors "),
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Red)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        if self.state.failed_tasks > 0 {
            spans.push(Span::styled(
                format!(" {} failed ", self.state.failed_tasks),
                Style::default().fg(Color::White).bg(Color::Red),
            ));
        }

        // Fill remaining width with keybinding hints
        let used_width: usize = spans.iter().map(|s| s.content.len()).sum();
        let hints = " j/k:nav  Tab:focus  q:quit  ?:help ";
        let remaining = area.width as usize - used_width.min(area.width as usize);
        if remaining > hints.len() {
            let padding = remaining - hints.len();
            spans.push(Span::raw(" ".repeat(padding)));
        }
        spans.push(Span::styled(hints, Style::default().fg(Color::DarkGray)));

        let line = Line::from(spans);
        Widget::render(line, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_state() -> DashboardState {
        let input = include_str!("../../tests/fixtures/sample_tasks.md");
        DashboardState::from_tasks_content(input).unwrap()
    }

    #[test]
    fn statusbar_renders() {
        let state = sample_state();
        let bar = StatusBar::new(&state);
        let area = Rect::new(0, 0, 80, 1);
        let mut buf = Buffer::empty(area);
        bar.render(area, &mut buf);
    }

    #[test]
    fn statusbar_narrow_renders() {
        let state = sample_state();
        let bar = StatusBar::new(&state);
        let area = Rect::new(0, 0, 20, 1);
        let mut buf = Buffer::empty(area);
        bar.render(area, &mut buf);
    }
}
