use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Creates a directory and all necessary parent directories
///
/// # Arguments
///
/// * `path` - The directory path to create
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if the creation fails
///
/// # Examples
///
/// ```no_run
/// use cli::utils::fs::filesystem::ensure_dir_exists;
///
/// ensure_dir_exists("./some/nested/directory").unwrap();
/// ```
pub fn ensure_dir_exists<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&path)?;
    Ok(())
}

/// Removes a file or directory and all its contents
///
/// # Arguments
///
/// * `path` - The path to remove
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if the removal fails
pub fn remove_path<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn std::error::Error>> {
    let path = path.as_ref();

    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else if path.exists() {
        fs::remove_file(path)?;
    }

    Ok(())
}

/// Checks if a path exists and is a directory
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// Returns `true` if the path exists and is a directory, `false` otherwise
#[allow(dead_code)]
pub fn is_directory<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_dir()
}

/// Checks if a path exists and is a file
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// Returns `true` if the path exists and is a file, `false` otherwise
#[allow(dead_code)]
pub fn is_file<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

/// Gets the size of a file in bytes
///
/// # Arguments
///
/// * `path` - The path to the file
///
/// # Returns
///
/// Returns the file size in bytes, or an error if the file cannot be accessed
#[allow(dead_code)]
pub fn file_size<P: AsRef<Path>>(path: P) -> Result<u64, Box<dyn std::error::Error>> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

/// Creates a temporary file path in the given directory
///
/// # Arguments
///
/// * `dir` - The directory to create the temporary file in
/// * `prefix` - Optional prefix for the temporary file name
/// * `suffix` - Optional suffix for the temporary file name
///
/// # Returns
///
/// Returns a path to a temporary file that doesn't exist yet
pub fn temp_file_path<P: AsRef<Path>>(
    dir: P,
    prefix: Option<&str>,
    suffix: Option<&str>,
) -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let prefix = prefix.unwrap_or("temp");
    let suffix = suffix.unwrap_or("");

    let filename = format!("{}_{}{}", prefix, timestamp, suffix);
    dir.as_ref().join(filename)
}

/// Gets the current working directory
///
/// # Returns
///
/// Returns the current working directory as a `PathBuf`, or an error if it cannot be determined
///
/// # Examples
///
/// ```no_run
/// use cli::utils::fs::filesystem::get_current_working_dir;
///
/// match get_current_working_dir() {
///     Ok(cwd) => println!("Current directory: {}", cwd.display()),
///     Err(e) => eprintln!("Failed to get current directory: {}", e),
/// }
/// ```
#[allow(dead_code)]
pub fn get_current_working_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let cwd = env::current_dir()?;
    Ok(cwd)
}

/// Creates and returns the agnostic working directory path (~/.agnostic/)
///
/// This function creates the directory `~/.agnostic/` if it doesn't exist and returns its path.
/// The .agnostic directory serves as a hidden standard working directory for the CLI tool.
///
/// # Returns
///
/// Returns the path to `~/.agnostic/` as a `PathBuf`, or an error if the directory cannot be created
///
/// # Examples
///
/// ```no_run
/// use cli::utils::fs::filesystem::create_agnostic_working_dir;
///
/// match create_agnostic_working_dir() {
///     Ok(agnostic_dir) => {
///         println!("Agnostic working directory: {}", agnostic_dir.display());
///         // Use the directory for your operations
///     },
///     Err(e) => eprintln!("Failed to create agnostic directory: {}", e),
/// }
/// ```
pub fn create_agnostic_working_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let home_dir = env::var("HOME").map_err(|_| "Could not determine home directory")?;
    let agnostic_dir = PathBuf::from(home_dir).join(".agnostic");

    ensure_dir_exists(&agnostic_dir)?;
    Ok(agnostic_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_dir_exists() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test").join("nested").join("dir");

        assert!(!test_path.exists());
        ensure_dir_exists(&test_path).unwrap();
        assert!(test_path.exists());
        assert!(is_directory(&test_path));
    }

    #[test]
    fn test_remove_path_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        // Create a test file
        File::create(&file_path).unwrap();
        assert!(file_path.exists());

        // Remove it
        remove_path(&file_path).unwrap();
        assert!(!file_path.exists());
    }

    #[test]
    fn test_remove_path_directory() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("test_dir");

        // Create a test directory with a file
        fs::create_dir_all(&dir_path).unwrap();
        File::create(dir_path.join("file.txt")).unwrap();
        assert!(dir_path.exists());

        // Remove it
        remove_path(&dir_path).unwrap();
        assert!(!dir_path.exists());
    }

    #[test]
    fn test_is_directory_and_is_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");
        let dir_path = temp_dir.path().join("test_dir");

        File::create(&file_path).unwrap();
        fs::create_dir_all(&dir_path).unwrap();

        assert!(is_file(&file_path));
        assert!(!is_directory(&file_path));
        assert!(is_directory(&dir_path));
        assert!(!is_file(&dir_path));
    }

    #[test]
    fn test_file_size() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        let content = b"Hello, world!";
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content).unwrap();

        let size = file_size(&file_path).unwrap();
        assert_eq!(size, content.len() as u64);
    }

    #[test]
    fn test_temp_file_path() {
        let temp_dir = TempDir::new().unwrap();

        let temp_path1 = temp_file_path(&temp_dir, None, None);
        let temp_path2 = temp_file_path(&temp_dir, Some("custom"), Some(".tmp"));

        assert!(temp_path1.parent().unwrap() == temp_dir.path());
        assert!(temp_path2.parent().unwrap() == temp_dir.path());
        assert!(temp_path1 != temp_path2);

        let filename2 = temp_path2.file_name().unwrap().to_string_lossy();
        assert!(filename2.starts_with("custom_"));
        assert!(filename2.ends_with(".tmp"));
    }

    #[test]
    fn test_get_current_working_dir() {
        let cwd = get_current_working_dir().unwrap();
        assert!(cwd.is_absolute());
        assert!(cwd.exists());
        assert!(is_directory(&cwd));
    }

    #[test]
    fn test_create_agnostic_working_dir() {
        // This test actually creates the ~/.agnostic directory
        // Clean up afterwards if needed
        let agnostic_dir = create_agnostic_working_dir().unwrap();

        // Verify it's in the expected location
        assert!(agnostic_dir.is_absolute());
        assert!(agnostic_dir.file_name().unwrap() == ".agnostic");

        // Verify the directory exists and is a directory
        assert!(agnostic_dir.exists());
        assert!(is_directory(&agnostic_dir));

        // Test that calling it again works (idempotent)
        let agnostic_dir2 = create_agnostic_working_dir().unwrap();
        assert_eq!(agnostic_dir, agnostic_dir2);
    }
}
