use crate::utils::net::dl_unzip;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ProjectAction {
    /// Initialize a new project
    Init {
        /// Name of the project
        name: String,
    },
    /// Get information about a project
    Info {
        /// Name of the project
        name: String,
    },
}

pub async fn handle_project_command(action: ProjectAction) {
    match action {
        ProjectAction::Init { name } => {
            println!("Initializing project: {}", name);

            if std::path::Path::new(&name).exists() {
                eprintln!("Error: Directory '{}' already exists", name);
                return;
            }

            match dl_unzip(
                "https://github.com/agnosticeng/init/archive/refs/heads/main.zip",
                &name,
            )
            .await
            {
                Ok(()) => println!("Successfully initialized project '{}'", name),
                Err(e) => eprintln!("Error initializing project '{}': {}", name, e),
            }
        }
        ProjectAction::Info { name } => {
            println!("Getting info for project: {}", name);
            // TODO: Implement project info retrieval logic
        }
    }
}
