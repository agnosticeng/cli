//! Binary management utilities
//!
//! This module handles downloading and managing external binary dependencies
//! such as s3fs, ClickHouse, and agt using a consolidated management system.
//!
//! Each binary has its own folder with its provider implementation:
//! - `s3fs/`: S3FS binary provider
//! - `clickhouse/`: ClickHouse binary provider
//! - `agt/`: AGT binary provider
//!
//! All functionality is consolidated in:
//! - `manager`: Complete binary management system with types, providers, and operations

// Binary provider modules
pub mod agt;
pub mod clickhouse;
pub mod s3fs;

// Consolidated management module
pub mod manager;

// Re-export commonly used types and functions
pub use manager::{BinResult, BinaryInfo, SystemTarget};

// Re-export provider system and management functions
pub use manager::{
    agt, clickhouse, ensure_required_binaries, get_binaries_status, get_binary_path,
    get_binary_version_by_name, registry, s3fs,
};

// Re-export core utilities
pub use manager::BinaryInfoProvider;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_system_target_detection() {
        let target = SystemTarget::detect();
        assert!(target.is_ok());
    }

    #[test]
    fn test_get_binary_path() {
        let temp_dir = TempDir::new().unwrap();
        let bin_path = get_binary_path(temp_dir.path(), "s3fs");
        assert!(bin_path.to_string_lossy().ends_with("s3fs"));
    }

    #[test]
    fn test_get_binaries_status() {
        let temp_dir = TempDir::new().unwrap();
        let bin_dir = temp_dir.path();

        let binaries = get_binaries_status(bin_dir);
        assert_eq!(binaries.len(), 3);

        let names: Vec<&String> = binaries.iter().map(|b| &b.name).collect();
        assert!(names.contains(&&"s3fs".to_string()));
        assert!(names.contains(&&"ClickHouse".to_string()));
        assert!(names.contains(&&"agt".to_string()));
    }
}
