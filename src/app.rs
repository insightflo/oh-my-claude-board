//! App state management and event loop

use crate::data::state::DashboardState;
use crate::ui::gantt::GanttState;
use crate::ui::layout::FocusedPane;

/// Main application state
pub struct App {
    pub running: bool,
    pub dashboard: DashboardState,
    pub gantt_state: GanttState,
    pub focused: FocusedPane,
    pub show_help: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            dashboard: DashboardState::default(),
            gantt_state: GanttState::default(),
            focused: FocusedPane::TaskList,
            show_help: false,
        }
    }

    pub fn with_dashboard(mut self, dashboard: DashboardState) -> Self {
        self.dashboard = dashboard;
        self
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_focus(&mut self) {
        self.focused = self.focused.toggle();
    }

    pub fn move_down(&mut self) {
        self.gantt_state.select_next();
    }

    pub fn move_up(&mut self) {
        self.gantt_state.select_prev();
    }

    /// Get the currently selected task as (phase_idx, task_idx)
    pub fn selected_task(&self) -> Option<(usize, usize)> {
        self.gantt_state.selected_task(&self.dashboard)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_default() {
        let app = App::new();
        assert!(app.running);
        assert!(!app.show_help);
        assert_eq!(app.focused, FocusedPane::TaskList);
    }

    #[test]
    fn app_quit() {
        let mut app = App::new();
        app.quit();
        assert!(!app.running);
    }

    #[test]
    fn app_toggle_help() {
        let mut app = App::new();
        assert!(!app.show_help);
        app.toggle_help();
        assert!(app.show_help);
        app.toggle_help();
        assert!(!app.show_help);
    }

    #[test]
    fn app_toggle_focus() {
        let mut app = App::new();
        assert_eq!(app.focused, FocusedPane::TaskList);
        app.toggle_focus();
        assert_eq!(app.focused, FocusedPane::Detail);
        app.toggle_focus();
        assert_eq!(app.focused, FocusedPane::TaskList);
    }

    #[test]
    fn app_navigation() {
        let input = include_str!("../tests/fixtures/sample_tasks.md");
        let dashboard = DashboardState::from_tasks_content(input).unwrap();
        let mut app = App::new().with_dashboard(dashboard);
        app.gantt_state.total_items = 11;

        app.move_down();
        assert_eq!(app.gantt_state.selected, 1);
        assert_eq!(app.selected_task(), Some((0, 0)));

        app.move_up();
        assert_eq!(app.gantt_state.selected, 0);
        assert!(app.selected_task().is_none()); // phase header
    }

    #[test]
    fn app_with_dashboard() {
        let input = include_str!("../tests/fixtures/sample_tasks.md");
        let dashboard = DashboardState::from_tasks_content(input).unwrap();
        let app = App::new().with_dashboard(dashboard);
        assert_eq!(app.dashboard.total_tasks, 8);
    }
}
