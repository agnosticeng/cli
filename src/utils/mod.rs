//! Utilities module for the CLI application
//!
//! This module is organized into several sub-modules for better code organization:
//! - `fs`: Filesystem operations (files, directories, archives)
//! - `net`: Network operations (downloads, remote operations)
//! - `app`: Application lifecycle (initialization, configuration, cleanup)
//! - `bin`: Binary management (downloading and managing external tools)

pub mod app;
pub mod bin;
pub mod fs;
pub mod net;

// Re-export commonly used functions for convenience
// Filesystem utilities
#[allow(unused_imports)]
pub use fs::{
    create_agnostic_working_dir, ensure_dir_exists, extract_zip, extract_zip_with_root_stripping,
    file_size, get_current_working_dir, is_directory, is_file, remove_path, temp_file_path,
};

// Network utilities
#[allow(unused_imports)]
pub use net::{dl_unzip, download_file, download_to_temp_file};

// Application utilities
#[allow(unused_imports)]
pub use app::{AppConfig, cleanup_app, get_agnostic_subdir, initialize_app};

// Binary utilities
#[allow(unused_imports)]
pub use bin::{
    BinResult, BinaryInfo, BinaryInfoProvider, SystemTarget, agt, clickhouse,
    ensure_required_binaries, get_binaries_status, get_binary_path, get_binary_version_by_name,
    registry, s3fs,
};
