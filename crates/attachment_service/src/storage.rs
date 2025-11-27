use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use crate::error::AttachmentError;

/// Trait for storage backend implementations
#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store a file and return its storage path
    async fn store(&self, data: &[u8], filename: &str) -> Result<String, AttachmentError>;

    /// Retrieve file data
    async fn retrieve(&self, storage_path: &str) -> Result<Vec<u8>, AttachmentError>;

    /// Delete a file
    async fn delete(&self, storage_path: &str) -> Result<(), AttachmentError>;

    /// Check if file exists
    async fn exists(&self, storage_path: &str) -> Result<bool, AttachmentError>;

    /// Get file size
    async fn size(&self, storage_path: &str) -> Result<i64, AttachmentError>;
}

/// Local filesystem storage
pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    fn resolve_path(&self, storage_path: &str) -> PathBuf {
        self.base_path.join(storage_path)
    }

    fn generate_storage_path(&self, filename: &str) -> String {
        let id = Uuid::new_v4();
        let extension = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");

        // Organize by date and UUID prefix for better directory distribution
        let now = chrono::Utc::now();
        let date_path = now.format("%Y/%m/%d").to_string();
        format!(
            "{}/{}/{}.{}",
            date_path,
            &id.to_string()[..2],
            id,
            extension
        )
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn store(&self, data: &[u8], filename: &str) -> Result<String, AttachmentError> {
        let storage_path = self.generate_storage_path(filename);
        let full_path = self.resolve_path(&storage_path);

        // Create parent directories
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Write file
        let mut file = fs::File::create(&full_path).await?;
        file.write_all(data).await?;
        file.flush().await?;

        tracing::info!("Stored file at: {:?}", full_path);

        Ok(storage_path)
    }

    async fn retrieve(&self, storage_path: &str) -> Result<Vec<u8>, AttachmentError> {
        let full_path = self.resolve_path(storage_path);
        fs::read(&full_path)
            .await
            .map_err(|e| AttachmentError::IoError(e))
    }

    async fn delete(&self, storage_path: &str) -> Result<(), AttachmentError> {
        let full_path = self.resolve_path(storage_path);
        fs::remove_file(&full_path)
            .await
            .map_err(|e| AttachmentError::IoError(e))
    }

    async fn exists(&self, storage_path: &str) -> Result<bool, AttachmentError> {
        let full_path = self.resolve_path(storage_path);
        Ok(full_path.exists())
    }

    async fn size(&self, storage_path: &str) -> Result<i64, AttachmentError> {
        let full_path = self.resolve_path(storage_path);
        let metadata = fs::metadata(&full_path).await?;
        Ok(metadata.len() as i64)
    }
}

#[cfg(feature = "s3-storage")]
pub struct S3Storage {
    client: aws_sdk_s3::Client,
    bucket: String,
    prefix: String,
}

#[cfg(feature = "s3-storage")]
impl S3Storage {
    /// Create S3 storage with existing client
    pub fn new(client: aws_sdk_s3::Client, bucket: String) -> Self {
        Self {
            client,
            bucket,
            prefix: "attachments".to_string(),
        }
    }

    /// Create S3 storage with custom prefix
    pub fn with_prefix(client: aws_sdk_s3::Client, bucket: String, prefix: String) -> Self {
        Self { client, bucket, prefix }
    }

    /// Create S3-compatible storage (MinIO, DigitalOcean Spaces, etc.)
    ///
    /// # Arguments
    /// * `endpoint_url` - Custom endpoint URL (e.g., "http://localhost:9000" for MinIO)
    /// * `region` - Region (can be any string for MinIO, e.g., "us-east-1")
    /// * `access_key` - Access key ID
    /// * `secret_key` - Secret access key
    /// * `bucket` - Bucket name
    pub async fn new_s3_compatible(
        endpoint_url: &str,
        region: &str,
        access_key: &str,
        secret_key: &str,
        bucket: String,
    ) -> Result<Self, AttachmentError> {
        use aws_sdk_s3::config::{Builder, Credentials, Region};

        let credentials = Credentials::new(
            access_key,
            secret_key,
            None,
            None,
            "static",
        );

        let config = Builder::new()
            .endpoint_url(endpoint_url)
            .region(Region::new(region.to_string()))
            .credentials_provider(credentials)
            .force_path_style(true) // Required for MinIO
            .build();

        let client = aws_sdk_s3::Client::from_conf(config);

        tracing::info!("Connected to S3-compatible storage at {}", endpoint_url);

        Ok(Self {
            client,
            bucket,
            prefix: "attachments".to_string(),
        })
    }

    /// Create from environment variables
    /// - S3_ENDPOINT_URL (optional, for MinIO/S3-compatible)
    /// - S3_REGION (default: us-east-1)
    /// - S3_ACCESS_KEY_ID
    /// - S3_SECRET_ACCESS_KEY
    /// - S3_BUCKET
    pub async fn from_env() -> Result<Self, AttachmentError> {
        let endpoint_url = std::env::var("S3_ENDPOINT_URL").ok();
        let region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
        let access_key = std::env::var("S3_ACCESS_KEY_ID")
            .map_err(|_| AttachmentError::StorageError("S3_ACCESS_KEY_ID not set".to_string()))?;
        let secret_key = std::env::var("S3_SECRET_ACCESS_KEY")
            .map_err(|_| AttachmentError::StorageError("S3_SECRET_ACCESS_KEY not set".to_string()))?;
        let bucket = std::env::var("S3_BUCKET")
            .map_err(|_| AttachmentError::StorageError("S3_BUCKET not set".to_string()))?;

        if let Some(endpoint) = endpoint_url {
            // S3-compatible (MinIO, DigitalOcean Spaces, etc.)
            Self::new_s3_compatible(&endpoint, &region, &access_key, &secret_key, bucket).await
        } else {
            // AWS S3
            use aws_sdk_s3::config::{Builder, Credentials, Region};

            let credentials = Credentials::new(
                &access_key,
                &secret_key,
                None,
                None,
                "static",
            );

            let config = Builder::new()
                .region(Region::new(region))
                .credentials_provider(credentials)
                .build();

            let client = aws_sdk_s3::Client::from_conf(config);

            tracing::info!("Connected to AWS S3");

            Ok(Self {
                client,
                bucket,
                prefix: "attachments".to_string(),
            })
        }
    }

    fn generate_key(&self, filename: &str) -> String {
        let id = Uuid::new_v4();
        let extension = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");

        // Organize by date for better listing performance
        let now = chrono::Utc::now();
        let date_path = now.format("%Y/%m/%d").to_string();

        format!(
            "{}/{}/{}/{}.{}",
            self.prefix,
            date_path,
            &id.to_string()[..2],
            id,
            extension
        )
    }
}

#[cfg(feature = "s3-storage")]
#[async_trait]
impl StorageBackend for S3Storage {
    async fn store(&self, data: &[u8], filename: &str) -> Result<String, AttachmentError> {
        let key = self.generate_key(filename);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(data.to_vec().into())
            .send()
            .await
            .map_err(|e| AttachmentError::StorageError(e.to_string()))?;

        tracing::info!("Stored file in S3: s3://{}/{}", self.bucket, key);

        Ok(key)
    }

    async fn retrieve(&self, storage_path: &str) -> Result<Vec<u8>, AttachmentError> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
            .map_err(|e| AttachmentError::StorageError(e.to_string()))?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| AttachmentError::StorageError(e.to_string()))?;

        Ok(data.to_vec())
    }

    async fn delete(&self, storage_path: &str) -> Result<(), AttachmentError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
            .map_err(|e| AttachmentError::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn exists(&self, storage_path: &str) -> Result<bool, AttachmentError> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn size(&self, storage_path: &str) -> Result<i64, AttachmentError> {
        let response = self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(storage_path)
            .send()
            .await
            .map_err(|e| AttachmentError::StorageError(e.to_string()))?;

        Ok(response.content_length().unwrap_or(0))
    }
}