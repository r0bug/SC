use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub organization: Option<String>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub social_handles: Vec<SocialHandle>,
    pub tags: Vec<Uuid>,
    pub projects: Vec<Uuid>,
    pub groups: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub version: i32,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialHandle {
    pub platform: String,
    pub handle: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: ProjectStatus,
    pub contacts: Vec<Uuid>,
    pub tags: Vec<Uuid>,
    pub attachment_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub version: i32,
    pub last_synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectStatus {
    Active,
    Completed,
    Archived,
    OnHold,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub all_day: bool,
    pub contacts: Vec<Uuid>,
    pub location: Option<String>,
    pub recurrence: Option<RecurrenceRule>,
    pub reminders: Vec<Reminder>,
    pub attachment_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrenceRule {
    pub frequency: RecurrenceFrequency,
    pub interval: i32,
    pub count: Option<i32>,
    pub until: Option<DateTime<Utc>>,
    pub by_day: Vec<Weekday>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub id: Uuid,
    pub minutes_before: i32,
    pub method: ReminderMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReminderMethod {
    Notification,
    Email,
    SMS,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: Uuid,
    pub contact_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub title: String,
    pub content: String,
    pub attachment_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub version: i32,
    pub last_synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
    pub thumbnail_path: Option<String>,
    pub entity_type: AttachmentEntityType,
    pub entity_id: Uuid,
    pub uploaded_by: Uuid,
    pub checksum: String,             // SHA256 hash for integrity verification
    pub encrypted: bool,              // Whether file is encrypted at rest
    pub scan_status: ScanStatus,      // Virus scan result
    pub scan_details: Option<String>, // Details if scan failed
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScanStatus {
    Pending,  // Not yet scanned
    Clean,    // Passed virus scan
    Infected, // Failed virus scan
    Error,    // Scan error occurred
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentEntityType {
    Contact,
    Project,
    Note,
    CalendarEvent,
    Communication,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationAttempt {
    pub id: Uuid,
    pub contact_id: Uuid,
    pub method: CommunicationMethod,
    pub subject: Option<String>,
    pub message: String,
    pub status: CommunicationStatus,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub attempted_at: Option<DateTime<Utc>>,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationMethod {
    Email,
    SMS,
    Social { platform: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationStatus {
    Pending,
    Sent,
    Failed { reason: String },
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareInvite {
    pub id: Uuid,
    pub entity_type: ShareEntityType,
    pub entity_id: Uuid,
    pub shared_by: Uuid,
    pub shared_with_email: String,
    pub shared_with_user: Option<Uuid>,
    pub permissions: Vec<Permission>,
    pub accepted: bool,
    pub accepted_at: Option<DateTime<Utc>>,
    pub revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ShareEntityType {
    Contact,
    Project,
    Note,
    CalendarEvent,
    Group,
    Concept,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Share,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAcl {
    pub id: Uuid,
    pub entity_type: ShareEntityType,
    pub entity_id: Uuid,
    pub owner_id: Uuid,
    pub grants: Vec<AclGrant>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclGrant {
    pub user_id: Uuid,
    pub permissions: Vec<Permission>,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub entity_type: ShareEntityType,
    pub entity_id: Uuid,
    pub action: AuditAction,
    pub user_id: Uuid,
    pub changes: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    Share,
    Unshare,
    AcceptShare,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResolution {
    pub id: Uuid,
    pub entity_type: ShareEntityType,
    pub entity_id: Uuid,
    pub local_version: i32,
    pub remote_version: i32,
    pub local_updated_at: DateTime<Utc>,
    pub remote_updated_at: DateTime<Utc>,
    pub local_changes: serde_json::Value,
    pub remote_changes: serde_json::Value,
    pub resolution_strategy: ResolutionStrategy,
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResolutionStrategy {
    LocalWins,
    RemoteWins,
    Manual,
    Merge,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub api_token: Option<String>,
    pub email_verified: bool,
    pub active: bool,
    pub preferences: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub contact_ids: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub related_contacts: Vec<Uuid>,
    pub related_projects: Vec<Uuid>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub query: String,
    pub filters: serde_json::Value,
    pub result_count: i32,
    pub result_ids: Vec<Uuid>,
    pub clicked_result_id: Option<Uuid>,
    pub privacy_mode: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInsight {
    pub id: Uuid,
    pub entity_type: AiInsightEntityType,
    pub entity_id: Uuid,
    pub insight_type: AiInsightType,
    pub content: String,
    pub confidence: f32,
    pub prompt_template: String,
    pub response_cached: bool,
    pub feedback: Option<AiInsightFeedback>,
    pub applied: bool,
    pub applied_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiInsightEntityType {
    Contact,
    Project,
    Note,
    Communication,
    CalendarEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiInsightType {
    TagSuggestion,
    ChannelRecommendation,
    NextAction,
    RelationshipStrength,
    ContentSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInsightFeedback {
    pub helpful: bool,
    pub comment: Option<String>,
    pub submitted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiInteraction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub interaction_type: String,
    pub prompt: String,
    pub response: String,
    pub confidence: f32,
    pub model: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub feedback_helpful: Option<bool>,
    pub feedback_applied: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub feedback_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestion {
    pub id: Uuid,
    pub contact_id: Option<Uuid>,
    pub suggestion_type: String,
    pub content: String,
    pub confidence: f32,
    pub applied: bool,
    pub created_at: DateTime<Utc>,
}

/// Historical communication record (SMS, calls, emails that already occurred)
/// Different from CommunicationAttempt which is for outgoing scheduled communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Communication {
    pub id: Uuid,
    pub contact_id: Uuid,
    pub communication_type: CommunicationType,
    pub direction: CommunicationDirection,
    pub timestamp: DateTime<Utc>,
    pub content: Option<String>, // Message body for SMS/email, description for calls
    pub duration_seconds: Option<i32>, // For calls
    pub phone_number: Option<String>, // Normalized phone number for linking
    pub thread_id: Option<String>, // Thread/conversation identifier for grouping
    pub status: CommunicationHistoryStatus,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationType {
    Sms,
    Call,
    Email,
    VideoCall,
    VoiceMail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationDirection {
    Inbound,
    Outbound,
    Missed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationHistoryStatus {
    Completed,
    Failed,
    Blocked,
    Rejected,
}
