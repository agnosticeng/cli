//! ClickHouse binary provider
//!
//! This module provides configuration and information for the ClickHouse binary.
//! All actual operations (install, run, check) are handled by the common manager.

use crate::utils::bin::manager::{BinaryInfoProvider, SystemTarget};

/// ClickHouse binary information provider
#[derive(Debug)]
pub struct ClickhouseProvider;

impl ClickhouseProvider {
    /// Create a new ClickHouse provider instance
    pub fn new() -> Self {
        Self
    }
}

impl BinaryInfoProvider for ClickhouseProvider {
    fn name(&self) -> &'static str {
        "ClickHouse"
    }

    fn local_name(&self) -> &'static str {
        "clickhouse"
    }

    fn get_download_url(&self, target: &SystemTarget) -> String {
        match target {
            SystemTarget::MacOsAarch64 => {
                "https://builds.clickhouse.com/master/macos-aarch64/clickhouse".to_string()
            }
            SystemTarget::MacOsX86_64 => {
                "https://builds.clickhouse.com/master/macos/clickhouse".to_string()
            }
            SystemTarget::LinuxX86_64 => {
                "https://builds.clickhouse.com/master/amd64/clickhouse".to_string()
            }
        }
    }

    fn version_args(&self) -> &[&str] {
        &["--version"]
    }

    fn parse_version_output(&self, output: &str) -> Option<String> {
        // Extract version from output like "ClickHouse client version 23.8.1.1"
        output
            .lines()
            .find(|line| line.contains("ClickHouse"))
            .map(|line| line.trim().to_string())
    }
}

/// Create a new ClickHouse provider instance
pub fn provider() -> ClickhouseProvider {
    ClickhouseProvider::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clickhouse_provider_info() {
        let provider = ClickhouseProvider::new();
        assert_eq!(provider.name(), "ClickHouse");
        assert_eq!(provider.local_name(), "clickhouse");
        assert_eq!(provider.version_args(), &["--version"]);
    }

    #[test]
    fn test_clickhouse_version_parsing() {
        let provider = ClickhouseProvider::new();

        let output = "ClickHouse client version 23.8.1.1";
        let version = provider.parse_version_output(output);
        assert_eq!(
            version,
            Some("ClickHouse client version 23.8.1.1".to_string())
        );

        let output_with_extra = "Some other text\nClickHouse server version 24.1.0.0\nMore text";
        let version = provider.parse_version_output(output_with_extra);
        assert_eq!(
            version,
            Some("ClickHouse server version 24.1.0.0".to_string())
        );

        let output_no_version = "Some help text without version info";
        let version = provider.parse_version_output(output_no_version);
        assert_eq!(version, None);
    }

    #[test]
    fn test_clickhouse_download_urls() {
        let provider = ClickhouseProvider::new();

        let macos_arm_url = provider.get_download_url(&SystemTarget::MacOsAarch64);
        assert!(macos_arm_url.contains("macos-aarch64"));
        assert!(macos_arm_url.contains("builds.clickhouse.com"));

        let macos_x86_url = provider.get_download_url(&SystemTarget::MacOsX86_64);
        assert!(macos_x86_url.contains("macos"));
        assert!(!macos_x86_url.contains("aarch64"));

        let linux_url = provider.get_download_url(&SystemTarget::LinuxX86_64);
        assert!(linux_url.contains("amd64"));
        assert!(linux_url.contains("builds.clickhouse.com"));
    }
}
