use std::fs;

use clap::Subcommand;

use crate::utils::{AppConfig, get_binaries_status, get_binary_version_by_name};

/// System-related subcommands
#[derive(Subcommand, Debug)]
pub enum SystemAction {
    /// Show overall system status
    Status,
}

impl SystemAction {
    pub async fn handle(self, config: &AppConfig) {
        match self {
            Self::Status => show_system_status(config).await,
        }
    }
}

/// Display overall system status
async fn show_system_status(config: &AppConfig) {
    println!("System Status");
    println!("=============");
    println!();

    // Working directory info
    println!("Working Directory");
    println!("   Location: {}", config.agnostic_dir.display());
    println!("   Exists: {}", config.agnostic_dir.exists());

    if let Ok(metadata) = fs::metadata(&config.agnostic_dir) {
        println!(
            "   Created: {}",
            format_system_time(metadata.created().ok())
        );
        println!(
            "   Modified: {}",
            format_system_time(metadata.modified().ok())
        );
    }
    println!();

    // Subdirectories
    println!("Subdirectories");
    let subdirs = ["bin", "user"];
    for subdir in subdirs {
        let path = config.agnostic_dir.join(subdir);
        let exists = path.exists();
        let status = if exists { "[EXISTS]" } else { "[MISSING]" };

        println!("   {} {} - {}", status, subdir, path.display());

        if exists && let Ok(entries) = fs::read_dir(&path) {
            let count = entries.count();
            println!("      Items: {}", count);
        }
    }
    println!();

    // Binary status summary
    println!("Binary Dependencies");
    show_binaries_status(config).await;

    // System information
    println!("System Information");
    println!("   OS: {}", std::env::consts::OS);
    println!("   Architecture: {}", std::env::consts::ARCH);
    println!("   Family: {}", std::env::consts::FAMILY);

    if let Ok(home) = std::env::var("HOME") {
        println!("   Home: {}", home);
    }

    if let Ok(user) = std::env::var("USER") {
        println!("   User: {}", user);
    }
}

/// Display the status of all managed binaries
async fn show_binaries_status(config: &AppConfig) {
    let bin_dir = config.agnostic_dir.join("bin");
    let binaries = get_binaries_status(&bin_dir);

    if binaries.is_empty() {
        println!("No managed binaries found.");
        return;
    }

    for binary in &binaries {
        let status_icon = if binary.is_ready() {
            "[READY]"
        } else {
            "[MISSING]"
        };
        let size_info = match binary.size {
            Some(size) => format_file_size(size),
            None => "N/A".to_string(),
        };

        println!("  {} {}", status_icon, binary.name);
        println!("    Path: {}", binary.path.display());
        println!("    Exists: {}", if binary.exists { "Yes" } else { "No" });
        println!(
            "    Executable: {}",
            if binary.executable { "Yes" } else { "No" }
        );
        println!("    Size: {}", size_info);

        // Show version info for ready binaries
        if binary.is_ready() {
            let bin_dir = &config.agnostic_dir.join("bin");
            match get_binary_version_by_name(&binary.name, bin_dir).await {
                Ok(version) => println!("    Version: {}", version),
                Err(_) => println!("    Version: Unknown"),
            }
        }

        println!();
    }

    // Summary
    let ready_count = binaries.iter().filter(|b| b.is_ready()).count();
    let total_count = binaries.len();

    if ready_count != total_count {
        println!(
            "Warning: {} of {} binaries are ready",
            ready_count, total_count
        );
    }
}

/// Format file size in human-readable format
fn format_file_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format system time for display
fn format_system_time(time: Option<std::time::SystemTime>) -> String {
    match time {
        Some(t) => {
            if let Ok(duration) = t.duration_since(std::time::UNIX_EPOCH) {
                let secs = duration.as_secs();
                let datetime = chrono::DateTime::from_timestamp(secs as i64, 0);
                match datetime {
                    Some(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    None => "Unknown".to_string(),
                }
            } else {
                "Unknown".to_string()
            }
        }
        None => "Unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(0), "0 B");
        assert_eq!(format_file_size(512), "512 B");
        assert_eq!(format_file_size(1024), "1.0 KB");
        assert_eq!(format_file_size(1536), "1.5 KB");
        assert_eq!(format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_file_size(6423168), "6.1 MB");
    }
}
