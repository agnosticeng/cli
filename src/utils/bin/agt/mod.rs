//! AGT binary provider
//!
//! This module provides configuration and information for the AGT binary.
//! All actual operations (install, run, check) are handled by the common manager.

use crate::utils::bin::manager::{BinaryInfoProvider, SystemTarget};

/// AGT binary information provider
#[derive(Debug)]
pub struct AgtProvider;

impl AgtProvider {
    /// Create a new AGT provider instance
    pub fn new() -> Self {
        Self
    }
}

impl BinaryInfoProvider for AgtProvider {
    fn name(&self) -> &'static str {
        "agt"
    }

    fn local_name(&self) -> &'static str {
        "agt"
    }

    fn get_download_url(&self, target: &SystemTarget) -> String {
        let asset_name = match target {
            SystemTarget::MacOsAarch64 => "agt_0.0.22_darwin_arm64",
            SystemTarget::MacOsX86_64 => "agt_0.0.22_darwin_amd64_v1",
            SystemTarget::LinuxX86_64 => "agt_0.0.22_linux_amd64_v1",
        };

        format!(
            "https://github.com/agnosticeng/agt/releases/download/v0.0.22/{}",
            asset_name
        )
    }

    fn version_args(&self) -> &[&str] {
        &["--version"]
    }

    fn parse_version_output(&self, output: &str) -> Option<String> {
        // Extract version from output like "agt v0.0.22"
        output.lines().next().map(|line| line.trim().to_string())
    }
}

/// Create a new AGT provider instance
pub fn provider() -> AgtProvider {
    AgtProvider::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agt_provider_info() {
        let provider = AgtProvider::new();
        assert_eq!(provider.name(), "agt");
        assert_eq!(provider.local_name(), "agt");
        assert_eq!(provider.version_args(), &["--version"]);
    }

    #[test]
    fn test_agt_version_parsing() {
        let provider = AgtProvider::new();

        let output = "agt v0.0.22";
        let version = provider.parse_version_output(output);
        assert_eq!(version, Some("agt v0.0.22".to_string()));

        let output_with_extra = "agt v0.0.22\nSome extra info";
        let version = provider.parse_version_output(output_with_extra);
        assert_eq!(version, Some("agt v0.0.22".to_string()));

        let empty_output = "";
        let version = provider.parse_version_output(empty_output);
        assert_eq!(version, None);
    }

    #[test]
    fn test_agt_download_urls() {
        let provider = AgtProvider::new();

        let macos_arm_url = provider.get_download_url(&SystemTarget::MacOsAarch64);
        assert!(macos_arm_url.contains("agt_0.0.22_darwin_arm64"));
        assert!(macos_arm_url.contains("github.com/agnosticeng/agt"));

        let macos_x86_url = provider.get_download_url(&SystemTarget::MacOsX86_64);
        assert!(macos_x86_url.contains("agt_0.0.22_darwin_amd64_v1"));

        let linux_url = provider.get_download_url(&SystemTarget::LinuxX86_64);
        assert!(linux_url.contains("agt_0.0.22_linux_amd64_v1"));

        // All should contain the release URL pattern
        for url in [&macos_arm_url, &macos_x86_url, &linux_url] {
            assert!(url.contains("releases/download/v0.0.22"));
        }
    }
}
