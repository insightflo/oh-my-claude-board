use anyhow::Result;
use clap::Parser;

mod app;
mod config;
mod event;

/// Claude Code orchestration TUI dashboard
#[derive(Parser, Debug)]
#[command(name = "oh-my-claude-board", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Watch TASKS.md and Hook events in real-time
    Watch {
        /// Path to TASKS.md
        #[arg(long, default_value = "./TASKS.md")]
        tasks: String,

        /// Path to Hook events directory
        #[arg(long)]
        hooks: Option<String>,

        /// Screen split ratio (left:right)
        #[arg(long, default_value = "50")]
        split: u16,

        /// Disable AI error analysis
        #[arg(long)]
        no_ai: bool,
    },
    /// Initialize configuration
    Init,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Watch {
            tasks,
            hooks,
            split,
            no_ai,
        } => {
            tracing_subscriber::fmt::init();
            tracing::info!("Starting oh-my-claude-board watch mode");
            tracing::info!(tasks = %tasks, hooks = ?hooks, split = split, no_ai = no_ai);

            let mut app = app::App::new();
            // TODO: Start TUI event loop
            println!("oh-my-claude-board watch mode (not yet implemented)");
            app.quit();
            Ok(())
        }
        Commands::Init => {
            println!("oh-my-claude-board init (not yet implemented)");
            Ok(())
        }
    }
}
