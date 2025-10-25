use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum PipelineAction {
    /// Spawn a new pipeline with S3 server
    Spawn {
        /// Name of the pipeline
        name: String,
    },
    /// Get information about a pipeline
    Info {
        /// Name of the pipeline
        name: String,
    },
}

pub async fn handle_pipeline_command(action: PipelineAction) {
    match action {
        PipelineAction::Spawn { name } => {
            println!("Spawning pipeline: {}", name);
        }
        PipelineAction::Info { name } => {
            println!("Getting info for pipeline: {}", name);
            // TODO: Implement pipeline info retrieval logic
        }
    }
}
