use clap::{Parser, Subcommand};

mod commands;
mod utils;
use commands::{
    PipelineAction, ProjectAction, StatusAction, handle_pipeline_command, handle_project_command,
    handle_status_command,
};
use utils::app::{cleanup_app, initialize_app};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Project management commands
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Pipeline management commands
    Pipeline {
        #[command(subcommand)]
        action: PipelineAction,
    },
    /// Show status of binaries and system
    Status {
        #[command(subcommand)]
        action: StatusAction,
    },
}

#[tokio::main]
async fn main() {
    // Initialize the application environment
    let config = match initialize_app().await {
        Ok(config) => {
            if std::env::var("VERBOSE").is_ok()
                || std::env::args().any(|arg| arg == "--verbose" || arg == "-v")
            {
                println!("Application initialized successfully");
                println!("Working directory: {}", config.agnostic_dir.display());
            }
            config
        }
        Err(e) => {
            eprintln!("Failed to initialize application: {}", e);
            std::process::exit(1);
        }
    };

    let args = Args::parse();

    // Handle the command
    match args.command {
        Commands::Project { action } => handle_project_command(action).await,
        Commands::Pipeline { action } => handle_pipeline_command(action).await,
        Commands::Status { action } => handle_status_command(action, &config).await,
    };

    // Cleanup on exit
    if let Err(e) = cleanup_app(&config).await {
        eprintln!("Warning: Cleanup failed: {}", e);
    }
}
