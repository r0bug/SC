use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

// Maximum size constants
pub const MAX_NAME_LENGTH: usize = 100;
pub const MAX_EMAIL_LENGTH: usize = 255;
pub const MAX_QUERY_LENGTH: usize = 1000;
pub const MAX_MESSAGE_LENGTH: usize = 10000;
pub const MAX_DESCRIPTION_LENGTH: usize = 5000;
pub const MAX_TITLE_LENGTH: usize = 200;
pub const MAX_LOCATION_LENGTH: usize = 500;
pub const MAX_FILENAME_LENGTH: usize = 255;
pub const MAX_PAGINATION_LIMIT: i64 = 1000;
pub const MAX_REQUEST_BODY_SIZE: usize = 10 * 1024 * 1024; // 10MB
pub const MAX_ATTACHMENT_SIZE: usize = 100 * 1024 * 1024; // 100MB
pub const MAX_TAG_LENGTH: usize = 50;
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
        "password123", "password1!", "qwerty123", "admin123", "welcome123",
        "letmein123", "passw0rd!", "12345678", "password1", "abc123456",
        "qwerty12345", "Password123!", "Welcome123!", "Admin123!",
    ];

    let password_lower = password.to_lowercase();
    for common in COMMON_PASSWORDS {
        if password_lower.contains(&common.to_lowercase()) {
            return Err(ValidationError(
                "Password is too common. Please choose a stronger, more unique password".to_string(),
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

/// Validate a tag
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
        assert!(validate_password("password123").is_ok());
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
}
