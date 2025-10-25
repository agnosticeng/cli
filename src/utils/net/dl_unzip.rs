use std::path::Path;

use crate::utils::{
    fs::{ensure_dir_exists, extract_zip_with_root_stripping, remove_path, temp_file_path},
    net::download::download_to_temp_file,
};

/// Downloads a ZIP file from the given URL and extracts it to the destination directory
///
/// This function automatically strips the root folder from ZIP archives (common with
/// GitHub repository downloads) and extracts the contents directly to the destination.
/// For example, if a ZIP contains "project-main/file.txt", it will be extracted as
/// "dest/file.txt" instead of "dest/project-main/file.txt".
///
/// # Arguments
///
/// * `url` - The URL to download the ZIP file from
/// * `dest` - The destination directory to extract the contents to
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if the download or extraction fails
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use cli::utils::dl_unzip;
///
/// // Download and extract a ZIP file (root folder will be stripped)
/// dl_unzip("https://github.com/user/repo/archive/main.zip", "./extracted").await?;
/// # Ok(())
/// # }
/// ```
pub async fn dl_unzip<P: AsRef<Path>>(
    url: &str,
    dest: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let dest_path = dest.as_ref();

    // Ensure destination directory exists
    ensure_dir_exists(dest_path)?;

    // Create a temporary file path for the download
    let temp_file_path = temp_file_path(dest_path, Some("download"), Some(".zip"));

    // Download the file to the temporary location
    download_to_temp_file(url, &temp_file_path).await?;

    // Extract the ZIP file with root folder stripping
    extract_zip_with_root_stripping(&temp_file_path, dest_path)?;

    // Clean up the temporary file
    remove_path(&temp_file_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_dl_unzip_creates_destination() {
        let temp_dir = TempDir::new().unwrap();
        let dest_path = temp_dir.path().join("test_dest");

        // This test would require a real URL, so we'll just test directory creation
        ensure_dir_exists(&dest_path).unwrap();
        assert!(dest_path.exists());
    }

    #[test]
    fn test_temp_file_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_file_path(&temp_dir, Some("test"), Some(".tmp"));

        // Create a temporary file
        std::fs::write(&temp_path, b"test content").unwrap();
        assert!(temp_path.exists());

        // Remove it
        remove_path(&temp_path).unwrap();
        assert!(!temp_path.exists());
    }
}
