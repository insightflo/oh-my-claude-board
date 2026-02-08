//! Screen split layout
//!
//! Defines the main dashboard layout: task list (left), detail panel (right),
//! and status bar (bottom).

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// The pane that currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedPane {
    TaskList,
    Detail,
    Agents,
}

impl FocusedPane {
    pub fn toggle(self) -> Self {
        match self {
            Self::TaskList => Self::Detail,
            Self::Detail => Self::Agents,
            Self::Agents => Self::TaskList,
        }
    }
}

/// Computed layout areas for the dashboard
pub struct DashboardLayout {
    pub task_list: Rect,
    pub detail: Rect,
    pub agents: Rect,
    pub status_bar: Rect,
}

impl DashboardLayout {
    /// Compute layout from terminal area
    ///
    /// ```text
    /// +------ 55% ------+------ 45% ------+
    /// |                  |     Detail      |
    /// |    Task List     |                 |
    /// |                  +-----------------+
    /// |                  |     Agents      |
    /// +------------------+-----------------+
    /// |            Status Bar              |
    /// +------------------------------------+
    /// ```
    pub fn compute(area: Rect) -> Self {
        let vertical = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(1)])
            .split(area);

        let horizontal = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(vertical[0]);

        // Split right panel: detail (top 70%) + agents (bottom 30%)
        let right_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(horizontal[1]);

        Self {
            task_list: horizontal[0],
            detail: right_split[0],
            agents: right_split[1],
            status_bar: vertical[1],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_toggle_3way() {
        assert_eq!(FocusedPane::TaskList.toggle(), FocusedPane::Detail);
        assert_eq!(FocusedPane::Detail.toggle(), FocusedPane::Agents);
        assert_eq!(FocusedPane::Agents.toggle(), FocusedPane::TaskList);
    }

    #[test]
    fn layout_standard_size() {
        let area = Rect::new(0, 0, 120, 40);
        let layout = DashboardLayout::compute(area);
        assert!(layout.task_list.width > 0);
        assert!(layout.detail.width > 0);
        assert!(layout.agents.width > 0);
        assert_eq!(layout.status_bar.height, 1);
        assert_eq!(layout.detail.width, layout.agents.width);
    }

    #[test]
    fn layout_small_size() {
        let area = Rect::new(0, 0, 40, 10);
        let layout = DashboardLayout::compute(area);
        assert!(layout.task_list.width > 0);
        assert!(layout.detail.width > 0);
        assert_eq!(layout.status_bar.height, 1);
    }

    #[test]
    fn layout_statusbar_at_bottom() {
        let area = Rect::new(0, 0, 80, 30);
        let layout = DashboardLayout::compute(area);
        assert_eq!(layout.status_bar.y, area.height - 1);
    }
}
