use crate::{AttachmentConfig, AttachmentError, StorageBackend, VirusScanner};
use core_domain::{Attachment, AttachmentEntityType, ScanStatus};
use image::ImageFormat;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::io::Cursor;
use std::sync::Arc;
use uuid::Uuid;

/// Default thumbnail size (width x height) - can be overridden by config
const DEFAULT_THUMBNAIL_SIZE: u32 = 200;

/// Image content types that support thumbnail generation
const THUMBNAIL_SUPPORTED_TYPES: &[&str] = &[
    "image/jpeg",
    "image/jpg",
    "image/png",
    "image/gif",
    "image/webp",
];

pub struct AttachmentService {
    config: AttachmentConfig,
    storage: Arc<dyn StorageBackend>,
    scanner: Arc<dyn VirusScanner>,
    db: PgPool,
}

impl AttachmentService {
    pub fn new(
        config: AttachmentConfig,
        storage: Arc<dyn StorageBackend>,
        scanner: Arc<dyn VirusScanner>,
        db: PgPool,
    ) -> Self {
        Self {
            config,
            storage,
            scanner,
            db,
        }
    }

    /// Upload and process an attachment
    pub async fn upload(
        &self,
        data: &[u8],
        filename: String,
        content_type: String,
        entity_type: AttachmentEntityType,
        entity_id: Uuid,
        uploaded_by: Uuid,
    ) -> Result<Attachment, AttachmentError> {
        // Validate size
        let size_bytes = data.len() as i64;
        if size_bytes > self.config.max_file_size {
            return Err(AttachmentError::FileTooLarge(
                size_bytes,
                self.config.max_file_size,
            ));
        }

        // Validate content type
        if !self.config.allowed_content_types.is_empty()
            && !self.config.allowed_content_types.contains(&content_type)
        {
            return Err(AttachmentError::InvalidContentType(content_type));
        }

        // Calculate checksum
        let mut hasher = Sha256::new();
        hasher.update(data);
        let checksum = hex::encode(hasher.finalize());

        // Store file
        let storage_path = self.storage.store(data, &filename).await?;

        // Generate thumbnail for images
        let thumbnail_path = if self.config.generate_thumbnails && self.supports_thumbnail(&content_type) {
            match self.generate_thumbnail(data, &filename).await {
                Ok(Some(thumb_path)) => {
                    tracing::debug!("Generated thumbnail: {}", thumb_path);
                    Some(thumb_path)
                }
                Ok(None) => None,
                Err(e) => {
                    tracing::warn!("Failed to generate thumbnail: {}", e);
                    None // Continue without thumbnail
                }
            }
        } else {
            None
        };

        // Scan for viruses
        let scan_result = if self.config.enable_virus_scan {
            let path = std::path::Path::new(&storage_path);
            self.scanner.scan_file(path).await?
        } else {
            crate::scanner::ScanResult {
                status: ScanStatus::Clean,
                details: None,
            }
        };

        // If infected, delete file and return error
        if scan_result.status == ScanStatus::Infected {
            let _ = self.storage.delete(&storage_path).await;
            return Err(AttachmentError::VirusDetected(
                scan_result.details.unwrap_or_default(),
            ));
        }

        // Create attachment record
        let attachment = Attachment {
            id: Uuid::new_v4(),
            filename,
            content_type,
            size_bytes,
            storage_path,
            thumbnail_path,
            entity_type,
            entity_id,
            uploaded_by,
            checksum,
            encrypted: self.config.encrypt_at_rest,
            scan_status: scan_result.status.clone(),
            scan_details: scan_result.details,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        };

        // Persist to database
        self.save_attachment(&attachment).await?;

        tracing::info!(
            "Uploaded attachment: {} ({} bytes, scan: {:?})",
            attachment.id,
            attachment.size_bytes,
            attachment.scan_status
        );

        Ok(attachment)
    }

    /// Download attachment data
    pub async fn download(&self, attachment_id: Uuid) -> Result<Vec<u8>, AttachmentError> {
        let attachment = self.get_attachment(attachment_id).await?;

        // Verify scan status before allowing download
        if attachment.scan_status == ScanStatus::Infected {
            return Err(AttachmentError::VirusDetected(
                attachment.scan_details.unwrap_or_default(),
            ));
        }

        let data = self.storage.retrieve(&attachment.storage_path).await?;

        // Verify checksum
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let checksum = hex::encode(hasher.finalize());

        if checksum != attachment.checksum {
            tracing::error!(
                "Checksum mismatch for attachment {}: expected {}, got {}",
                attachment_id,
                attachment.checksum,
                checksum
            );
            return Err(AttachmentError::InvalidChecksum);
        }

        Ok(data)
    }

    /// Delete an attachment
    pub async fn delete(&self, attachment_id: Uuid) -> Result<(), AttachmentError> {
        let attachment = self.get_attachment(attachment_id).await?;

        // Delete from storage
        self.storage.delete(&attachment.storage_path).await?;

        // Delete from database
        sqlx::query!(
            "DELETE FROM attachments WHERE id = $1",
            attachment_id
        )
        .execute(&self.db)
        .await?;

        tracing::info!("Deleted attachment: {}", attachment_id);

        Ok(())
    }

    /// List attachments for an entity
    pub async fn list_for_entity(
        &self,
        entity_type: AttachmentEntityType,
        entity_id: Uuid,
    ) -> Result<Vec<Attachment>, AttachmentError> {
        let entity_type_str = format!("{:?}", entity_type);

        let rows = sqlx::query!(
            r#"
            SELECT id, filename, content_type, size_bytes, storage_path, thumbnail_path,
                   entity_type, entity_id, uploaded_by, checksum, encrypted,
                   scan_status, scan_details, metadata, created_at
            FROM attachments
            WHERE entity_type = $1 AND entity_id = $2
            ORDER BY created_at DESC
            "#,
            entity_type_str,
            entity_id
        )
        .fetch_all(&self.db)
        .await?;

        let attachments = rows
            .into_iter()
            .map(|row| {
                let scan_status = match row.scan_status.as_str() {
                    "Pending" => ScanStatus::Pending,
                    "Clean" => ScanStatus::Clean,
                    "Infected" => ScanStatus::Infected,
                    "Error" => ScanStatus::Error,
                    _ => ScanStatus::Pending,
                };

                Attachment {
                    id: row.id,
                    filename: row.filename,
                    content_type: row.content_type,
                    size_bytes: row.size_bytes,
                    storage_path: row.storage_path,
                    thumbnail_path: row.thumbnail_path,
                    entity_type: entity_type.clone(),
                    entity_id: row.entity_id,
                    uploaded_by: row.uploaded_by,
                    checksum: row.checksum,
                    encrypted: row.encrypted,
                    scan_status,
                    scan_details: row.scan_details,
                    metadata: row.metadata,
                    created_at: row.created_at,
                }
            })
            .collect();

        Ok(attachments)
    }

    /// Get a single attachment
    async fn get_attachment(&self, id: Uuid) -> Result<Attachment, AttachmentError> {
        let row = sqlx::query!(
            r#"
            SELECT id, filename, content_type, size_bytes, storage_path, thumbnail_path,
                   entity_type, entity_id, uploaded_by, checksum, encrypted,
                   scan_status, scan_details, metadata, created_at
            FROM attachments
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(AttachmentError::NotFound(id))?;

        let entity_type = match row.entity_type.as_str() {
            "Contact" => AttachmentEntityType::Contact,
            "Note" => AttachmentEntityType::Note,
            "Project" => AttachmentEntityType::Project,
            "CalendarEvent" => AttachmentEntityType::CalendarEvent,
            "Communication" => AttachmentEntityType::Communication,
            _ => AttachmentEntityType::Contact, // Default fallback
        };

        let scan_status = match row.scan_status.as_str() {
            "Pending" => ScanStatus::Pending,
            "Clean" => ScanStatus::Clean,
            "Infected" => ScanStatus::Infected,
            "Error" => ScanStatus::Error,
            _ => ScanStatus::Pending,
        };

        Ok(Attachment {
            id: row.id,
            filename: row.filename,
            content_type: row.content_type,
            size_bytes: row.size_bytes,
            storage_path: row.storage_path,
            thumbnail_path: row.thumbnail_path,
            entity_type,
            entity_id: row.entity_id,
            uploaded_by: row.uploaded_by,
            checksum: row.checksum,
            encrypted: row.encrypted,
            scan_status,
            scan_details: row.scan_details,
            metadata: row.metadata,
            created_at: row.created_at,
        })
    }

    /// Save attachment to database
    async fn save_attachment(&self, attachment: &Attachment) -> Result<(), AttachmentError> {
        let entity_type_str = format!("{:?}", attachment.entity_type);
        let scan_status_str = format!("{:?}", attachment.scan_status);

        sqlx::query!(
            r#"
            INSERT INTO attachments (
                id, filename, content_type, size_bytes, storage_path, thumbnail_path,
                entity_type, entity_id, uploaded_by, checksum, encrypted,
                scan_status, scan_details, metadata, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
            attachment.id,
            attachment.filename,
            attachment.content_type,
            attachment.size_bytes,
            attachment.storage_path,
            attachment.thumbnail_path,
            entity_type_str,
            attachment.entity_id,
            attachment.uploaded_by,
            attachment.checksum,
            attachment.encrypted,
            scan_status_str,
            attachment.scan_details,
            attachment.metadata,
            attachment.created_at
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Check if content type supports thumbnail generation
    fn supports_thumbnail(&self, content_type: &str) -> bool {
        THUMBNAIL_SUPPORTED_TYPES
            .iter()
            .any(|&t| content_type.to_lowercase().starts_with(t))
    }

    /// Generate a thumbnail for an image
    async fn generate_thumbnail(
        &self,
        data: &[u8],
        original_filename: &str,
    ) -> Result<Option<String>, AttachmentError> {
        // Load the image
        let img = match image::load_from_memory(data) {
            Ok(img) => img,
            Err(e) => {
                tracing::debug!("Could not decode image for thumbnail: {}", e);
                return Ok(None);
            }
        };

        // Create thumbnail maintaining aspect ratio
        let size = self.config.thumbnail_size;
        let thumbnail = img.thumbnail(size, size);

        // Encode thumbnail as JPEG for consistent output
        let mut thumbnail_data = Vec::new();
        let mut cursor = Cursor::new(&mut thumbnail_data);

        thumbnail
            .write_to(&mut cursor, ImageFormat::Jpeg)
            .map_err(|e| AttachmentError::Other(format!("Failed to encode thumbnail: {}", e)))?;

        // Generate thumbnail filename
        let thumb_filename = format!(
            "thumb_{}_{}.jpg",
            Uuid::new_v4(),
            std::path::Path::new(original_filename)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("image")
        );

        // Store thumbnail
        let thumb_path = self.storage.store(&thumbnail_data, &thumb_filename).await?;

        tracing::info!(
            "Generated thumbnail: {} ({}x{} -> {}x{})",
            thumb_path,
            img.width(),
            img.height(),
            thumbnail.width(),
            thumbnail.height()
        );

        Ok(Some(thumb_path))
    }

    /// Download a thumbnail
    pub async fn download_thumbnail(&self, attachment_id: Uuid) -> Result<Option<Vec<u8>>, AttachmentError> {
        let attachment = self.get_attachment(attachment_id).await?;

        match attachment.thumbnail_path {
            Some(ref path) => {
                let data = self.storage.retrieve(path).await?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }
}