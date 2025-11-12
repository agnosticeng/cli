use std::path::PathBuf;

use super::super::bin::ensure_required_binaries;
use super::super::fs::filesystem::create_agnostic_working_dir;

/// Result type for initialization operations
pub type InitResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Configuration structure for the CLI application
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Path to the .agnostic working directory
    pub agnostic_dir: PathBuf,
    /// Whether verbose logging is enabled
    #[allow(dead_code)]
    pub verbose: bool,
}

impl AppConfig {
    /// Creates a new AppConfig with default settings
    pub fn new(agnostic_dir: PathBuf) -> Self {
        Self {
            agnostic_dir,
            verbose: false,
        }
    }

    /// Enables verbose logging
    #[allow(dead_code)]
    pub fn with_verbose(mut self) -> Self {
        self.verbose = true;
        self
    }
}

/// Initializes the CLI application environment
///
/// This function performs all necessary setup tasks at application startup:
/// - Creates the ~/.agnostic working directory
/// - Sets up logging (if needed)
/// - Validates system requirements
/// - Returns configuration for the application
///
/// # Returns
///
/// Returns an `AppConfig` struct containing the application configuration,
/// or an error if initialization fails
///
/// # Examples
///
/// ```no_run
/// use cli::utils::app::init::initialize_app;
///
/// #[tokio::main]
/// async fn main() {
///     match initialize_app().await {
///         Ok(config) => {
///             println!("App initialized successfully!");
///             println!("Working directory: {}", config.agnostic_dir.display());
///             // Continue with application logic...
///         },
///         Err(e) => {
///             eprintln!("Failed to initialize application: {}", e);
///             std::process::exit(1);
///         }
///     }
/// }
/// ```
pub async fn initialize_app() -> InitResult<AppConfig> {
    // Create the .agnostic working directory
    let agnostic_dir = create_agnostic_working_dir()
        .map_err(|e| format!("Failed to create agnostic working directory: {}", e))?;

    // Create subdirectories for organization
    create_app_subdirectories(&agnostic_dir)?;

    // Initialize logging (basic setup for now)
    setup_logging(&agnostic_dir)?;

    // Validate system requirements
    validate_system_requirements()?;

    // Download and install required binaries
    let bin_dir = agnostic_dir.join("bin");
    match ensure_required_binaries(&bin_dir).await {
        Ok(_binaries) => {
            // Binary installation messages are handled by ensure_required_binaries
        }
        Err(e) => {
            eprintln!("Warning: Failed to install some binaries: {}", e);
            // Don't fail initialization for binary installation failures
            // The CLI can still work without external binaries in most cases
        }
    }

    // Create and return configuration
    let config = AppConfig::new(agnostic_dir);

    Ok(config)
}

/// Creates necessary subdirectories within the .agnostic directory
fn create_app_subdirectories(agnostic_dir: &PathBuf) -> InitResult<()> {
    use super::super::fs::filesystem::ensure_dir_exists;

    // Create common subdirectories
    let subdirs = ["bin", "user"];

    for subdir in &subdirs {
        let dir_path = agnostic_dir.join(subdir);
        ensure_dir_exists(&dir_path)
            .map_err(|e| format!("Failed to create {} directory: {}", subdir, e))?;
    }

    Ok(())
}

/// Sets up basic logging for the application
fn setup_logging(agnostic_dir: &PathBuf) -> InitResult<()> {
    // For now, this is a placeholder
    // In the future, you might want to set up file logging to ~/.agnostic/logs/
    let _log_dir = agnostic_dir.join("logs");

    // TODO: Implement proper logging setup
    // This could include:
    // - Setting up file rotation
    // - Configuring log levels
    // - Setting up structured logging

    Ok(())
}

/// Validates system requirements for the CLI application
fn validate_system_requirements() -> InitResult<()> {
    // Check if we can write to the home directory
    if std::env::var("HOME").is_err() {
        return Err("HOME environment variable not set".into());
    }

    // Add other system requirement checks as needed
    // For example:
    // - Check for required external tools
    // - Verify network connectivity if needed
    // - Check disk space

    Ok(())
}

/// Performs cleanup operations when the application shuts down
///
/// This function can be called during graceful shutdown to clean up
/// temporary files, save state, etc.
///
/// # Arguments
///
/// * `config` - The application configuration
///
/// # Examples
///
/// ```no_run
/// use cli::utils::app::init::{initialize_app, cleanup_app};
///
/// #[tokio::main]
/// async fn main() {
///     let config = initialize_app().await.unwrap();
///
///     // ... application logic ...
///
///     // Cleanup before exit
///     if let Err(e) = cleanup_app(&config).await {
///         eprintln!("Warning: Cleanup failed: {}", e);
///     }
/// }
/// ```
pub async fn cleanup_app(config: &AppConfig) -> InitResult<()> {
    // Clean up temporary files
    let temp_dir = config.agnostic_dir.join("temp");
    if temp_dir.exists() {
        // Remove old temporary files (keep recent ones)
        cleanup_temp_directory(&temp_dir)?;
    }

    // TODO: Add other cleanup tasks as needed
    // - Save application state
    // - Close database connections
    // - Flush logs

    Ok(())
}

/// Cleans up old temporary files from the temp directory
fn cleanup_temp_directory(temp_dir: &PathBuf) -> InitResult<()> {
    use std::fs;
    use std::time::{Duration, SystemTime};

    // Remove files older than 24 hours
    let cutoff_time = SystemTime::now()
        .checked_sub(Duration::from_secs(24 * 60 * 60))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    if let Ok(entries) = fs::read_dir(temp_dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff_time {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }

    Ok(())
}

/// Gets the path to a specific subdirectory within the agnostic directory
///
/// # Arguments
///
/// * `config` - The application configuration
/// * `subdir` - The subdirectory name
///
/// # Returns
///
/// Returns the full path to the requested subdirectory
///
/// # Examples
///
/// ```no_run
/// use cli::utils::app::init::{initialize_app, get_agnostic_subdir};
///
/// #[tokio::main]
/// async fn main() {
///     let config = initialize_app().await.unwrap();
///
///     let projects_dir = get_agnostic_subdir(&config, "projects");
///     let temp_dir = get_agnostic_subdir(&config, "temp");
///     let cache_dir = get_agnostic_subdir(&config, "cache");
/// }
/// ```
#[allow(dead_code)]
pub fn get_agnostic_subdir(config: &AppConfig, subdir: &str) -> PathBuf {
    config.agnostic_dir.join(subdir)
}

#[cfg(test)]
mod tests {
    use super::*;

    use tempfile::TempDir;

    #[tokio::test]
    async fn test_initialize_app() {
        // This test will create the actual ~/.agnostic directory
        let result = initialize_app().await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.agnostic_dir.exists());
        assert!(config.agnostic_dir.is_dir());

        // Check that subdirectories were created
        let subdirs = ["bin", "user"];
        for subdir in &subdirs {
            let dir_path = config.agnostic_dir.join(subdir);
            assert!(dir_path.exists(), "Subdirectory {} should exist", subdir);
            assert!(
                dir_path.is_dir(),
                "Subdirectory {} should be a directory",
                subdir
            );
        }
    }

    #[test]
    fn test_app_config() {
        let temp_dir = TempDir::new().unwrap();
        let agnostic_path = temp_dir.path().to_path_buf();

        let config = AppConfig::new(agnostic_path.clone());
        assert_eq!(config.agnostic_dir, agnostic_path);
        assert!(!config.verbose);

        let verbose_config = config.with_verbose();
        assert!(verbose_config.verbose);
    }

    #[test]
    fn test_get_agnostic_subdir() {
        let temp_dir = TempDir::new().unwrap();
        let config = AppConfig::new(temp_dir.path().to_path_buf());

        let projects_dir = get_agnostic_subdir(&config, "projects");
        let expected = temp_dir.path().join("projects");
        assert_eq!(projects_dir, expected);
    }

    #[test]
    fn test_validate_system_requirements() {
        // This should pass on most systems
        let result = validate_system_requirements();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_cleanup_app() {
        // Create a temporary config for testing
        let temp_dir = TempDir::new().unwrap();
        let config = AppConfig::new(temp_dir.path().to_path_buf());

        // Create temp subdirectory
        let temp_subdir = config.agnostic_dir.join("temp");
        std::fs::create_dir_all(&temp_subdir).unwrap();

        // Test cleanup
        let result = cleanup_app(&config).await;
        assert!(result.is_ok());
    }
}
