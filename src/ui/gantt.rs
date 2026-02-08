//! Gantt chart widget
//!
//! Renders phases and tasks as a vertical list with colored status indicators.
//! Each phase is a section header, tasks are indented rows with status bars.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, StatefulWidget, Widget},
};

use crate::data::state::DashboardState;
use crate::data::tasks_parser::TaskStatus;

/// Selection state for the gantt view
#[derive(Debug, Default, Clone)]
pub struct GanttState {
    /// Index into the flattened list (phases + tasks)
    pub selected: usize,
    /// Total number of selectable items
    pub total_items: usize,
    /// Scroll offset for vertical scrolling
    pub offset: usize,
}

impl GanttState {
    pub fn select_next(&mut self) {
        if self.total_items > 0 {
            self.selected = (self.selected + 1).min(self.total_items - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Get the (phase_idx, task_idx) for the current selection.
    /// Returns None if a phase header is selected.
    pub fn selected_task(&self, state: &DashboardState) -> Option<(usize, usize)> {
        let mut idx = 0;
        for (pi, phase) in state.phases.iter().enumerate() {
            if idx == self.selected {
                return None; // phase header selected
            }
            idx += 1;
            for ti in 0..phase.tasks.len() {
                if idx == self.selected {
                    return Some((pi, ti));
                }
                idx += 1;
            }
        }
        None
    }
}

/// Color for a task status
fn status_color(status: &TaskStatus) -> Color {
    match status {
        TaskStatus::Completed => Color::Green,
        TaskStatus::InProgress => Color::Yellow,
        TaskStatus::Pending => Color::DarkGray,
        TaskStatus::Failed => Color::Red,
        TaskStatus::Blocked => Color::Magenta,
    }
}

/// Status icon character
fn status_icon(status: &TaskStatus) -> &'static str {
    match status {
        TaskStatus::Completed => "[x]",
        TaskStatus::InProgress => "[/]",
        TaskStatus::Pending => "[ ]",
        TaskStatus::Failed => "[!]",
        TaskStatus::Blocked => "[B]",
    }
}

/// The Gantt widget renders the dashboard state as a scrollable task list
pub struct GanttWidget<'a> {
    state: &'a DashboardState,
    focused: bool,
}

impl<'a> GanttWidget<'a> {
    pub fn new(state: &'a DashboardState, focused: bool) -> Self {
        Self { state, focused }
    }

    fn build_lines(&self, gantt_state: &GanttState) -> Vec<(Line<'static>, bool)> {
        let mut lines = Vec::new();
        let mut idx = 0;

        for phase in &self.state.phases {
            let is_selected = idx == gantt_state.selected;
            let progress = phase.progress();
            let pct = (progress * 100.0) as u8;
            let header = Line::from(vec![
                Span::styled(
                    format!(" {} ", phase.id),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    phase.name.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::styled(format!(" ({pct}%)"), Style::default().fg(Color::DarkGray)),
            ]);
            lines.push((header, is_selected));
            idx += 1;

            for task in &phase.tasks {
                let is_selected = idx == gantt_state.selected;
                let icon = status_icon(&task.status);
                let color = status_color(&task.status);
                let agent_str = task
                    .agent
                    .as_deref()
                    .map(|a| format!(" @{a}"))
                    .unwrap_or_default();

                let line = Line::from(vec![
                    Span::raw("  "),
                    Span::styled(icon.to_string(), Style::default().fg(color)),
                    Span::raw(" "),
                    Span::styled(
                        task.id.clone(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(": "),
                    Span::raw(task.name.clone()),
                    Span::styled(agent_str, Style::default().fg(Color::Blue)),
                ]);
                lines.push((line, is_selected));
                idx += 1;
            }
        }
        lines
    }
}

impl<'a> StatefulWidget for GanttWidget<'a> {
    type State = GanttState;

    fn render(self, area: Rect, buf: &mut Buffer, gantt_state: &mut Self::State) {
        let border_style = if self.focused {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(" Tasks ")
            .borders(Borders::ALL)
            .border_style(border_style);
        let inner = block.inner(area);
        block.render(area, buf);

        let lines = self.build_lines(gantt_state);
        gantt_state.total_items = lines.len();

        // Adjust scroll offset to keep selection visible
        let visible_height = inner.height as usize;
        if gantt_state.selected < gantt_state.offset {
            gantt_state.offset = gantt_state.selected;
        } else if gantt_state.selected >= gantt_state.offset + visible_height {
            gantt_state.offset = gantt_state.selected - visible_height + 1;
        }

        for (i, (line, is_selected)) in lines
            .iter()
            .skip(gantt_state.offset)
            .enumerate()
            .take(visible_height)
        {
            let y = inner.y + i as u16;
            if y >= inner.y + inner.height {
                break;
            }

            if *is_selected && self.focused {
                buf.set_style(
                    Rect::new(inner.x, y, inner.width, 1),
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                );
            }

            let line_area = Rect::new(inner.x, y, inner.width, 1);
            Widget::render(line.clone(), line_area, buf);
        }
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
    fn gantt_state_navigation() {
        let mut gs = GanttState {
            selected: 0,
            total_items: 5,
            offset: 0,
        };
        gs.select_next();
        assert_eq!(gs.selected, 1);
        gs.select_prev();
        assert_eq!(gs.selected, 0);
        gs.select_prev(); // should not go below 0
        assert_eq!(gs.selected, 0);
    }

    #[test]
    fn gantt_state_max_bound() {
        let mut gs = GanttState {
            selected: 4,
            total_items: 5,
            offset: 0,
        };
        gs.select_next(); // should cap at 4
        assert_eq!(gs.selected, 4);
    }

    #[test]
    fn selected_task_phase_header() {
        let state = sample_state();
        let gs = GanttState {
            selected: 0,
            total_items: 11,
            offset: 0,
        };
        assert!(gs.selected_task(&state).is_none());
    }

    #[test]
    fn selected_task_first_task() {
        let state = sample_state();
        let gs = GanttState {
            selected: 1,
            total_items: 11,
            offset: 0,
        };
        assert_eq!(gs.selected_task(&state), Some((0, 0)));
    }

    #[test]
    fn selected_task_second_phase() {
        let state = sample_state();
        // Phase 0: header(0) + 2 tasks(1,2) = 3 items
        // Phase 1: header(3)
        let gs = GanttState {
            selected: 3,
            total_items: 11,
            offset: 0,
        };
        assert!(gs.selected_task(&state).is_none()); // phase 1 header
        let gs2 = GanttState {
            selected: 4,
            total_items: 11,
            offset: 0,
        };
        assert_eq!(gs2.selected_task(&state), Some((1, 0)));
    }

    #[test]
    fn status_colors_all_mapped() {
        assert_eq!(status_color(&TaskStatus::Completed), Color::Green);
        assert_eq!(status_color(&TaskStatus::InProgress), Color::Yellow);
        assert_eq!(status_color(&TaskStatus::Pending), Color::DarkGray);
        assert_eq!(status_color(&TaskStatus::Failed), Color::Red);
        assert_eq!(status_color(&TaskStatus::Blocked), Color::Magenta);
    }

    #[test]
    fn status_icons_all_mapped() {
        assert_eq!(status_icon(&TaskStatus::Completed), "[x]");
        assert_eq!(status_icon(&TaskStatus::InProgress), "[/]");
        assert_eq!(status_icon(&TaskStatus::Pending), "[ ]");
        assert_eq!(status_icon(&TaskStatus::Failed), "[!]");
        assert_eq!(status_icon(&TaskStatus::Blocked), "[B]");
    }

    #[test]
    fn build_lines_count() {
        let state = sample_state();
        let widget = GanttWidget::new(&state, true);
        let gs = GanttState::default();
        let lines = widget.build_lines(&gs);
        // 3 phases + 8 tasks = 11 lines
        assert_eq!(lines.len(), 11);
    }

    #[test]
    fn render_does_not_panic() {
        let state = sample_state();
        let widget = GanttWidget::new(&state, true);
        let mut gs = GanttState::default();
        let area = Rect::new(0, 0, 60, 20);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf, &mut gs);
        assert_eq!(gs.total_items, 11);
    }
}
