use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

// Maximum size constants
pub const MAX_NAME_LENGTH: usize = 100;
pub const MAX_EMAIL_LENGTH: usize = 255;
pub const MAX_QUERY_LENGTH: usize = 1000;
#[allow(dead_code)]
pub const MAX_MESSAGE_LENGTH: usize = 10000;
pub const MAX_DESCRIPTION_LENGTH: usize = 5000;
pub const MAX_TITLE_LENGTH: usize = 200;
pub const MAX_LOCATION_LENGTH: usize = 500;
pub const MAX_FILENAME_LENGTH: usize = 255;
pub const MAX_PAGINATION_LIMIT: i64 = 1000;
#[allow(dead_code)]
pub const MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024; // 10MB
pub const MAX_ATTACHMENT_SIZE: usize = 100 * 1024 * 1024; // 100MB
#[allow(dead_code)]
pub const MAX_TAG_LENGTH: usize = 50;
#[allow(dead_code)]
pub const MAX_TAGS_COUNT: usize = 50;
pub const MAX_CONTACTS_COUNT: usize = 1000;

#[derive(Debug, Clone)]
pub struct ValidationError(pub String);

impl IntoResponse for ValidationError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Validation failed",
                "message": self.0,
            })),
        )
            .into_response()
    }
}

/// Validate a name field
pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() {
        return Err(ValidationError("Name cannot be empty".to_string()));
    }
    if name.len() > MAX_NAME_LENGTH {
        return Err(ValidationError(format!(
            "Name too long (max {} characters)",
            MAX_NAME_LENGTH
        )));
    }
    // Check for null bytes
    if name.contains('\0') {
        return Err(ValidationError(
            "Name contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

/// Validate an email address
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    if email.is_empty() {
        return Err(ValidationError("Email cannot be empty".to_string()));
    }
    if email.len() > MAX_EMAIL_LENGTH {
        return Err(ValidationError(format!(
            "Email too long (max {} characters)",
            MAX_EMAIL_LENGTH
        )));
    }
    // Basic email validation
    if !email.contains('@') {
        return Err(ValidationError(
            "Invalid email format: missing @".to_string(),
        ));
    }
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(ValidationError("Invalid email format".to_string()));
    }
    if parts[0].is_empty() || parts[1].is_empty() {
        return Err(ValidationError(
            "Invalid email format: empty local or domain part".to_string(),
        ));
    }
    if !parts[1].contains('.') {
        return Err(ValidationError(
            "Invalid email format: domain missing dot".to_string(),
        ));
    }
    // Check for null bytes
    if email.contains('\0') {
        return Err(ValidationError(
            "Email contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

/// Validate a password with strong security requirements
pub fn validate_password(password: &str) -> Result<(), ValidationError> {
    // Length requirements
    if password.len() < 12 {
        return Err(ValidationError(
            "Password must be at least 12 characters for security".to_string(),
        ));
    }
    if password.len() > 128 {
        return Err(ValidationError(
            "Password too long (max 128 characters)".to_string(),
        ));
    }

    // Character class requirements
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_numeric());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_lowercase {
        return Err(ValidationError(
            "Password must contain at least one lowercase letter".to_string(),
        ));
    }
    if !has_uppercase {
        return Err(ValidationError(
            "Password must contain at least one uppercase letter".to_string(),
        ));
    }
    if !has_digit {
        return Err(ValidationError(
            "Password must contain at least one digit".to_string(),
        ));
    }
    if !has_special {
        return Err(ValidationError(
            "Password must contain at least one special character (!@#$%^&*, etc.)".to_string(),
        ));
    }

    // Common password check (case-insensitive)
    const COMMON_PASSWORDS: &[&str] = &[
        "password123",
        "password1!",
        "qwerty123",
        "admin123",
        "welcome123",
        "letmein123",
        "passw0rd!",
        "12345678",
        "password1",
        "abc123456",
        "qwerty12345",
        "Password123!",
        "Welcome123!",
        "Admin123!",
    ];

    let password_lower = password.to_lowercase();
    for common in COMMON_PASSWORDS {
        if password_lower.contains(&common.to_lowercase()) {
            return Err(ValidationError(
                "Password is too common. Please choose a stronger, more unique password"
                    .to_string(),
            ));
        }
    }

    Ok(())
}

/// Validate pagination parameters
pub fn validate_pagination(limit: i64, offset: i64) -> Result<(), ValidationError> {
    if limit > MAX_PAGINATION_LIMIT {
        return Err(ValidationError(format!(
            "Limit too high (max {})",
            MAX_PAGINATION_LIMIT
        )));
    }
    if limit < 1 {
        return Err(ValidationError("Limit must be at least 1".to_string()));
    }
    if offset < 0 {
        return Err(ValidationError("Offset cannot be negative".to_string()));
    }
    Ok(())
}

/// Validate a search query string
pub fn validate_query(query: &str) -> Result<(), ValidationError> {
    if query.len() > MAX_QUERY_LENGTH {
        return Err(ValidationError(format!(
            "Query too long (max {} characters)",
            MAX_QUERY_LENGTH
        )));
    }
    // Check for null bytes
    if query.contains('\0') {
        return Err(ValidationError(
            "Query contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

/// Validate a description field
pub fn validate_description(description: &str) -> Result<(), ValidationError> {
    if description.len() > MAX_DESCRIPTION_LENGTH {
        return Err(ValidationError(format!(
            "Description too long (max {} characters)",
            MAX_DESCRIPTION_LENGTH
        )));
    }
    // Check for null bytes
    if description.contains('\0') {
        return Err(ValidationError(
            "Description contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

/// Validate a title field
pub fn validate_title(title: &str) -> Result<(), ValidationError> {
    if title.is_empty() {
        return Err(ValidationError("Title cannot be empty".to_string()));
    }
    if title.len() > MAX_TITLE_LENGTH {
        return Err(ValidationError(format!(
            "Title too long (max {} characters)",
            MAX_TITLE_LENGTH
        )));
    }
    // Check for null bytes
    if title.contains('\0') {
        return Err(ValidationError(
            "Title contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

/// Validate a location field
pub fn validate_location(location: &str) -> Result<(), ValidationError> {
    if location.len() > MAX_LOCATION_LENGTH {
        return Err(ValidationError(format!(
            "Location too long (max {} characters)",
            MAX_LOCATION_LENGTH
        )));
    }
    // Check for null bytes
    if location.contains('\0') {
        return Err(ValidationError(
            "Location contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

/// Validate a filename
#[allow(dead_code)]
pub fn validate_filename(filename: &str) -> Result<(), ValidationError> {
    if filename.is_empty() {
        return Err(ValidationError("Filename cannot be empty".to_string()));
    }
    if filename.len() > MAX_FILENAME_LENGTH {
        return Err(ValidationError(format!(
            "Filename too long (max {} characters)",
            MAX_FILENAME_LENGTH
        )));
    }
    // Check for path traversal attempts
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(ValidationError(
            "Filename contains invalid path characters".to_string(),
        ));
    }
    // Check for null bytes
    if filename.contains('\0') {
        return Err(ValidationError(
            "Filename contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

/// Validate file size
#[allow(dead_code)]
pub fn validate_file_size(size: usize, max_size: usize) -> Result<(), ValidationError> {
    if size > max_size {
        return Err(ValidationError(format!(
            "File too large (max {} bytes)",
            max_size
        )));
    }
    if size == 0 {
        return Err(ValidationError("File cannot be empty".to_string()));
    }
    Ok(())
}

/// Allowed file extensions for uploads (security whitelist)
#[allow(dead_code)]
const ALLOWED_EXTENSIONS: &[&str] = &[
    // Images
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "svg", "ico", // Documents
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md", "rtf", "odt", "ods", "odp",
    // Archives
    "zip", "tar", "gz", "7z", "rar", // Data
    "csv", "json", "xml", "yaml", "yml", // Media
    "mp3", "mp4", "wav", "avi", "mkv", "mov", "flac", // Code (for development use cases)
    "rs", "toml", "js", "ts", "py", "go", "java", "c", "cpp", "h",
];

/// Validate file type based on extension and MIME type
#[allow(dead_code)]
pub fn validate_file_type(filename: &str, content_type: &str) -> Result<(), ValidationError> {
    // Extract file extension
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| ValidationError("File has no extension".to_string()))?
        .to_lowercase();

    // Check if extension is allowed
    if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
        return Err(ValidationError(format!(
            "File type '.{}' is not allowed. Allowed types: {}",
            extension,
            ALLOWED_EXTENSIONS.join(", ")
        )));
    }

    // Validate MIME type matches extension (basic check)
    let mime_valid = match extension.as_str() {
        "jpg" | "jpeg" => content_type.starts_with("image/jpeg"),
        "png" => content_type.starts_with("image/png"),
        "gif" => content_type.starts_with("image/gif"),
        "pdf" => content_type.starts_with("application/pdf"),
        "zip" => {
            content_type.starts_with("application/zip")
                || content_type.starts_with("application/x-zip")
        }
        "json" => content_type.starts_with("application/json"),
        "xml" => {
            content_type.starts_with("application/xml") || content_type.starts_with("text/xml")
        }
        "txt" | "md" => {
            content_type.starts_with("text/plain") || content_type.starts_with("text/markdown")
        }
        "csv" => {
            content_type.starts_with("text/csv") || content_type.starts_with("application/csv")
        }
        // For other types, accept application/octet-stream or allow pass-through
        _ => content_type.starts_with("application/octet-stream") || !content_type.is_empty(),
    };

    if !mime_valid {
        return Err(ValidationError(format!(
            "MIME type '{}' does not match file extension '.{}'",
            content_type, extension
        )));
    }

    Ok(())
}

/// Validate a tag
#[allow(dead_code)]
pub fn validate_tag(tag: &str) -> Result<(), ValidationError> {
    if tag.is_empty() {
        return Err(ValidationError("Tag cannot be empty".to_string()));
    }
    if tag.len() > MAX_TAG_LENGTH {
        return Err(ValidationError(format!(
            "Tag too long (max {} characters)",
            MAX_TAG_LENGTH
        )));
    }
    // Check for null bytes
    if tag.contains('\0') {
        return Err(ValidationError(
            "Tag contains invalid characters".to_string(),
        ));
    }
    Ok(())
}

/// Validate a list of tags
#[allow(dead_code)]
pub fn validate_tags(tags: &[String]) -> Result<(), ValidationError> {
    if tags.len() > MAX_TAGS_COUNT {
        return Err(ValidationError(format!(
            "Too many tags (max {})",
            MAX_TAGS_COUNT
        )));
    }
    for tag in tags {
        validate_tag(tag)?;
    }
    Ok(())
}

/// Validate a list of UUIDs (e.g., contact IDs)
pub fn validate_uuid_list(
    uuids: &[uuid::Uuid],
    max_count: usize,
    field_name: &str,
) -> Result<(), ValidationError> {
    if uuids.len() > max_count {
        return Err(ValidationError(format!(
            "Too many {} (max {})",
            field_name, max_count
        )));
    }
    Ok(())
}

/// Validate optional field with validator function
pub fn validate_optional<T, F>(value: &Option<T>, validator: F) -> Result<(), ValidationError>
where
    F: Fn(&T) -> Result<(), ValidationError>,
{
    if let Some(v) = value {
        validator(v)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_name() {
        assert!(validate_name("John Doe").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name(&"a".repeat(MAX_NAME_LENGTH + 1)).is_err());
        assert!(validate_name("test\0name").is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("").is_err());
        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("user@domain").is_err());
        assert!(validate_email(&format!("{}@example.com", "a".repeat(MAX_EMAIL_LENGTH))).is_err());
    }

    #[test]
    fn test_validate_password() {
        assert!(validate_password("S3cur3P@ssXyz").is_ok());
        assert!(validate_password("short").is_err());
        assert!(validate_password(&"a".repeat(129)).is_err());
    }

    #[test]
    fn test_validate_pagination() {
        assert!(validate_pagination(50, 0).is_ok());
        assert!(validate_pagination(1, 100).is_ok());
        assert!(validate_pagination(MAX_PAGINATION_LIMIT + 1, 0).is_err());
        assert!(validate_pagination(0, 0).is_err());
        assert!(validate_pagination(50, -1).is_err());
    }

    #[test]
    fn test_validate_query() {
        assert!(validate_query("search term").is_ok());
        assert!(validate_query(&"a".repeat(MAX_QUERY_LENGTH + 1)).is_err());
        assert!(validate_query("test\0query").is_err());
    }

    #[test]
    fn test_validate_filename() {
        assert!(validate_filename("document.pdf").is_ok());
        assert!(validate_filename("").is_err());
        assert!(validate_filename("../etc/passwd").is_err());
        assert!(validate_filename("path/to/file").is_err());
        assert!(validate_filename("C:\\Windows\\file.exe").is_err());
    }

    #[test]
    fn test_validate_file_size() {
        assert!(validate_file_size(1000, MAX_ATTACHMENT_SIZE).is_ok());
        assert!(validate_file_size(0, MAX_ATTACHMENT_SIZE).is_err());
        assert!(validate_file_size(MAX_ATTACHMENT_SIZE + 1, MAX_ATTACHMENT_SIZE).is_err());
    }

    #[test]
    fn test_validate_tags() {
        assert!(validate_tags(&["tag1".to_string(), "tag2".to_string()]).is_ok());
        assert!(validate_tags(&vec!["tag".to_string(); MAX_TAGS_COUNT + 1]).is_err());
        assert!(validate_tags(&["".to_string()]).is_err());
        assert!(validate_tags(&["a".repeat(MAX_TAG_LENGTH + 1)]).is_err());
    }

    // Additional file validation tests from SEC-009
    #[test]
    fn test_validate_file_upload_valid() {
        assert!(validate_file_upload("test.pdf", "application/pdf", 1000).is_ok());
        assert!(validate_file_upload("image.png", "image/png", 5000).is_ok());
        assert!(validate_file_upload("data.csv", "text/csv", 2000).is_ok());
        assert!(validate_file_upload("contact.vcf", "text/vcard", 500).is_ok());
        assert!(validate_file_upload(
            "document.docx",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            10000
        )
        .is_ok());
    }

    #[test]
    fn test_validate_file_upload_invalid_extensions() {
        assert!(validate_file_upload("malware.exe", "application/x-msdownload", 1000).is_err());
        assert!(validate_file_upload("script.sh", "text/x-shellscript", 500).is_err());
        assert!(validate_file_upload("binary.bin", "application/octet-stream", 2000).is_err());
        assert!(validate_file_upload("code.py", "text/x-python", 1500).is_err());
    }

    #[test]
    fn test_validate_file_upload_mime_mismatch() {
        assert!(validate_file_upload("fake.pdf", "image/png", 1000).is_err());
        assert!(validate_file_upload("fake.png", "application/pdf", 1000).is_err());
    }

    #[test]
    fn test_validate_file_upload_path_traversal() {
        assert!(validate_file_upload("../../../etc/passwd", "text/plain", 100).is_err());
        assert!(validate_file_upload("dir/file.txt", "text/plain", 100).is_err());
        assert!(validate_file_upload("subdir\\file.txt", "text/plain", 100).is_err());
    }

    #[test]
    fn test_validate_file_upload_size_limits() {
        assert!(validate_file_upload("huge.zip", "application/zip", 200_000_000).is_err());
        assert!(validate_file_upload("empty.txt", "text/plain", 0).is_err());
        assert!(validate_file_upload("large.zip", "application/zip", 99_000_000).is_ok());
    }

    #[test]
    fn test_sanitize_filename_function() {
        assert_eq!(sanitize_filename("hello world.txt"), "hello_world.txt");
        assert_eq!(sanitize_filename("file@#$%.pdf"), "file____.pdf");
        assert_eq!(
            sanitize_filename("normal-file_123.csv"),
            "normal-file_123.csv"
        );
    }
}

/// Comprehensive file upload validation combining size, extension, MIME type, and path safety
pub fn validate_file_upload(
    filename: &str,
    content_type: &str,
    size: usize,
) -> Result<(), ValidationError> {
    use std::path::Path;

    // Size validation
    if size == 0 {
        return Err(ValidationError("File is empty".into()));
    }
    if size > MAX_ATTACHMENT_SIZE {
        return Err(ValidationError(format!(
            "File too large: {} MB (max: {} MB)",
            size / 1024 / 1024,
            MAX_ATTACHMENT_SIZE / 1024 / 1024
        )));
    }

    // Filename length validation
    if filename.len() > MAX_FILENAME_LENGTH {
        return Err(ValidationError(
            "Filename too long (max 255 characters)".into(),
        ));
    }

    // Path traversal protection
    if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
        return Err(ValidationError(
            "Invalid filename (path traversal attempt detected)".into(),
        ));
    }

    // Extension validation with strict whitelist (excluding executable code)
    const UPLOAD_ALLOWED_EXTENSIONS: &[&str] = &[
        // Images
        "jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico", // Documents
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md", "rtf", "odt", "ods",
        "odp", // Archives
        "zip", "tar", "gz", "7z", "rar", // Data
        "csv", "json", "xml", "yaml", "yml", // Contact formats
        "vcf", "ics", "eml",
    ];

    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| ValidationError("No file extension found".into()))?
        .to_lowercase();

    if !UPLOAD_ALLOWED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(ValidationError(format!(
            "File type not allowed: .{} (allowed: {})",
            ext,
            UPLOAD_ALLOWED_EXTENSIONS.join(", ")
        )));
    }

    // MIME type validation
    validate_upload_mime_type(&ext, content_type)?;

    Ok(())
}

/// Validate MIME type matches file extension for strict security
fn validate_upload_mime_type(ext: &str, content_type: &str) -> Result<(), ValidationError> {
    let expected_mime = match ext {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "json" => "application/json",
        "xml" => "application/xml",
        "csv" => "text/csv",
        "txt" | "md" => "text/plain",
        "vcf" => "text/vcard",
        "ics" => "text/calendar",
        // SVG can be image/svg+xml or text/xml
        "svg" => {
            return if content_type.contains("svg") || content_type.contains("xml") {
                Ok(())
            } else {
                Err(ValidationError("Invalid MIME type for SVG file".into()))
            };
        }
        // Skip strict MIME check for office documents (complex MIME types)
        "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "odt" | "ods" | "odp" => return Ok(()),
        // Skip strict check for archives (complex MIME types)
        "tar" | "gz" | "7z" | "rar" => return Ok(()),
        _ => return Ok(()), // Skip strict check for other types
    };

    if !content_type.starts_with(expected_mime) {
        return Err(ValidationError(format!(
            "MIME type mismatch: expected {}, got {}",
            expected_mime, content_type
        )));
    }

    Ok(())
}

/// Sanitize filename by replacing unsafe characters with underscores
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
