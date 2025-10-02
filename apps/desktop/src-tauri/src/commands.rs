use core_domain::{Contact, Group, Project, CalendarEvent, Note, CommunicationAttempt, ProjectStatus, Attachment, AttachmentEntityType, ScanStatus};
use local_store::LocalStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{State, AppHandle};
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::Utc;
use std::path::PathBuf;

pub struct AppState {
    pub store: Arc<Mutex<LocalStore>>,
    pub sync_url: Arc<Mutex<Option<String>>>,
}

impl AppState {
    pub fn new(store_path: &str) -> Result<Self, String> {
        let store = LocalStore::new(store_path).map_err(|e| e.to_string())?;
        Ok(Self {
            store: Arc::new(Mutex::new(store)),
            sync_url: Arc::new(Mutex::new(None)),
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct DashboardData {
    contacts_count: usize,
    projects_active: usize,
    upcoming_events: Vec<CalendarEvent>,
    recent_communications: Vec<CommunicationAttempt>,
}

#[derive(Serialize, Deserialize)]
pub struct SyncResult {
    synced_contacts: usize,
    synced_groups: usize,
    synced_projects: usize,
    synced_events: usize,
    synced_notes: usize,
}

// Contact Commands
#[tauri::command]
pub async fn get_contacts(
    limit: i64,
    offset: i64,
    state: State<'_, AppState>
) -> Result<Vec<Contact>, String> {
    let store = state.store.lock().await;
    store.list_contacts(limit, offset).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_contact(
    mut contact: Contact,
    state: State<'_, AppState>
) -> Result<Contact, String> {
    let mut store = state.store.lock().await;
    if contact.id.is_empty() {
        contact.id = Uuid::new_v4().to_string();
    }
    contact.created_at = Utc::now();
    contact.updated_at = Utc::now();
    store.upsert_contact(&contact).await.map_err(|e| e.to_string())?;
    Ok(contact)
}

#[tauri::command]
pub async fn update_contact(
    id: String,
    updates: serde_json::Value,
    state: State<'_, AppState>
) -> Result<Contact, String> {
    let mut store = state.store.lock().await;
    let mut contact = store.get_contact(&id).await
        .map_err(|e| e.to_string())?
        .ok_or("Contact not found")?;

    // Apply updates
    if let Some(first_name) = updates.get("first_name").and_then(|v| v.as_str()) {
        contact.first_name = first_name.to_string();
    }
    if let Some(last_name) = updates.get("last_name").and_then(|v| v.as_str()) {
        contact.last_name = Some(last_name.to_string());
    }
    if let Some(email) = updates.get("email").and_then(|v| v.as_str()) {
        contact.email = Some(email.to_string());
    }
    if let Some(phone) = updates.get("phone").and_then(|v| v.as_str()) {
        contact.phone = Some(phone.to_string());
    }

    contact.updated_at = Utc::now();
    contact.version += 1;
    store.upsert_contact(&contact).await.map_err(|e| e.to_string())?;
    Ok(contact)
}

#[tauri::command]
pub async fn delete_contact(
    id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let mut store = state.store.lock().await;
    store.delete_contact(&id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_contacts(
    query: String,
    state: State<'_, AppState>
) -> Result<Vec<Contact>, String> {
    let store = state.store.lock().await;
    store.search_contacts(&query).await.map_err(|e| e.to_string())
}

// Group Commands
#[tauri::command]
pub async fn get_groups(state: State<'_, AppState>) -> Result<Vec<Group>, String> {
    let store = state.store.lock().await;
    store.list_groups(100, 0).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_group(
    mut group: Group,
    state: State<'_, AppState>
) -> Result<Group, String> {
    let mut store = state.store.lock().await;
    if group.id.is_empty() {
        group.id = Uuid::new_v4().to_string();
    }
    group.created_at = Utc::now();
    group.updated_at = Utc::now();
    store.upsert_group(&group).await.map_err(|e| e.to_string())?;
    Ok(group)
}

// Project Commands
#[tauri::command]
pub async fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let store = state.store.lock().await;
    store.list_projects(100, 0).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_project(
    mut project: Project,
    state: State<'_, AppState>
) -> Result<Project, String> {
    let mut store = state.store.lock().await;
    if project.id.is_empty() {
        project.id = Uuid::new_v4().to_string();
    }
    project.created_at = Utc::now();
    project.updated_at = Utc::now();
    store.upsert_project(&project).await.map_err(|e| e.to_string())?;
    Ok(project)
}

// Calendar Commands
#[tauri::command]
pub async fn get_calendar_events(
    start: Option<String>,
    end: Option<String>,
    state: State<'_, AppState>
) -> Result<Vec<CalendarEvent>, String> {
    let store = state.store.lock().await;
    store.list_calendar_events(100, 0).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_event(
    mut event: CalendarEvent,
    state: State<'_, AppState>
) -> Result<CalendarEvent, String> {
    let mut store = state.store.lock().await;
    if event.id.is_empty() {
        event.id = Uuid::new_v4().to_string();
    }
    event.created_at = Utc::now();
    event.updated_at = Utc::now();
    store.upsert_calendar_event(&event).await.map_err(|e| e.to_string())?;
    Ok(event)
}

// Note Commands
#[tauri::command]
pub async fn get_notes(
    entity_type: Option<String>,
    entity_id: Option<String>,
    state: State<'_, AppState>
) -> Result<Vec<Note>, String> {
    let store = state.store.lock().await;
    store.list_notes(100, 0).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_note(
    mut note: Note,
    state: State<'_, AppState>
) -> Result<Note, String> {
    let mut store = state.store.lock().await;
    if note.id.is_empty() {
        note.id = Uuid::new_v4().to_string();
    }
    note.created_at = Utc::now();
    note.updated_at = Utc::now();
    store.upsert_note(&note).await.map_err(|e| e.to_string())?;
    Ok(note)
}

// Communication Commands
#[tauri::command]
pub async fn queue_communication(
    mut attempt: CommunicationAttempt,
    state: State<'_, AppState>
) -> Result<CommunicationAttempt, String> {
    let mut store = state.store.lock().await;
    if attempt.id.is_empty() {
        attempt.id = Uuid::new_v4().to_string();
    }
    store.upsert_communication_attempt(&attempt).await.map_err(|e| e.to_string())?;
    Ok(attempt)
}

// Import Commands
#[tauri::command]
pub async fn import_csv(
    path: String,
    state: State<'_, AppState>
) -> Result<usize, String> {
    // For now, return 0. Full CSV import would require import_service integration
    Ok(0)
}

// Dashboard Commands
#[tauri::command]
pub async fn get_dashboard(state: State<'_, AppState>) -> Result<DashboardData, String> {
    let store = state.store.lock().await;
    let contacts = store.list_contacts(1000, 0).await.map_err(|e| e.to_string())?;
    let projects = store.list_projects(1000, 0).await.map_err(|e| e.to_string())?;
    let events = store.list_calendar_events(10, 0).await.map_err(|e| e.to_string())?;
    let comms = store.list_communication_attempts(10, 0).await.map_err(|e| e.to_string())?;

    let active_projects = projects.iter().filter(|p| {
        matches!(p.status, ProjectStatus::Active)
    }).count();

    Ok(DashboardData {
        contacts_count: contacts.len(),
        projects_active: active_projects,
        upcoming_events: events,
        recent_communications: comms,
    })
}

// Settings Commands
#[tauri::command]
pub async fn get_settings() -> Result<serde_json::Value, String> {
    // Load from config file or defaults
    Ok(serde_json::json!({
        "theme": "light",
        "notifications": true,
        "sync_enabled": true,
        "sync_url": "http://localhost:3000/api"
    }))
}

#[tauri::command]
pub async fn update_settings(settings: serde_json::Value, state: State<'_, AppState>) -> Result<(), String> {
    // Update sync URL if provided
    if let Some(sync_url) = settings.get("sync_url").and_then(|v| v.as_str()) {
        let mut url = state.sync_url.lock().await;
        *url = Some(sync_url.to_string());
    }
    Ok(())
}

// Sync Commands
#[tauri::command]
pub async fn sync_with_server(state: State<'_, AppState>) -> Result<SyncResult, String> {
    // In alpha version, we'll implement a basic sync mechanism
    // In production, this would integrate with the sync-service API
    let sync_url_opt = state.sync_url.lock().await.clone();

    if sync_url_opt.is_none() {
        return Err("Sync URL not configured".to_string());
    }

    // For now, return mock sync result
    Ok(SyncResult {
        synced_contacts: 0,
        synced_groups: 0,
        synced_projects: 0,
        synced_events: 0,
        synced_notes: 0,
    })
}

#[tauri::command]
pub async fn check_online() -> Result<bool, String> {
    // Simple connectivity check
    Ok(reqwest::get("https://www.google.com").await.is_ok())
}

// Attachment Commands
#[tauri::command]
pub async fn get_attachments(
    entity_type: String,
    entity_id: String,
    state: State<'_, AppState>
) -> Result<Vec<Attachment>, String> {
    let entity_type = match entity_type.as_str() {
        "Contact" => AttachmentEntityType::Contact,
        "Project" => AttachmentEntityType::Project,
        "Note" => AttachmentEntityType::Note,
        "CalendarEvent" => AttachmentEntityType::CalendarEvent,
        "Communication" => AttachmentEntityType::Communication,
        _ => return Err("Invalid entity type".to_string()),
    };

    let entity_uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;

    let store = state.store.lock().await;
    store.list_attachments(entity_type, entity_uuid).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn upload_attachment(
    file_path: String,
    entity_type: String,
    entity_id: String,
    uploaded_by: String,
    state: State<'_, AppState>
) -> Result<Attachment, String> {
    // Read file
    let file_data = tokio::fs::read(&file_path).await.map_err(|e| e.to_string())?;

    // Get filename
    let filename = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or("Invalid filename")?
        .to_string();

    // Detect content type
    let content_type = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .to_string();

    // Parse entity type
    let entity_type = match entity_type.as_str() {
        "Contact" => AttachmentEntityType::Contact,
        "Project" => AttachmentEntityType::Project,
        "Note" => AttachmentEntityType::Note,
        "CalendarEvent" => AttachmentEntityType::CalendarEvent,
        "Communication" => AttachmentEntityType::Communication,
        _ => return Err("Invalid entity type".to_string()),
    };

    let entity_uuid = Uuid::parse_str(&entity_id).map_err(|e| e.to_string())?;
    let uploaded_by_uuid = Uuid::parse_str(&uploaded_by).map_err(|e| e.to_string())?;

    // Calculate checksum
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let checksum = hex::encode(hasher.finalize());

    // Create storage directory
    let storage_dir = PathBuf::from("./data/attachments");
    tokio::fs::create_dir_all(&storage_dir).await.map_err(|e| e.to_string())?;

    // Generate unique storage path
    let file_id = Uuid::new_v4();
    let extension = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    let storage_filename = format!("{}.{}", file_id, extension);
    let storage_path = storage_dir.join(&storage_filename);

    // Write file
    tokio::fs::write(&storage_path, &file_data).await.map_err(|e| e.to_string())?;

    // Create attachment record
    let attachment = Attachment {
        id: Uuid::new_v4(),
        filename,
        content_type,
        size_bytes: file_data.len() as i64,
        storage_path: storage_path.to_string_lossy().to_string(),
        thumbnail_path: None,
        entity_type,
        entity_id: entity_uuid,
        uploaded_by: uploaded_by_uuid,
        checksum,
        encrypted: false,
        scan_status: ScanStatus::Pending,
        scan_details: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    };

    // Save to database
    let mut store = state.store.lock().await;
    store.upsert_attachment(&attachment).await.map_err(|e| e.to_string())?;

    Ok(attachment)
}

#[tauri::command]
pub async fn download_attachment(
    id: String,
    save_path: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let attachment_uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let store = state.store.lock().await;
    let attachment = store.get_attachment(attachment_uuid).await
        .map_err(|e| e.to_string())?
        .ok_or("Attachment not found")?;

    // Check scan status
    if attachment.scan_status == ScanStatus::Infected {
        return Err(format!("File is infected: {}", attachment.scan_details.unwrap_or_default()));
    }

    // Read file
    let file_data = tokio::fs::read(&attachment.storage_path).await.map_err(|e| e.to_string())?;

    // Verify checksum
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let checksum = hex::encode(hasher.finalize());

    if checksum != attachment.checksum {
        return Err("File integrity check failed".to_string());
    }

    // Write to save location
    tokio::fs::write(&save_path, &file_data).await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn delete_attachment(
    id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let attachment_uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    let mut store = state.store.lock().await;

    // Get attachment to find file path
    let attachment = store.get_attachment(attachment_uuid).await
        .map_err(|e| e.to_string())?
        .ok_or("Attachment not found")?;

    // Delete file from storage
    if let Err(e) = tokio::fs::remove_file(&attachment.storage_path).await {
        tracing::warn!("Failed to delete file {}: {}", attachment.storage_path, e);
        // Continue anyway - database record is more important
    }

    // Delete from database
    store.delete_attachment(attachment_uuid).await.map_err(|e| e.to_string())?;

    Ok(())
}
