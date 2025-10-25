//! Comprehensive binary management system
//!
//! This module provides a complete solution for managing external binary dependencies
//! such as s3fs, ClickHouse, and agt. It combines type definitions, core functionality,
//! and provider coordination in a single, efficient module.

use futures_util::TryStreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

// Re-export binary providers
pub use crate::utils::bin::agt::provider as agt;
pub use crate::utils::bin::clickhouse::provider as clickhouse;
pub use crate::utils::bin::s3fs::provider as s3fs;

/// Result type for binary operations
pub type BinResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Supported system architectures and platforms
#[derive(Debug, Clone, PartialEq)]
pub enum SystemTarget {
    MacOsAarch64,
    MacOsX86_64,
    LinuxX86_64,
}

impl SystemTarget {
    /// Detect the current system platform and architecture
    pub fn detect() -> BinResult<Self> {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        match (os, arch) {
            ("macos", "aarch64") => Ok(SystemTarget::MacOsAarch64),
            ("macos", "x86_64") => Ok(SystemTarget::MacOsX86_64),
            ("linux", "x86_64") => Ok(SystemTarget::LinuxX86_64),
            _ => Err(format!("Unsupported system: {} {}", os, arch).into()),
        }
    }
}

/// Information about a binary's status
#[derive(Debug, Clone)]
pub struct BinaryInfo {
    /// Name of the binary
    pub name: String,
    /// Path to the binary
    pub path: PathBuf,
    /// Whether the binary file exists
    pub exists: bool,
    /// Whether the binary is executable
    pub executable: bool,
    /// Size of the binary in bytes
    pub size: Option<u64>,
}

impl BinaryInfo {
    /// Create BinaryInfo from a path
    pub fn from_path(name: String, path: PathBuf) -> Self {
        let exists = path.exists();
        let executable = if exists {
            is_executable(&path).unwrap_or(false)
        } else {
            false
        };
        let size = if exists {
            std::fs::metadata(&path).ok().map(|m| m.len())
        } else {
            None
        };

        Self {
            name,
            path,
            exists,
            executable,
            size,
        }
    }

    /// Check if the binary is ready (exists and is executable)
    pub fn is_ready(&self) -> bool {
        self.exists && self.executable
    }
}

/// Trait that all binary information providers must implement
pub trait BinaryInfoProvider: Send + Sync {
    /// The display name of the binary (e.g., "ClickHouse", "s3fs", "agt")
    fn name(&self) -> &'static str;

    /// The local filename to save the binary as (e.g., "clickhouse", "s3fs", "agt")
    fn local_name(&self) -> &'static str;

    /// Generate the download URL for this binary on the given platform
    fn get_download_url(&self, target: &SystemTarget) -> String;

    /// Arguments to pass to get version info (e.g., ["--version"] or ["--help"])
    fn version_args(&self) -> &[&str];

    /// Parse version information from the command output
    fn parse_version_output(&self, output: &str) -> Option<String>;
}

/// Registry of all available binary providers
pub struct ProviderRegistry {
    providers: Vec<Box<dyn BinaryInfoProvider>>,
}

impl ProviderRegistry {
    /// Create a new provider registry with all available providers
    fn new() -> Self {
        let providers: Vec<Box<dyn BinaryInfoProvider>> =
            vec![Box::new(s3fs()), Box::new(clickhouse()), Box::new(agt())];

        Self { providers }
    }

    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<&dyn BinaryInfoProvider> {
        self.providers
            .iter()
            .find(|p| p.name() == name)
            .map(|p| p.as_ref())
    }

    /// Get status of all binary providers
    pub fn get_all_status<P: AsRef<Path>>(&self, bin_dir: P) -> Vec<BinaryInfo> {
        let bin_dir = bin_dir.as_ref();
        self.providers
            .iter()
            .map(|provider| get_binary_info(provider.as_ref(), bin_dir))
            .collect()
    }

    /// Ensures all required binaries are installed
    pub async fn ensure_all_binaries<P: AsRef<Path>>(&self, bin_dir: P) -> BinResult<Vec<PathBuf>> {
        let bin_dir = bin_dir.as_ref();
        let mut installed_binaries = Vec::new();
        let mut newly_installed = 0;

        for provider in &self.providers {
            let binary_exists = get_binary_info(provider.as_ref(), bin_dir).exists;
            if !binary_exists {
                println!("Installing {} binary...", provider.name());
            }
            let binary_path = install_binary(provider.as_ref(), bin_dir, false).await?;
            if !binary_exists {
                newly_installed += 1;
            }
            installed_binaries.push(binary_path);
        }

        if newly_installed > 0 {
            println!(
                "Binary setup completed: {} new binaries installed",
                newly_installed
            );
        }

        Ok(installed_binaries)
    }
}

/// Global provider registry instance
static REGISTRY: std::sync::LazyLock<ProviderRegistry> =
    std::sync::LazyLock::new(ProviderRegistry::new);

/// Get the global provider registry
pub fn registry() -> &'static ProviderRegistry {
    &REGISTRY
}

// Core utility functions

/// Downloads a binary from a URL with progress bar
pub async fn download_binary_with_progress(url: &str, binary_name: &str) -> BinResult<Vec<u8>> {
    let client = Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download {} binary: HTTP {}",
            binary_name,
            response.status()
        )
        .into());
    }

    let total_size = response.content_length();

    // Create progress bar
    let progress_bar = if let Some(size) = total_size {
        let pb = ProgressBar::new(size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                .unwrap()
                .progress_chars("#>-"),
        );
        pb.set_message(format!("Downloading {}", binary_name));
        Some(pb)
    } else {
        println!("Starting download (size unknown)...");
        None
    };

    // Stream the download with progress updates
    let mut content = Vec::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.try_next().await? {
        content.extend_from_slice(&chunk);
        if let Some(pb) = &progress_bar {
            pb.set_position(content.len() as u64);
        }
    }

    if let Some(pb) = progress_bar {
        pb.finish_with_message("Download completed");
    } else {
        println!("Download completed: {} bytes", content.len());
    }

    Ok(content)
}

/// Writes binary content to file and makes it executable
pub fn write_and_make_executable<P: AsRef<Path>>(binary_path: P, content: &[u8]) -> BinResult<()> {
    let binary_path = binary_path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = binary_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write binary to file
    fs::write(binary_path, content)?;

    // Make executable on Unix systems
    #[cfg(unix)]
    {
        let mut perms = fs::metadata(binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(binary_path, perms)?;
    }

    Ok(())
}

/// Check if a file exists and is executable
pub fn is_executable<P: AsRef<Path>>(path: P) -> BinResult<bool> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(false);
    }

    #[cfg(unix)]
    {
        let metadata = fs::metadata(path)?;
        let permissions = metadata.permissions();
        Ok(permissions.mode() & 0o111 != 0)
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, just check if file exists
        Ok(true)
    }
}

/// Gets the path to a specific binary in the bin directory
pub fn get_binary_path<P: AsRef<Path>>(bin_dir: P, binary_name: &str) -> PathBuf {
    let bin_dir = bin_dir.as_ref();
    bin_dir.join(binary_name)
}

/// Get the path where a binary should be located using provider info
pub fn get_provider_binary_path<P: AsRef<Path>>(
    provider: &dyn BinaryInfoProvider,
    bin_dir: P,
) -> PathBuf {
    get_binary_path(bin_dir, provider.local_name())
}

/// Check if a binary is installed and ready to use
pub fn is_binary_ready<P: AsRef<Path>>(provider: &dyn BinaryInfoProvider, bin_dir: P) -> bool {
    let path = get_provider_binary_path(provider, bin_dir);
    path.exists() && is_executable(&path).unwrap_or(false)
}

/// Get binary information including status
pub fn get_binary_info<P: AsRef<Path>>(
    provider: &dyn BinaryInfoProvider,
    bin_dir: P,
) -> BinaryInfo {
    let path = get_provider_binary_path(provider, bin_dir);
    BinaryInfo::from_path(provider.name().to_string(), path)
}

/// Install a binary using provider information
pub async fn install_binary<P: AsRef<Path>>(
    provider: &dyn BinaryInfoProvider,
    bin_dir: P,
    force_download: bool,
) -> BinResult<PathBuf> {
    let bin_dir = bin_dir.as_ref();
    let binary_path = get_provider_binary_path(provider, bin_dir);

    // Check if binary already exists and is executable
    if !force_download && binary_path.exists() && is_executable(&binary_path)? {
        return Ok(binary_path);
    }

    let target = SystemTarget::detect()?;
    let download_url = provider.get_download_url(&target);

    println!(
        "Downloading {} binary for {}...",
        provider.name(),
        format!("{:?}", target).to_lowercase()
    );

    // Download the binary with progress
    let content = download_binary_with_progress(&download_url, provider.name()).await?;

    // Write and make executable
    write_and_make_executable(&binary_path, &content)?;

    println!(
        "{} binary installed successfully at: {}",
        provider.name(),
        binary_path.display()
    );

    // Verify the binary works by checking version
    println!("Verifying {} binary...", provider.name());
    match get_binary_version(provider, &bin_dir).await {
        Ok(version) => println!("{} version: {}", provider.name(), version),
        Err(e) => {
            eprintln!(
                "Warning: Could not verify {} version: {}",
                provider.name(),
                e
            );
        }
    }

    Ok(binary_path)
}

/// Run a binary with given arguments and return the output
pub async fn run_binary<P: AsRef<Path>>(
    binary_path: P,
    args: &[&str],
    binary_name: &str,
) -> BinResult<std::process::Output> {
    let binary_path = binary_path.as_ref();

    if !binary_path.exists() {
        return Err(format!(
            "{} binary does not exist at: {}",
            binary_name,
            binary_path.display()
        )
        .into());
    }

    if !is_executable(binary_path)? {
        return Err(format!(
            "{} binary is not executable: {}",
            binary_name,
            binary_path.display()
        )
        .into());
    }

    let output = std::process::Command::new(binary_path)
        .args(args)
        .output()?;

    Ok(output)
}

/// Run a binary using provider information
pub async fn run_binary_with_provider<P: AsRef<Path>>(
    provider: &dyn BinaryInfoProvider,
    bin_dir: P,
    args: &[&str],
) -> BinResult<std::process::Output> {
    let binary_path = get_provider_binary_path(provider, &bin_dir);

    if !is_binary_ready(provider, &bin_dir) {
        return Err(format!(
            "{} binary is not installed or not executable",
            provider.name()
        )
        .into());
    }

    run_binary(&binary_path, args, provider.name()).await
}

/// Get the version of a binary using provider information
pub async fn get_binary_version<P: AsRef<Path>>(
    provider: &dyn BinaryInfoProvider,
    bin_dir: P,
) -> BinResult<String> {
    let output = run_binary_with_provider(provider, &bin_dir, provider.version_args()).await?;

    if output.status.success() {
        let output_text = String::from_utf8_lossy(&output.stdout);

        if let Some(version) = provider.parse_version_output(&output_text) {
            Ok(version)
        } else {
            Ok(format!("{} (version unknown)", provider.name()))
        }
    } else {
        Err(format!("Could not determine {} version", provider.name()).into())
    }
}

// Public API functions

/// Get status of all binary providers
pub fn get_all_status<P: AsRef<Path>>(bin_dir: P) -> Vec<BinaryInfo> {
    registry().get_all_status(bin_dir)
}

/// Ensures all required binaries are installed
pub async fn ensure_required_binaries<P: AsRef<Path>>(bin_dir: P) -> BinResult<Vec<PathBuf>> {
    registry().ensure_all_binaries(bin_dir).await
}

/// Returns status information for all managed binaries
pub fn get_binaries_status<P: AsRef<Path>>(bin_dir: P) -> Vec<BinaryInfo> {
    get_all_status(bin_dir)
}

/// Get version of a specific binary by name
pub async fn get_binary_version_by_name<P: AsRef<Path>>(
    name: &str,
    bin_dir: P,
) -> BinResult<String> {
    if let Some(provider) = registry().get_provider(name) {
        get_binary_version(provider, bin_dir).await
    } else {
        Err(format!("Unknown binary provider: {}", name).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct TestProvider;

    impl BinaryInfoProvider for TestProvider {
        fn name(&self) -> &'static str {
            "test-binary"
        }

        fn local_name(&self) -> &'static str {
            "testbin"
        }

        fn get_download_url(&self, _target: &SystemTarget) -> String {
            "https://example.com/testbin".to_string()
        }

        fn version_args(&self) -> &[&str] {
            &["--version"]
        }

        fn parse_version_output(&self, output: &str) -> Option<String> {
            if output.contains("test-binary") {
                Some(output.trim().to_string())
            } else {
                None
            }
        }
    }

    #[test]
    fn test_system_target_detection() {
        let target = SystemTarget::detect();
        assert!(target.is_ok());
    }

    #[test]
    fn test_binary_info_creation() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test_binary");

        let info = BinaryInfo::from_path("test".to_string(), path);
        assert_eq!(info.name, "test");
        assert!(!info.exists);
        assert!(!info.executable);
        assert!(!info.is_ready());
    }

    #[test]
    fn test_get_binary_path() {
        let temp_dir = TempDir::new().unwrap();
        let bin_path = get_binary_path(temp_dir.path(), "s3fs");
        assert!(bin_path.to_string_lossy().ends_with("s3fs"));
    }

    #[test]
    fn test_is_executable_nonexistent() {
        let result = is_executable("/nonexistent/path");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_provider_path_generation() {
        let temp_dir = TempDir::new().unwrap();
        let provider = TestProvider;

        let path = get_provider_binary_path(&provider, temp_dir.path());
        assert!(path.to_string_lossy().ends_with("testbin"));
    }

    #[test]
    fn test_binary_info_provider() {
        let temp_dir = TempDir::new().unwrap();
        let provider = TestProvider;

        let info = get_binary_info(&provider, temp_dir.path());
        assert_eq!(info.name, "test-binary");
        assert!(!info.exists);
        assert!(!info.executable);
        assert!(!info.is_ready());
    }

    #[test]
    fn test_registry_creation() {
        let registry = ProviderRegistry::new();
        assert_eq!(
            registry
                .get_all_status(tempfile::TempDir::new().unwrap().path())
                .len(),
            3
        );
    }

    #[test]
    fn test_get_all_status() {
        let temp_dir = TempDir::new().unwrap();
        let bin_dir = temp_dir.path();

        let statuses = get_all_status(bin_dir);
        assert_eq!(statuses.len(), 3);

        let names: Vec<&String> = statuses.iter().map(|s| &s.name).collect();
        assert!(names.contains(&&"s3fs".to_string()));
        assert!(names.contains(&&"ClickHouse".to_string()));
        assert!(names.contains(&&"agt".to_string()));
    }

    #[test]
    fn test_binary_info_direct() {
        let temp_dir = TempDir::new().unwrap();
        let bin_dir = temp_dir.path();

        // Test direct provider access
        let provider = registry().get_provider("s3fs").unwrap();
        let info = get_binary_info(provider, bin_dir);
        assert_eq!(info.name, "s3fs");
        assert!(info.path.to_string_lossy().ends_with("s3fs"));
        assert!(!info.exists);
        assert!(!info.executable);

        let unknown_provider = registry().get_provider("unknown");
        assert!(unknown_provider.is_none());
    }

    #[tokio::test]
    async fn test_binary_version_by_name() {
        let temp_dir = TempDir::new().unwrap();
        let bin_dir = temp_dir.path();

        // Should fail for non-existent binary
        let result = get_binary_version_by_name("s3fs", bin_dir).await;
        assert!(result.is_err());

        // Should fail for unknown binary
        let result = get_binary_version_by_name("unknown", bin_dir).await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unknown binary provider")
        );
    }
}
