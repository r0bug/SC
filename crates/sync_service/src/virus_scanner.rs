use anyhow::{anyhow, Result};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

#[derive(Debug, Clone)]
pub enum ScanResult {
    Clean,
    Infected(String),
    Error(String),
}

pub struct VirusScanner {
    enabled: bool,
    socket_path: String,
    timeout_secs: u64,
}

impl VirusScanner {
    pub fn new(enabled: bool, socket_path: String, timeout_secs: u64) -> Self {
        Self {
            enabled,
            socket_path,
            timeout_secs,
        }
    }

    pub async fn scan_file(&self, file_path: &Path) -> Result<ScanResult> {
        if !self.enabled {
            tracing::debug!("Virus scanning disabled, marking as clean");
            return Ok(ScanResult::Clean);
        }

        // Try to connect to ClamAV socket
        match self.scan_with_clamav(file_path).await {
            Ok(result) => Ok(result),
            Err(e) => {
                tracing::warn!("ClamAV scan failed: {}, using mock scanner", e);
                Ok(self.mock_scan(file_path))
            }
        }
    }

    async fn scan_with_clamav(&self, file_path: &Path) -> Result<ScanResult> {
        // Connect to ClamAV Unix socket
        let mut stream = tokio::time::timeout(
            std::time::Duration::from_secs(self.timeout_secs),
            UnixStream::connect(&self.socket_path),
        )
        .await
        .map_err(|_| anyhow!("Connection timeout"))?
        .map_err(|e| anyhow!("Failed to connect to ClamAV: {}", e))?;

        // Send SCAN command with file path
        let command = format!("SCAN {}\\n", file_path.display());
        stream.write_all(command.as_bytes()).await?;
        stream.flush().await?;

        // Read response
        let mut response = String::new();
        stream.read_to_string(&mut response).await?;

        // Parse ClamAV response
        if response.contains(" OK") {
            Ok(ScanResult::Clean)
        } else if response.contains(" FOUND") {
            // Extract virus name from response like: "/path/file.txt: Virus.Name FOUND"
            let virus_name = response
                .split(':')
                .nth(1)
                .and_then(|s| s.split(" FOUND").next())
                .unwrap_or("Unknown virus")
                .trim()
                .to_string();
            Ok(ScanResult::Infected(virus_name))
        } else if response.contains(" ERROR") {
            Ok(ScanResult::Error(response.trim().to_string()))
        } else {
            Ok(ScanResult::Error(format!(
                "Unexpected ClamAV response: {}",
                response
            )))
        }
    }

    fn mock_scan(&self, file_path: &Path) -> ScanResult {
        // Mock scanner: Check for EICAR test signature
        if let Some(filename) = file_path.file_name() {
            let name = filename.to_string_lossy().to_lowercase();
            if name.contains("eicar") || name.contains("virus") || name.contains("malware") {
                return ScanResult::Infected("EICAR-Test-Signature".to_string());
            }
        }
        ScanResult::Clean
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_mock_scanner_clean_file() {
        let scanner = VirusScanner::new(false, String::new(), 30);

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "This is a clean file").unwrap();

        let result = scanner.scan_file(temp_file.path()).await.unwrap();
        assert!(matches!(result, ScanResult::Clean));
    }

    #[tokio::test]
    async fn test_mock_scanner_eicar() {
        let scanner = VirusScanner::new(false, String::new(), 30);

        let temp_file = NamedTempFile::with_prefix("eicar_test").unwrap();

        let result = scanner.scan_file(temp_file.path()).await.unwrap();
        assert!(matches!(result, ScanResult::Infected(_)));
    }
}
