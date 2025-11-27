pub mod service;
pub mod storage;
pub mod scanner;
pub mod error;

pub use service::AttachmentService;
pub use storage::{StorageBackend, LocalStorage};
pub use scanner::{VirusScanner, MockScanner};
pub use error::AttachmentError;

use core_domain::{Attachment, AttachmentEntityType, ScanStatus};
use uuid::Uuid;
use std::path::PathBuf;

/// Configuration for attachment service
#[derive(Debug, Clone)]
pub struct AttachmentConfig {
    /// Maximum file size in bytes (default: 100MB)
    pub max_file_size: i64,

    /// Allowed content types (empty = all allowed)
    pub allowed_content_types: Vec<String>,

    /// Base storage path for local storage
    pub storage_path: PathBuf,

    /// Whether to encrypt files at rest
    pub encrypt_at_rest: bool,

    /// Whether to enable virus scanning
    pub enable_virus_scan: bool,

    /// Whether to generate thumbnails for images
    pub generate_thumbnails: bool,

    /// Thumbnail size (max dimension in pixels)
    pub thumbnail_size: u32,
}

impl Default for AttachmentConfig {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_content_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
                "application/pdf".to_string(),
                "text/plain".to_string(),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            ],
            storage_path: PathBuf::from("./data/attachments"),
            encrypt_at_rest: false,
            enable_virus_scan: true,
            generate_thumbnails: true,
            thumbnail_size: 200,
        }
    }
}