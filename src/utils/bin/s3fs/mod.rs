//! S3FS binary provider
//!
//! This module provides configuration and information for the S3FS binary.
//! All actual operations (install, run, check) are handled by the common manager.

use crate::utils::bin::manager::{BinaryInfoProvider, SystemTarget};

/// S3FS binary information provider
#[derive(Debug)]
pub struct S3fsProvider;

impl S3fsProvider {
    /// Create a new S3FS provider instance
    pub fn new() -> Self {
        Self
    }
}

impl BinaryInfoProvider for S3fsProvider {
    fn name(&self) -> &'static str {
        "s3fs"
    }

    fn local_name(&self) -> &'static str {
        "s3fs"
    }

    fn get_download_url(&self, target: &SystemTarget) -> String {
        let asset_name = match target {
            SystemTarget::MacOsAarch64 => "s3fs_aarch64-apple-darwin",
            SystemTarget::MacOsX86_64 => "s3fs_x86_64-apple-darwin",
            SystemTarget::LinuxX86_64 => "s3fs_x86_64-unknown-linux-gnu",
        };

        format!(
            "https://github.com/agnosticeng/s3fs/releases/download/v0.0.1/{}",
            asset_name
        )
    }

    fn version_args(&self) -> &[&str] {
        &["--help"] // s3fs doesn't have --version
    }

    fn parse_version_output(&self, _output: &str) -> Option<String> {
        // s3fs doesn't provide version info, so we return a static version
        Some("v0.0.1 (from agnosticeng/s3fs)".to_string())
    }
}

/// Create a new S3FS provider instance
pub fn provider() -> S3fsProvider {
    S3fsProvider::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3fs_provider_info() {
        let provider = S3fsProvider::new();
        assert_eq!(provider.name(), "s3fs");
        assert_eq!(provider.local_name(), "s3fs");
        assert_eq!(provider.version_args(), &["--help"]);

        let target = SystemTarget::MacOsAarch64;
        let url = provider.get_download_url(&target);
        assert!(url.contains("s3fs_aarch64-apple-darwin"));
        assert!(url.contains("github.com/agnosticeng/s3fs"));
    }

    #[test]
    fn test_s3fs_version_parsing() {
        let provider = S3fsProvider::new();
        let version = provider.parse_version_output("some help text");
        assert_eq!(version, Some("v0.0.1 (from agnosticeng/s3fs)".to_string()));
    }

    #[test]
    fn test_s3fs_download_urls() {
        let provider = S3fsProvider::new();

        let macos_arm_url = provider.get_download_url(&SystemTarget::MacOsAarch64);
        assert!(macos_arm_url.contains("s3fs_aarch64-apple-darwin"));

        let macos_x86_url = provider.get_download_url(&SystemTarget::MacOsX86_64);
        assert!(macos_x86_url.contains("s3fs_x86_64-apple-darwin"));

        let linux_url = provider.get_download_url(&SystemTarget::LinuxX86_64);
        assert!(linux_url.contains("s3fs_x86_64-unknown-linux-gnu"));
    }
}
