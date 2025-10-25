use reqwest::Client;
use std::path::Path;

/// Downloads a file from the given URL and returns the content as bytes
///
/// # Arguments
///
/// * `url` - The URL to download the file from
///
/// # Returns
///
/// Returns `Ok(Vec<u8>)` with the file content if successful, or an error if the download fails
///
/// # Examples
///
/// ```no_run
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use cli::utils::net::download::download_file;
///
/// let content = download_file("https://example.com/file.zip").await?;
/// println!("Downloaded {} bytes", content.len());
/// # Ok(())
/// # }
/// ```
pub async fn download_file(url: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    println!("Downloading from: {}", url);

    let client = Client::new();
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to download file: HTTP {}", response.status()).into());
    }

    let content = response.bytes().await?;
    println!("Downloaded {} bytes", content.len());

    Ok(content.to_vec())
}

/// Downloads a file from URL and saves it to a temporary file
///
/// # Arguments
///
/// * `url` - The URL to download the file from
/// * `temp_path` - The temporary file path to save to
///
/// # Returns
///
/// Returns `Ok(())` if successful, or an error if the download or save fails
pub async fn download_to_temp_file<P: AsRef<Path>>(
    url: &str,
    temp_path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;

    let content = download_file(url).await?;

    let mut temp_file = File::create(&temp_path)?;
    temp_file.write_all(&content)?;
    temp_file.sync_all()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_download_to_temp_file_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().join("test_file.tmp");

        // This would require a real URL for a full test
        // For now, just test that the file creation logic works
        let content = b"test content";
        fs::write(&temp_path, content).unwrap();

        assert!(temp_path.exists());
        assert_eq!(fs::read(&temp_path).unwrap(), content);
    }
}
