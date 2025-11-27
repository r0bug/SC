use crate::{
    config::ImportFormat,
    connector::{ConnectorMetadata, ImportConnector, ParseResult},
    ImportError,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

/// LinkedIn Export Connector - Imports connections.csv from LinkedIn data export
///
/// LinkedIn export CSV format includes:
/// - First Name, Last Name, Email Address
/// - Company, Position
/// - Connected On (date)
/// - URL (LinkedIn profile URL)
pub struct LinkedInConnector;

impl LinkedInConnector {
    pub fn new() -> Self {
        Self
    }

    /// Extract LinkedIn username from profile URL
    fn extract_linkedin_username(url: &str) -> Option<String> {
        // LinkedIn URLs: https://www.linkedin.com/in/username
        if let Some(pos) = url.find("/in/") {
            let username = &url[pos + 4..];
            // Remove trailing slash if present
            let username = username.trim_end_matches('/');
            if !username.is_empty() {
                return Some(username.to_string());
            }
        }
        None
    }
}

#[async_trait]
impl ImportConnector for LinkedInConnector {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "linkedin".to_string(),
            name: "LinkedIn".to_string(),
            description: "Import contacts from LinkedIn data export (Connections.csv)".to_string(),
            supported_extensions: vec!["csv".to_string()],
            supported_mime_types: vec!["text/csv".to_string()],
            format: ImportFormat::LinkedIn,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        // Check for LinkedIn export CSV structure
        // LinkedIn export filenames are typically "Connections.csv" or similar
        if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
            let filename_lower = filename.to_lowercase();
            if filename_lower.contains("connection") {
                if let Ok(mut reader) = csv::Reader::from_path(file_path) {
                    if let Ok(headers) = reader.headers() {
                        let headers_str = headers.iter().collect::<Vec<_>>().join(",");
                        // LinkedIn connections CSV has these specific columns
                        return headers_str.contains("First Name")
                            && headers_str.contains("Last Name")
                            && (headers_str.contains("Company") || headers_str.contains("Position"));
                    }
                }
            }
        }
        false
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        // LinkedIn export typically contains:
        // First Name, Last Name, Email Address, Company, Position, Connected On, URL

        let mut reader = csv::Reader::from_path(file_path)?;
        let headers: Vec<String> = reader.headers()?.iter().map(|h| h.to_string()).collect();

        let mut rows = Vec::new();
        let mut warnings = Vec::new();
        let mut with_email = 0;
        let mut with_linkedin_url = 0;

        for (idx, result) in reader.records().enumerate() {
            match result {
                Ok(record) => {
                    let mut row = HashMap::new();

                    for (i, field) in record.iter().enumerate() {
                        if let Some(header) = headers.get(i) {
                            // Normalize LinkedIn column names to our standard fields
                            let normalized_key = match header.as_str() {
                                "First Name" => "first_name",
                                "Last Name" => "last_name",
                                "Email Address" => "email",
                                "Company" => "organization",
                                "Position" => "title",
                                "Connected On" => "connection_date",
                                "URL" => "linkedin_url",
                                // Additional columns that may appear in LinkedIn exports
                                "Industry" => "industry",
                                "Location" => "location",
                                _ => header.as_str(),
                            };

                            let value = field.trim();
                            if !value.is_empty() {
                                row.insert(normalized_key.to_string(), value.to_string());
                            }
                        }
                    }

                    // Only add rows that have at least a name
                    if row.contains_key("first_name") || row.contains_key("last_name") {
                        // Add source metadata
                        row.insert("source".to_string(), "linkedin".to_string());

                        // Extract LinkedIn username from URL for social_handles
                        if let Some(url) = row.get("linkedin_url") {
                            with_linkedin_url += 1;
                            if let Some(username) = Self::extract_linkedin_username(url) {
                                row.insert("linkedin_username".to_string(), username);
                            }
                        }

                        if row.contains_key("email") {
                            with_email += 1;
                        }

                        rows.push(row);
                    } else {
                        warnings.push(format!("Row {} missing name, skipped", idx + 1));
                    }
                }
                Err(e) => {
                    warnings.push(format!("CSV parsing error at row {}: {}", idx + 1, e));
                }
            }
        }

        // Standard field mappings
        let suggested_mappings = vec![
            ("first_name".to_string(), "first_name".to_string()),
            ("last_name".to_string(), "last_name".to_string()),
            ("email".to_string(), "email".to_string()),
            ("organization".to_string(), "organization".to_string()),
            ("title".to_string(), "title".to_string()),
            ("linkedin_url".to_string(), "social_handles.linkedin".to_string()),
            ("linkedin_username".to_string(), "social_handles.linkedin_username".to_string()),
            ("location".to_string(), "address".to_string()),
            ("connection_date".to_string(), "metadata.linkedin_connected_on".to_string()),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "linkedin_csv".to_string());
        metadata.insert("total_contacts".to_string(), rows.len().to_string());
        metadata.insert("contacts_with_email".to_string(), with_email.to_string());
        metadata.insert("contacts_with_linkedin_url".to_string(), with_linkedin_url.to_string());

        // Add info about what was found
        if with_email == 0 && rows.len() > 0 {
            warnings.push(format!(
                "Note: LinkedIn exports typically don't include email addresses (found {} contacts with 0 emails).",
                rows.len()
            ));
        }

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }
}

/// Twitter/X Export Connector - Imports following.js/followers.js from Twitter data export
///
/// Twitter exports data as JavaScript files in this format:
/// ```javascript
/// window.YTD.following.part0 = [ { "following" : { "accountId" : "123", "userLink" : "https://twitter.com/username" } } ]
/// ```
///
/// This connector parses both following.js and followers.js files.
pub struct TwitterConnector;

impl TwitterConnector {
    pub fn new() -> Self {
        Self
    }

    /// Extract Twitter username from profile URL
    fn extract_twitter_username(url: &str) -> Option<String> {
        // Twitter URLs: https://twitter.com/username or https://x.com/username
        let url = url.trim().trim_end_matches('/');
        if let Some(pos) = url.rfind('/') {
            let username = &url[pos + 1..];
            if !username.is_empty() && !username.contains('/') {
                return Some(username.to_string());
            }
        }
        None
    }

    /// Parse Twitter's JavaScript export format
    fn parse_twitter_js(content: &str) -> Result<Vec<serde_json::Value>, ImportError> {
        // Twitter export format: window.YTD.{type}.part0 = [...]
        // We need to extract the JSON array after the '='

        let content = content.trim();

        // Find the JSON array start
        let json_start = content.find('[')
            .ok_or_else(|| ImportError::Other("Could not find JSON array in Twitter export".to_string()))?;

        let json_content = &content[json_start..];

        // Parse the JSON array
        let array: Vec<serde_json::Value> = serde_json::from_str(json_content)
            .map_err(|e| ImportError::Other(format!("Failed to parse Twitter JSON: {}", e)))?;

        Ok(array)
    }

    /// Determine if this is a following or followers file
    fn detect_file_type(content: &str) -> &'static str {
        if content.contains("YTD.following") || content.contains("\"following\"") {
            "following"
        } else if content.contains("YTD.follower") || content.contains("\"follower\"") {
            "followers"
        } else {
            "unknown"
        }
    }
}

#[async_trait]
impl ImportConnector for TwitterConnector {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "twitter".to_string(),
            name: "Twitter/X".to_string(),
            description: "Import contacts from Twitter/X data export archive (following.js, followers.js)".to_string(),
            supported_extensions: vec!["js".to_string(), "json".to_string()],
            supported_mime_types: vec![
                "application/javascript".to_string(),
                "application/json".to_string(),
            ],
            format: ImportFormat::Twitter,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
            let filename_lower = filename.to_lowercase();
            // Twitter export files
            if filename_lower.contains("following") || filename_lower.contains("follower") {
                // Check file content for Twitter format
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    let content_preview = &content[..content.len().min(500)];
                    return content_preview.contains("window.YTD") ||
                           content_preview.contains("\"following\"") ||
                           content_preview.contains("\"follower\"");
                }
            }
        }
        false
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let content = std::fs::read_to_string(file_path)?;
        let file_type = Self::detect_file_type(&content);

        let array = Self::parse_twitter_js(&content)?;

        let mut rows = Vec::new();
        let mut warnings = Vec::new();
        let mut with_username = 0;

        for (idx, item) in array.iter().enumerate() {
            // Twitter export structure: { "following": { "accountId": "...", "userLink": "..." } }
            // or for followers: { "follower": { "accountId": "...", "userLink": "..." } }
            let entry = item.get("following")
                .or_else(|| item.get("follower"));

            if let Some(entry) = entry {
                let mut row = HashMap::new();

                // Extract account ID
                if let Some(account_id) = entry.get("accountId").and_then(|v| v.as_str()) {
                    row.insert("twitter_account_id".to_string(), account_id.to_string());
                }

                // Extract username from userLink
                if let Some(user_link) = entry.get("userLink").and_then(|v| v.as_str()) {
                    row.insert("twitter_url".to_string(), user_link.to_string());

                    if let Some(username) = Self::extract_twitter_username(user_link) {
                        row.insert("twitter_username".to_string(), format!("@{}", username));
                        row.insert("first_name".to_string(), format!("@{}", username));
                        with_username += 1;
                    }
                }

                // Add source metadata
                row.insert("source".to_string(), format!("twitter_{}", file_type));

                if !row.is_empty() && row.contains_key("twitter_username") {
                    rows.push(row);
                }
            } else {
                warnings.push(format!("Row {} has unexpected structure", idx + 1));
            }
        }

        let suggested_mappings = vec![
            ("first_name".to_string(), "first_name".to_string()),
            ("twitter_username".to_string(), "social_handles.twitter".to_string()),
            ("twitter_url".to_string(), "social_handles.twitter_url".to_string()),
            ("twitter_account_id".to_string(), "metadata.twitter_account_id".to_string()),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "twitter_js".to_string());
        metadata.insert("file_type".to_string(), file_type.to_string());
        metadata.insert("total_contacts".to_string(), rows.len().to_string());
        metadata.insert("contacts_with_username".to_string(), with_username.to_string());

        // Add note about limited data
        if rows.len() > 0 {
            warnings.push(format!(
                "Note: Twitter exports only contain usernames and account IDs. Display names and bios are not included. Found {} accounts.",
                rows.len()
            ));
        }

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }
}

/// Facebook Export Connector - Imports friends.json from Facebook data export
///
/// Facebook export format (friends/friends.json):
/// ```json
/// {
///   "friends_v2": [
///     {
///       "name": "John Doe",
///       "timestamp": 1234567890
///     }
///   ]
/// }
/// ```
///
/// Note: Facebook exports only contain names and timestamps - no email/phone/profile URLs.
pub struct FacebookConnector;

impl FacebookConnector {
    pub fn new() -> Self {
        Self
    }

    /// Parse a Facebook name into first/last name
    fn parse_name(name: &str) -> (String, Option<String>) {
        let parts: Vec<&str> = name.trim().split_whitespace().collect();
        match parts.len() {
            0 => (name.to_string(), None),
            1 => (parts[0].to_string(), None),
            _ => {
                let first = parts[0].to_string();
                let last = parts[1..].join(" ");
                (first, Some(last))
            }
        }
    }

    /// Convert Unix timestamp to readable date string
    fn timestamp_to_date(timestamp: i64) -> String {
        chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

#[async_trait]
impl ImportConnector for FacebookConnector {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "facebook".to_string(),
            name: "Facebook".to_string(),
            description: "Import contacts from Facebook data export (friends.json)".to_string(),
            supported_extensions: vec!["json".to_string()],
            supported_mime_types: vec!["application/json".to_string()],
            format: ImportFormat::Facebook,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
            let filename_lower = filename.to_lowercase();
            if filename_lower.contains("friends") && filename_lower.ends_with(".json") {
                // Verify it's a Facebook export format
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    let content_preview = &content[..content.len().min(500)];
                    return content_preview.contains("friends_v2") ||
                           content_preview.contains("\"name\"") && content_preview.contains("\"timestamp\"");
                }
            }
        }
        false
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let content = std::fs::read_to_string(file_path)?;

        // Parse the JSON
        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| ImportError::Other(format!("Failed to parse Facebook JSON: {}", e)))?;

        let mut rows = Vec::new();
        let mut warnings = Vec::new();

        // Facebook exports use "friends_v2" key
        let friends = json.get("friends_v2")
            .or_else(|| json.get("friends"))
            .and_then(|v| v.as_array());

        if let Some(friends) = friends {
            for (idx, friend) in friends.iter().enumerate() {
                let mut row = HashMap::new();

                // Extract name
                if let Some(name) = friend.get("name").and_then(|v| v.as_str()) {
                    let (first_name, last_name) = Self::parse_name(name);
                    row.insert("first_name".to_string(), first_name);
                    if let Some(last) = last_name {
                        row.insert("last_name".to_string(), last);
                    }
                    row.insert("full_name".to_string(), name.to_string());
                } else {
                    warnings.push(format!("Friend {} has no name, skipped", idx + 1));
                    continue;
                }

                // Extract timestamp (when you became friends)
                if let Some(timestamp) = friend.get("timestamp").and_then(|v| v.as_i64()) {
                    row.insert("friend_since".to_string(), Self::timestamp_to_date(timestamp));
                    row.insert("friend_timestamp".to_string(), timestamp.to_string());
                }

                // Add source metadata
                row.insert("source".to_string(), "facebook".to_string());

                rows.push(row);
            }
        } else {
            // Try alternative format (simple array)
            if let Some(array) = json.as_array() {
                for (idx, friend) in array.iter().enumerate() {
                    let mut row = HashMap::new();

                    if let Some(name) = friend.get("name").and_then(|v| v.as_str()) {
                        let (first_name, last_name) = Self::parse_name(name);
                        row.insert("first_name".to_string(), first_name);
                        if let Some(last) = last_name {
                            row.insert("last_name".to_string(), last);
                        }
                        row.insert("full_name".to_string(), name.to_string());
                    } else {
                        warnings.push(format!("Friend {} has no name, skipped", idx + 1));
                        continue;
                    }

                    if let Some(timestamp) = friend.get("timestamp").and_then(|v| v.as_i64()) {
                        row.insert("friend_since".to_string(), Self::timestamp_to_date(timestamp));
                    }

                    row.insert("source".to_string(), "facebook".to_string());
                    rows.push(row);
                }
            } else {
                warnings.push("Could not find friends list in JSON structure".to_string());
            }
        }

        let suggested_mappings = vec![
            ("first_name".to_string(), "first_name".to_string()),
            ("last_name".to_string(), "last_name".to_string()),
            ("friend_since".to_string(), "metadata.facebook_friend_since".to_string()),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "facebook_json".to_string());
        metadata.insert("total_contacts".to_string(), rows.len().to_string());

        // Add privacy note
        if rows.len() > 0 {
            warnings.push(format!(
                "Note: Facebook exports only include names and friendship dates. Email addresses and phone numbers are not available. Found {} friends.",
                rows.len()
            ));
        }

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }
}

/// Instagram Export Connector - Imports followers/following from Instagram data export
///
/// Instagram export format (followers_and_following/followers_1.json or following.json):
/// ```json
/// {
///   "relationships_following": [
///     {
///       "title": "username",
///       "string_list_data": [
///         {
///           "href": "https://www.instagram.com/username",
///           "value": "username",
///           "timestamp": 1234567890
///         }
///       ]
///     }
///   ]
/// }
/// ```
///
/// Or older format:
/// ```json
/// [
///   {
///     "title": "username",
///     "href": "https://www.instagram.com/username"
///   }
/// ]
/// ```
pub struct InstagramConnector;

impl InstagramConnector {
    pub fn new() -> Self {
        Self
    }

    /// Extract Instagram username from profile URL or title
    fn extract_username(href: Option<&str>, title: Option<&str>, value: Option<&str>) -> Option<String> {
        // Try value field first (most reliable)
        if let Some(v) = value {
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }

        // Try title field
        if let Some(t) = title {
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }

        // Try extracting from URL
        if let Some(url) = href {
            let url = url.trim().trim_end_matches('/');
            if let Some(pos) = url.rfind('/') {
                let username = &url[pos + 1..];
                if !username.is_empty() {
                    return Some(username.to_string());
                }
            }
        }

        None
    }

    /// Determine file type from content
    fn detect_file_type(json: &serde_json::Value) -> &'static str {
        if json.get("relationships_following").is_some() {
            "following"
        } else if json.get("relationships_followers").is_some() {
            "followers"
        } else {
            "unknown"
        }
    }
}

#[async_trait]
impl ImportConnector for InstagramConnector {
    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            id: "instagram".to_string(),
            name: "Instagram".to_string(),
            description: "Import contacts from Instagram data export (followers.json, following.json)".to_string(),
            supported_extensions: vec!["json".to_string()],
            supported_mime_types: vec!["application/json".to_string()],
            format: ImportFormat::Instagram,
            required_auth_scopes: vec![],
            version: "1.0.0".to_string(),
        }
    }

    fn can_handle(&self, file_path: &Path) -> bool {
        if let Some(filename) = file_path.file_name().and_then(|n| n.to_str()) {
            let filename_lower = filename.to_lowercase();
            if (filename_lower.contains("follower") || filename_lower.contains("following")) &&
               filename_lower.ends_with(".json") {
                // Verify it's Instagram format
                if let Ok(content) = std::fs::read_to_string(file_path) {
                    let content_preview = &content[..content.len().min(1000)];
                    return content_preview.contains("relationships_follow") ||
                           content_preview.contains("string_list_data") ||
                           (content_preview.contains("\"title\"") && content_preview.contains("instagram.com"));
                }
            }
        }
        false
    }

    async fn parse(&self, file_path: &Path) -> Result<ParseResult, ImportError> {
        let content = std::fs::read_to_string(file_path)?;

        let json: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| ImportError::Other(format!("Failed to parse Instagram JSON: {}", e)))?;

        let mut rows = Vec::new();
        let mut warnings = Vec::new();
        let file_type = Self::detect_file_type(&json);

        // Try new format first (relationships_following/relationships_followers)
        let users = json.get("relationships_following")
            .or_else(|| json.get("relationships_followers"))
            .and_then(|v| v.as_array());

        if let Some(users) = users {
            for (idx, user) in users.iter().enumerate() {
                let mut row = HashMap::new();

                // Get username from title
                let title = user.get("title").and_then(|v| v.as_str());

                // Get additional data from string_list_data
                let (href, value, timestamp) = if let Some(list_data) = user.get("string_list_data").and_then(|v| v.as_array()) {
                    if let Some(first) = list_data.first() {
                        (
                            first.get("href").and_then(|v| v.as_str()),
                            first.get("value").and_then(|v| v.as_str()),
                            first.get("timestamp").and_then(|v| v.as_i64())
                        )
                    } else {
                        (None, None, None)
                    }
                } else {
                    (None, None, None)
                };

                if let Some(username) = Self::extract_username(href, title, value) {
                    row.insert("instagram_username".to_string(), format!("@{}", username));
                    row.insert("first_name".to_string(), format!("@{}", username));

                    if let Some(url) = href {
                        row.insert("instagram_url".to_string(), url.to_string());
                    }

                    if let Some(ts) = timestamp {
                        row.insert("followed_timestamp".to_string(), ts.to_string());
                        let date = chrono::DateTime::from_timestamp(ts, 0)
                            .map(|dt| dt.format("%Y-%m-%d").to_string())
                            .unwrap_or_else(|| "Unknown".to_string());
                        row.insert("followed_date".to_string(), date);
                    }

                    row.insert("source".to_string(), format!("instagram_{}", file_type));
                    rows.push(row);
                } else {
                    warnings.push(format!("User {} has no identifiable username, skipped", idx + 1));
                }
            }
        } else {
            // Try older/simpler format (array of {title, href})
            if let Some(array) = json.as_array() {
                for (idx, user) in array.iter().enumerate() {
                    let mut row = HashMap::new();

                    let title = user.get("title").and_then(|v| v.as_str());
                    let href = user.get("href").and_then(|v| v.as_str());

                    if let Some(username) = Self::extract_username(href, title, None) {
                        row.insert("instagram_username".to_string(), format!("@{}", username));
                        row.insert("first_name".to_string(), format!("@{}", username));

                        if let Some(url) = href {
                            row.insert("instagram_url".to_string(), url.to_string());
                        }

                        row.insert("source".to_string(), "instagram".to_string());
                        rows.push(row);
                    } else {
                        warnings.push(format!("User {} has no identifiable username, skipped", idx + 1));
                    }
                }
            } else {
                warnings.push("Could not find users list in JSON structure".to_string());
            }
        }

        let suggested_mappings = vec![
            ("first_name".to_string(), "first_name".to_string()),
            ("instagram_username".to_string(), "social_handles.instagram".to_string()),
            ("instagram_url".to_string(), "social_handles.instagram_url".to_string()),
            ("followed_date".to_string(), "metadata.instagram_followed_date".to_string()),
        ];

        let mut metadata = HashMap::new();
        metadata.insert("format".to_string(), "instagram_json".to_string());
        metadata.insert("file_type".to_string(), file_type.to_string());
        metadata.insert("total_contacts".to_string(), rows.len().to_string());

        // Add note about limited data
        if rows.len() > 0 {
            warnings.push(format!(
                "Note: Instagram exports only contain usernames and profile links. Display names and bios are not included. Found {} accounts.",
                rows.len()
            ));
        }

        Ok(ParseResult {
            rows,
            suggested_mappings,
            metadata,
            warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_linkedin_metadata() {
        let connector = LinkedInConnector::new();
        let metadata = connector.metadata();
        assert_eq!(metadata.id, "linkedin");
        assert_eq!(metadata.format, ImportFormat::LinkedIn);
    }

    #[test]
    fn test_extract_linkedin_username() {
        assert_eq!(
            LinkedInConnector::extract_linkedin_username("https://www.linkedin.com/in/johndoe"),
            Some("johndoe".to_string())
        );
        assert_eq!(
            LinkedInConnector::extract_linkedin_username("https://www.linkedin.com/in/johndoe/"),
            Some("johndoe".to_string())
        );
        assert_eq!(
            LinkedInConnector::extract_linkedin_username("https://linkedin.com/in/jane-smith-123"),
            Some("jane-smith-123".to_string())
        );
        assert_eq!(
            LinkedInConnector::extract_linkedin_username("https://example.com/user"),
            None
        );
    }

    #[tokio::test]
    async fn test_linkedin_parse_csv() {
        let csv_content = r#"First Name,Last Name,Email Address,Company,Position,Connected On,URL
John,Doe,john@example.com,Acme Corp,Software Engineer,15 Jan 2023,https://www.linkedin.com/in/johndoe
Jane,Smith,,Tech Inc,Product Manager,20 Feb 2023,https://www.linkedin.com/in/janesmith
Bob,Johnson,bob@test.com,StartupXYZ,CEO,01 Mar 2023,
"#;

        let mut temp_file = NamedTempFile::with_prefix("Connections").unwrap();
        temp_file.write_all(csv_content.as_bytes()).unwrap();

        let connector = LinkedInConnector::new();

        // Test can_handle
        assert!(connector.can_handle(temp_file.path()));

        // Test parse
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 3);
        assert_eq!(result.metadata.get("contacts_with_email").unwrap(), "2");
        assert_eq!(result.metadata.get("contacts_with_linkedin_url").unwrap(), "2");

        // Check first row
        let first = &result.rows[0];
        assert_eq!(first.get("first_name"), Some(&"John".to_string()));
        assert_eq!(first.get("last_name"), Some(&"Doe".to_string()));
        assert_eq!(first.get("email"), Some(&"john@example.com".to_string()));
        assert_eq!(first.get("organization"), Some(&"Acme Corp".to_string()));
        assert_eq!(first.get("title"), Some(&"Software Engineer".to_string()));
        assert_eq!(first.get("linkedin_username"), Some(&"johndoe".to_string()));
        assert_eq!(first.get("source"), Some(&"linkedin".to_string()));
    }

    #[tokio::test]
    async fn test_linkedin_handles_missing_fields() {
        let csv_content = r#"First Name,Last Name,Company,Position,Connected On
Alice,,,Designer,01 Jan 2024
,Brown,Some Company,Manager,02 Jan 2024
"#;

        let mut temp_file = NamedTempFile::with_prefix("connections_export").unwrap();
        temp_file.write_all(csv_content.as_bytes()).unwrap();

        let connector = LinkedInConnector::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        // Both rows should be included (one has first name, one has last name)
        assert_eq!(result.rows.len(), 2);
    }

    #[test]
    fn test_extract_twitter_username() {
        assert_eq!(
            TwitterConnector::extract_twitter_username("https://twitter.com/johndoe"),
            Some("johndoe".to_string())
        );
        assert_eq!(
            TwitterConnector::extract_twitter_username("https://x.com/janesmith"),
            Some("janesmith".to_string())
        );
        assert_eq!(
            TwitterConnector::extract_twitter_username("https://twitter.com/user123/"),
            Some("user123".to_string())
        );
    }

    #[tokio::test]
    async fn test_twitter_parse_following_js() {
        let js_content = r#"window.YTD.following.part0 = [
  {
    "following" : {
      "accountId" : "123456789",
      "userLink" : "https://twitter.com/testuser1"
    }
  },
  {
    "following" : {
      "accountId" : "987654321",
      "userLink" : "https://twitter.com/testuser2"
    }
  }
]"#;

        let mut temp_file = NamedTempFile::with_prefix("following").unwrap();
        temp_file.write_all(js_content.as_bytes()).unwrap();

        let connector = TwitterConnector::new();

        // Test can_handle
        assert!(connector.can_handle(temp_file.path()));

        // Test parse
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.metadata.get("file_type").unwrap(), "following");
        assert_eq!(result.metadata.get("contacts_with_username").unwrap(), "2");

        // Check first row
        let first = &result.rows[0];
        assert_eq!(first.get("twitter_username"), Some(&"@testuser1".to_string()));
        assert_eq!(first.get("twitter_account_id"), Some(&"123456789".to_string()));
        assert_eq!(first.get("source"), Some(&"twitter_following".to_string()));
    }

    #[tokio::test]
    async fn test_twitter_parse_followers_js() {
        let js_content = r#"window.YTD.follower.part0 = [
  {
    "follower" : {
      "accountId" : "111222333",
      "userLink" : "https://twitter.com/follower1"
    }
  }
]"#;

        let mut temp_file = NamedTempFile::with_prefix("followers").unwrap();
        temp_file.write_all(js_content.as_bytes()).unwrap();

        let connector = TwitterConnector::new();
        let result = connector.parse(temp_file.path()).await.unwrap();

        assert_eq!(result.rows.len(), 1);
        assert_eq!(result.metadata.get("file_type").unwrap(), "followers");

        let first = &result.rows[0];
        assert_eq!(first.get("twitter_username"), Some(&"@follower1".to_string()));
        assert_eq!(first.get("source"), Some(&"twitter_followers".to_string()));
    }

    #[test]
    fn test_facebook_parse_name() {
        assert_eq!(FacebookConnector::parse_name("John Doe"), ("John".to_string(), Some("Doe".to_string())));
        assert_eq!(FacebookConnector::parse_name("Jane"), ("Jane".to_string(), None));
        assert_eq!(FacebookConnector::parse_name("Mary Jane Watson"), ("Mary".to_string(), Some("Jane Watson".to_string())));
    }

    #[tokio::test]
    async fn test_facebook_parse_friends_json() {
        let json_content = r#"{
  "friends_v2": [
    {
      "name": "John Doe",
      "timestamp": 1609459200
    },
    {
      "name": "Jane Smith",
      "timestamp": 1612137600
    },
    {
      "name": "Bob",
      "timestamp": 1614556800
    }
  ]
}"#;

        let mut temp_file = NamedTempFile::with_prefix("friends").unwrap();
        std::fs::write(temp_file.path(), json_content).unwrap();

        // Rename to have .json extension
        let json_path = temp_file.path().with_extension("json");
        std::fs::rename(temp_file.path(), &json_path).unwrap();

        let connector = FacebookConnector::new();

        // Test can_handle
        assert!(connector.can_handle(&json_path));

        // Test parse
        let result = connector.parse(&json_path).await.unwrap();

        assert_eq!(result.rows.len(), 3);
        assert_eq!(result.metadata.get("format").unwrap(), "facebook_json");

        // Check first row
        let first = &result.rows[0];
        assert_eq!(first.get("first_name"), Some(&"John".to_string()));
        assert_eq!(first.get("last_name"), Some(&"Doe".to_string()));
        assert_eq!(first.get("source"), Some(&"facebook".to_string()));
        assert!(first.contains_key("friend_since"));

        // Check third row (single name)
        let third = &result.rows[2];
        assert_eq!(third.get("first_name"), Some(&"Bob".to_string()));
        assert_eq!(third.get("last_name"), None);

        // Cleanup
        let _ = std::fs::remove_file(&json_path);
    }

    #[test]
    fn test_instagram_extract_username() {
        assert_eq!(
            InstagramConnector::extract_username(
                Some("https://www.instagram.com/testuser"),
                Some("testuser"),
                Some("testuser")
            ),
            Some("testuser".to_string())
        );
        assert_eq!(
            InstagramConnector::extract_username(
                None,
                Some("another_user"),
                None
            ),
            Some("another_user".to_string())
        );
        assert_eq!(
            InstagramConnector::extract_username(
                Some("https://www.instagram.com/url_user/"),
                None,
                None
            ),
            Some("url_user".to_string())
        );
    }

    #[tokio::test]
    async fn test_instagram_parse_new_format() {
        let json_content = r#"{
  "relationships_following": [
    {
      "title": "user1",
      "media_list_data": [],
      "string_list_data": [
        {
          "href": "https://www.instagram.com/user1",
          "value": "user1",
          "timestamp": 1609459200
        }
      ]
    },
    {
      "title": "user2",
      "media_list_data": [],
      "string_list_data": [
        {
          "href": "https://www.instagram.com/user2",
          "value": "user2",
          "timestamp": 1612137600
        }
      ]
    }
  ]
}"#;

        let mut temp_file = NamedTempFile::with_prefix("following").unwrap();
        std::fs::write(temp_file.path(), json_content).unwrap();

        let json_path = temp_file.path().with_extension("json");
        std::fs::rename(temp_file.path(), &json_path).unwrap();

        let connector = InstagramConnector::new();

        // Test can_handle
        assert!(connector.can_handle(&json_path));

        // Test parse
        let result = connector.parse(&json_path).await.unwrap();

        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.metadata.get("file_type").unwrap(), "following");

        let first = &result.rows[0];
        assert_eq!(first.get("instagram_username"), Some(&"@user1".to_string()));
        assert_eq!(first.get("source"), Some(&"instagram_following".to_string()));
        assert!(first.contains_key("followed_date"));

        let _ = std::fs::remove_file(&json_path);
    }

    #[tokio::test]
    async fn test_instagram_parse_simple_format() {
        let json_content = r#"[
  {
    "title": "simple_user1",
    "href": "https://www.instagram.com/simple_user1"
  },
  {
    "title": "simple_user2",
    "href": "https://www.instagram.com/simple_user2"
  }
]"#;

        let mut temp_file = NamedTempFile::with_prefix("followers").unwrap();
        std::fs::write(temp_file.path(), json_content).unwrap();

        let json_path = temp_file.path().with_extension("json");
        std::fs::rename(temp_file.path(), &json_path).unwrap();

        let connector = InstagramConnector::new();
        let result = connector.parse(&json_path).await.unwrap();

        assert_eq!(result.rows.len(), 2);

        let first = &result.rows[0];
        assert_eq!(first.get("instagram_username"), Some(&"@simple_user1".to_string()));
        assert_eq!(first.get("instagram_url"), Some(&"https://www.instagram.com/simple_user1".to_string()));

        let _ = std::fs::remove_file(&json_path);
    }
}
