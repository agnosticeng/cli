use clap::{Parser, Subcommand};

mod commands;
mod utils;
use commands::{
    PipelineAction, ProjectAction, SystemAction, UserAction, handle_pipeline_command,
    handle_project_command,
};
use utils::app::{cleanup_app, initialize_app};

use crate::commands::TeamAction;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, short = 'v', env = "VERBOSE")]
    verbose: bool,

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

    System {
        #[command(subcommand)]
        action: SystemAction,
    },

    Team {
        #[command(subcommand)]
        action: TeamAction,
    },

    User {
        #[command(subcommand)]
        action: UserAction,
    },
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Initialize the application environment
    let config = match initialize_app().await {
        Ok(config) => {
            if args.verbose {
                println!("Application initialized successfully");
                println!("Working directory: {}", config.agnostic_dir.display());
                config.with_verbose()
            } else {
                config
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize application: {}", e);
            std::process::exit(1);
        }
    };

    // Handle the command
    match args.command {
        Commands::Project { action } => handle_project_command(action).await,
        Commands::Pipeline { action } => handle_pipeline_command(action).await,
        Commands::System { action } => action.handle(&config).await,
        Commands::Team { action } => action.handle(&config).await,
        Commands::User { action } => action.handle(&config).await,
    };

    // Cleanup on exit
    if let Err(e) = cleanup_app(&config).await {
        eprintln!("Warning: Cleanup failed: {}", e);
    }
}
