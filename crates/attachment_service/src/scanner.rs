use async_trait::async_trait;
use std::path::Path;
use crate::error::AttachmentError;
use core_domain::ScanStatus;

/// Trait for virus scanning implementations
#[async_trait]
pub trait VirusScanner: Send + Sync {
    /// Scan a file for viruses
    async fn scan_file(&self, path: &Path) -> Result<ScanResult, AttachmentError>;
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub status: ScanStatus,
    pub details: Option<String>,
}

/// Mock scanner for development/testing
pub struct MockScanner {
    always_clean: bool,
}

impl MockScanner {
    pub fn new(always_clean: bool) -> Self {
        Self { always_clean }
    }
}

impl Default for MockScanner {
    fn default() -> Self {
        Self::new(true)
    }
}

#[async_trait]
impl VirusScanner for MockScanner {
    async fn scan_file(&self, path: &Path) -> Result<ScanResult, AttachmentError> {
        tracing::info!("Mock scanning file: {:?}", path);

        // Simulate scan delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        if self.always_clean {
            Ok(ScanResult {
                status: ScanStatus::Clean,
                details: None,
            })
        } else {
            // For testing: mark files with "virus" in name as infected
            if path.to_string_lossy().contains("virus") {
                Ok(ScanResult {
                    status: ScanStatus::Infected,
                    details: Some("Mock virus detected".to_string()),
                })
            } else {
                Ok(ScanResult {
                    status: ScanStatus::Clean,
                    details: None,
                })
            }
        }
    }
}

/// ClamAV scanner implementation using Unix socket or TCP connection
pub struct ClamAvScanner {
    /// Socket path (e.g., /var/run/clamav/clamd.ctl) or TCP address (e.g., 127.0.0.1:3310)
    connection: ClamAvConnection,
}

pub enum ClamAvConnection {
    UnixSocket(String),
    Tcp(String, u16),
}

impl ClamAvScanner {
    /// Create scanner with Unix socket connection (default for Linux)
    pub fn with_socket(socket_path: String) -> Self {
        Self {
            connection: ClamAvConnection::UnixSocket(socket_path),
        }
    }

    /// Create scanner with TCP connection
    pub fn with_tcp(host: String, port: u16) -> Self {
        Self {
            connection: ClamAvConnection::Tcp(host, port),
        }
    }

    /// Create from environment variables
    /// CLAMAV_SOCKET=/var/run/clamav/clamd.ctl or
    /// CLAMAV_HOST=127.0.0.1 CLAMAV_PORT=3310
    pub fn from_env() -> Option<Self> {
        if let Ok(socket) = std::env::var("CLAMAV_SOCKET") {
            Some(Self::with_socket(socket))
        } else if let Ok(host) = std::env::var("CLAMAV_HOST") {
            let port = std::env::var("CLAMAV_PORT")
                .unwrap_or_else(|_| "3310".to_string())
                .parse()
                .unwrap_or(3310);
            Some(Self::with_tcp(host, port))
        } else {
            None
        }
    }

    /// Legacy constructor for backward compatibility
    pub fn new(socket_path: String) -> Self {
        Self::with_socket(socket_path)
    }
}

#[async_trait]
impl VirusScanner for ClamAvScanner {
    async fn scan_file(&self, path: &Path) -> Result<ScanResult, AttachmentError> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        tracing::info!("ClamAV scanning file: {:?}", path);

        // Verify file exists
        if !path.exists() {
            return Err(AttachmentError::NotFound(format!(
                "File not found: {:?}",
                path
            )));
        }

        let path_str = path.to_string_lossy();

        // Connect to ClamAV daemon
        let response = match &self.connection {
            ClamAvConnection::UnixSocket(socket_path) => {
                #[cfg(unix)]
                {
                    use tokio::net::UnixStream;

                    let mut stream = UnixStream::connect(socket_path).await.map_err(|e| {
                        AttachmentError::ScanError(format!(
                            "Failed to connect to ClamAV socket {}: {}",
                            socket_path, e
                        ))
                    })?;

                    // Send SCAN command
                    let command = format!("SCAN {}\n", path_str);
                    stream.write_all(command.as_bytes()).await.map_err(|e| {
                        AttachmentError::ScanError(format!("Failed to send scan command: {}", e))
                    })?;

                    // Read response
                    let mut reader = BufReader::new(stream);
                    let mut response = String::new();
                    reader.read_line(&mut response).await.map_err(|e| {
                        AttachmentError::ScanError(format!("Failed to read scan response: {}", e))
                    })?;
                    response
                }
                #[cfg(not(unix))]
                {
                    return Err(AttachmentError::ScanError(
                        "Unix sockets not supported on this platform".to_string(),
                    ));
                }
            }
            ClamAvConnection::Tcp(host, port) => {
                use tokio::net::TcpStream;

                let addr = format!("{}:{}", host, port);
                let mut stream = TcpStream::connect(&addr).await.map_err(|e| {
                    AttachmentError::ScanError(format!(
                        "Failed to connect to ClamAV at {}: {}",
                        addr, e
                    ))
                })?;

                // Send SCAN command
                let command = format!("SCAN {}\n", path_str);
                stream.write_all(command.as_bytes()).await.map_err(|e| {
                    AttachmentError::ScanError(format!("Failed to send scan command: {}", e))
                })?;

                // Read response
                let mut reader = BufReader::new(stream);
                let mut response = String::new();
                reader.read_line(&mut response).await.map_err(|e| {
                    AttachmentError::ScanError(format!("Failed to read scan response: {}", e))
                })?;
                response
            }
        };

        tracing::debug!("ClamAV response: {}", response.trim());

        // Parse ClamAV response
        // Format: /path/to/file: OK or /path/to/file: VirusName FOUND
        let response = response.trim();
        if response.ends_with("OK") {
            tracing::info!("ClamAV: File is clean");
            Ok(ScanResult {
                status: ScanStatus::Clean,
                details: None,
            })
        } else if response.contains("FOUND") {
            // Extract virus name
            let virus_name = response
                .rsplit(':')
                .next()
                .map(|s| s.trim().replace(" FOUND", ""))
                .unwrap_or_else(|| "Unknown".to_string());

            tracing::warn!("ClamAV: Virus detected - {}", virus_name);
            Ok(ScanResult {
                status: ScanStatus::Infected,
                details: Some(virus_name),
            })
        } else if response.contains("ERROR") {
            let error_msg = response
                .rsplit(':')
                .next()
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "Unknown error".to_string());

            tracing::error!("ClamAV error: {}", error_msg);
            Ok(ScanResult {
                status: ScanStatus::Error,
                details: Some(error_msg),
            })
        } else {
            tracing::warn!("ClamAV: Unexpected response format");
            Ok(ScanResult {
                status: ScanStatus::Error,
                details: Some(format!("Unexpected response: {}", response)),
            })
        }
    }
}