use std::io;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use oh_my_claude_board::app::App;
use oh_my_claude_board::data::state::DashboardState;
use oh_my_claude_board::event::{key_to_action, poll_event, Action, AppEvent};
use oh_my_claude_board::ui::detail::DetailWidget;
use oh_my_claude_board::ui::gantt::GanttWidget;
use oh_my_claude_board::ui::help::HelpOverlay;
use oh_my_claude_board::ui::layout::{DashboardLayout, FocusedPane};
use oh_my_claude_board::ui::statusbar::StatusBar;

/// Claude Code orchestration TUI dashboard
#[derive(Parser, Debug)]
#[command(name = "oh-my-claude-board", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path to TASKS.md
    #[arg(long, default_value = "./TASKS.md", global = true)]
    tasks: String,

    /// Path to Hook events directory
    #[arg(long, global = true)]
    hooks: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Watch TASKS.md and Hook events in real-time (default)
    Watch,
    /// Initialize configuration
    Init,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Watch) {
        Commands::Watch => run_tui(&cli.tasks, cli.hooks.as_deref()),
        Commands::Init => {
            println!("oh-my-claude-board init (not yet implemented)");
            Ok(())
        }
    }
}

fn run_tui(tasks_path: &str, hooks_dir: Option<&str>) -> Result<()> {
    // Load initial state
    let dashboard = match std::fs::read_to_string(tasks_path) {
        Ok(content) => DashboardState::from_tasks_content(&content)
            .unwrap_or_else(|_| DashboardState::default()),
        Err(_) => DashboardState::default(),
    };

    // Load hook events if directory provided
    let mut dashboard = dashboard;
    if let Some(dir) = hooks_dir {
        let _ = dashboard.load_hook_events(Path::new(dir));
    }

    let mut app = App::new().with_dashboard(dashboard);

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    let tick_rate = Duration::from_millis(250);

    while app.running {
        // Draw
        terminal.draw(|frame| {
            let area = frame.area();
            let layout = DashboardLayout::compute(area);

            // Left panel: Gantt chart
            let gantt = GanttWidget::new(
                &app.dashboard,
                app.focused == FocusedPane::TaskList,
            );
            frame.render_stateful_widget(gantt, layout.task_list, &mut app.gantt_state);

            // Right panel: Detail view
            let selected_task = app.selected_task();
            let detail = DetailWidget::from_selection(
                &app.dashboard,
                selected_task,
                app.gantt_state.selected,
                app.focused == FocusedPane::Detail,
            );
            frame.render_widget(detail, layout.detail);

            // Bottom: Status bar
            let statusbar = StatusBar::new(&app.dashboard);
            frame.render_widget(statusbar, layout.status_bar);

            // Help overlay (on top if active)
            if app.show_help {
                frame.render_widget(HelpOverlay, area);
            }
        })?;

        // Handle events
        if let Some(event) = poll_event(tick_rate)? {
            match event {
                AppEvent::Key(key) => match key_to_action(key) {
                    Action::Quit => app.quit(),
                    Action::MoveDown => app.move_down(),
                    Action::MoveUp => app.move_up(),
                    Action::ToggleFocus => app.toggle_focus(),
                    Action::ToggleHelp => app.toggle_help(),
                    Action::None => {}
                },
                AppEvent::Resize(_, _) => {} // terminal auto-handles resize
                AppEvent::FileChanged(_) => {} // TODO: reload state
                AppEvent::Tick => {}
            }
        }
    }

    Ok(())
}
