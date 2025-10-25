//! Integration tests for CLI binary management utilities
//!
//! These tests verify utility functions work correctly without
//! performing actual binary downloads.

use cli::utils::get_binaries_status;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_binary_path_generation() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path();

    let binaries = get_binaries_status(bin_dir);
    let s3fs_info = binaries.iter().find(|b| b.name == "s3fs").unwrap();
    assert!(s3fs_info.path.to_string_lossy().ends_with("s3fs"));
}

#[test]
fn test_s3fs_ready_check_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path().join("nonexistent");

    let binaries = get_binaries_status(&bin_dir);
    let s3fs_info = binaries.iter().find(|b| b.name == "s3fs").unwrap();
    assert!(!s3fs_info.is_ready());
}

#[test]
fn test_binaries_status_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path().join("nonexistent");

    let binaries = get_binaries_status(&bin_dir);
    assert_eq!(binaries.len(), 3); // All binaries should be reported even if not installed

    // Check s3fs binary status
    let s3fs_binary = binaries.iter().find(|b| b.name == "s3fs").unwrap();
    assert!(!s3fs_binary.is_ready());
    assert!(!s3fs_binary.exists);
    assert!(!s3fs_binary.executable);
    assert!(s3fs_binary.size.is_none());

    // Check ClickHouse binary status
    let clickhouse_binary = binaries.iter().find(|b| b.name == "ClickHouse").unwrap();
    assert!(!clickhouse_binary.is_ready());
    assert!(!clickhouse_binary.exists);
    assert!(!clickhouse_binary.executable);
    assert!(clickhouse_binary.size.is_none());

    // Check agt binary status
    let agt_binary = binaries.iter().find(|b| b.name == "agt").unwrap();
    assert!(!agt_binary.is_ready());
    assert!(!agt_binary.exists);
    assert!(!agt_binary.executable);
    assert!(agt_binary.size.is_none());
}

#[test]
fn test_clickhouse_path_generation() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path();

    let binaries = get_binaries_status(bin_dir);
    let clickhouse_info = binaries.iter().find(|b| b.name == "ClickHouse").unwrap();
    assert!(
        clickhouse_info
            .path
            .to_string_lossy()
            .ends_with("clickhouse")
    );
}

#[test]
fn test_clickhouse_ready_check_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path().join("nonexistent");

    let binaries = get_binaries_status(&bin_dir);
    let clickhouse_info = binaries.iter().find(|b| b.name == "ClickHouse").unwrap();
    assert!(!clickhouse_info.is_ready());
}

#[test]
fn test_agt_path_generation() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path();

    let binaries = get_binaries_status(bin_dir);
    let agt_info = binaries.iter().find(|b| b.name == "agt").unwrap();
    assert!(agt_info.path.to_string_lossy().ends_with("agt"));
}

#[test]
fn test_agt_ready_check_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path().join("nonexistent");

    let binaries = get_binaries_status(&bin_dir);
    let agt_info = binaries.iter().find(|b| b.name == "agt").unwrap();
    assert!(!agt_info.is_ready());
}

#[test]
fn test_binaries_status_with_fake_binaries() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();

    // Get binary paths using binaries status
    let binaries = get_binaries_status(&bin_dir);
    let s3fs_info = binaries.iter().find(|b| b.name == "s3fs").unwrap();
    let clickhouse_info = binaries.iter().find(|b| b.name == "ClickHouse").unwrap();
    let agt_info = binaries.iter().find(|b| b.name == "agt").unwrap();

    let s3fs_path = &s3fs_info.path;
    let clickhouse_path = &clickhouse_info.path;
    let agt_path = &agt_info.path;

    // Create fake binary files
    fs::write(s3fs_path, "fake s3fs binary").unwrap();
    fs::write(clickhouse_path, "fake clickhouse binary").unwrap();
    fs::write(agt_path, "fake agt binary").unwrap();

    // Make them executable on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(s3fs_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(s3fs_path, perms).unwrap();

        let mut perms = fs::metadata(clickhouse_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(clickhouse_path, perms).unwrap();

        let mut perms = fs::metadata(agt_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(agt_path, perms).unwrap();
    }

    // Test binaries status with the new files
    let updated_binaries = get_binaries_status(&bin_dir);
    assert_eq!(updated_binaries.len(), 3);

    // Find s3fs binary
    let s3fs_status = updated_binaries.iter().find(|b| b.name == "s3fs").unwrap();
    assert!(s3fs_status.exists);
    assert!(s3fs_status.size.is_some());
    assert!(s3fs_status.size.unwrap() > 0);

    // Find ClickHouse binary
    let clickhouse_status = updated_binaries
        .iter()
        .find(|b| b.name == "ClickHouse")
        .unwrap();
    assert!(clickhouse_status.exists);
    assert!(clickhouse_status.size.is_some());
    assert!(clickhouse_status.size.unwrap() > 0);

    // Find agt binary
    let agt_status = updated_binaries.iter().find(|b| b.name == "agt").unwrap();
    assert!(agt_status.exists);
    assert!(agt_status.size.is_some());
    assert!(agt_status.size.unwrap() > 0);

    // Check readiness - should be ready on Unix systems where we set executable permissions
    #[cfg(unix)]
    {
        assert!(s3fs_status.is_ready());
        assert!(clickhouse_status.is_ready());
        assert!(agt_status.is_ready());
    }
}

#[test]
fn test_unknown_binary_handling() {
    let temp_dir = TempDir::new().unwrap();
    let bin_dir = temp_dir.path();

    // Test that get_binaries_status only returns known binaries
    let binaries = get_binaries_status(bin_dir);
    assert_eq!(binaries.len(), 3);

    let binary_names: Vec<&str> = binaries.iter().map(|b| b.name.as_str()).collect();
    assert!(binary_names.contains(&"s3fs"));
    assert!(binary_names.contains(&"ClickHouse"));
    assert!(binary_names.contains(&"agt"));
    assert!(!binary_names.contains(&"unknown"));
}
