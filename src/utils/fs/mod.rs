pub mod archive;
pub mod filesystem;

// Re-export commonly used filesystem functions
#[allow(unused_imports)]
pub use archive::{extract_zip, extract_zip_with_root_stripping};
#[allow(unused_imports)]
pub use filesystem::{
    create_agnostic_working_dir, ensure_dir_exists, file_size, get_current_working_dir,
    is_directory, is_file, remove_path, temp_file_path,
};
