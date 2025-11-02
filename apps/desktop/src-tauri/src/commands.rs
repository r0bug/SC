use tauri::{State, AppHandle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;
use chrono::{Utc, DateTime};
use local_store::{LocalStore, ContactRepository, GroupRepository, ProjectRepository, CalendarEventRepository, NoteRepository, CommunicationRepository, SmsHistoryRepository, SmsMessage};
use core_domain::{Contact, Group, Project, CalendarEvent, Note, CommunicationAttempt, Communication, CommunicationType, CommunicationDirection, CommunicationHistoryStatus};
use ai_middleware::SegmindClient;
use import_service::{
    connectors::create_smart_registry,
    connector::ConnectorRegistry,
};
use quick_xml::events::Event;
use quick_xml::Reader as XmlReader;
use std::io::BufReader;
use std::fs::File;

pub struct AppState {
    store: LocalStore,
    ai_client: SegmindClient,
    import_registry: ConnectorRegistry,
}

impl AppState {
    pub async fn new(db_path: &str) -> Result<Self, String> {
        let store = LocalStore::new(db_path)
            .await
            .map_err(|e| format!("Failed to initialize LocalStore: {}", e))?;

        // Initialize AI client
        // Reads from SEGMIND_API_KEY environment variable
        // If not set, runs in mock mode (no real AI calls)
        let api_key = std::env::var("SEGMIND_API_KEY").ok();

        let ai_client = if api_key.is_some() {
            tracing::info!("Segmind AI enabled with API key");
            SegmindClient::new(api_key)
        } else {
            tracing::warn!("Segmind AI running in MOCK MODE (no API key found)");
            tracing::warn!("Set SEGMIND_API_KEY environment variable to enable real AI");
            SegmindClient::new(None)
        };

        // Create smart import registry with AI
        let import_registry = create_smart_registry(ai_client.clone());

        Ok(AppState {
            store,
            ai_client,
            import_registry,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub total_contacts: usize,
    pub total_projects: usize,
    pub total_groups: usize,
    pub upcoming_events: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub synced_contacts: usize,
}

// Contact Commands
#[tauri::command]
pub async fn get_contacts(
    limit: i64,
    offset: i64,
    state: State<'_, AppState>
) -> Result<Vec<Contact>, String> {
    let pool = state.store.pool();
    let repo = ContactRepository::new(pool);
    repo.list(limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_contact(
    contact: Contact,
    state: State<'_, AppState>
) -> Result<Contact, String> {
    let pool = state.store.pool();
    let repo = ContactRepository::new(pool);
    repo.create(&contact)
        .await
        .map_err(|e| e.to_string())?;
    Ok(contact)
}

#[tauri::command]
pub async fn update_contact(
    id: String,
    updates: Contact,
    state: State<'_, AppState>
) -> Result<Contact, String> {
    let pool = state.store.pool();
    let repo = ContactRepository::new(pool);
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    // Get existing contact and merge updates
    let mut contact = repo.get_by_id(uuid)
        .await
        .map_err(|e| e.to_string())?;

    // Update fields
    if !updates.first_name.is_empty() {
        contact.first_name = updates.first_name;
    }
    if let Some(last_name) = updates.last_name {
        if !last_name.is_empty() {
            contact.last_name = Some(last_name);
        }
    }
    if let Some(email) = updates.email {
        contact.email = Some(email);
    }
    if let Some(phone) = updates.phone {
        contact.phone = Some(phone);
    }

    repo.update(&contact)
        .await
        .map_err(|e| e.to_string())?;

    Ok(contact)
}

#[tauri::command]
pub async fn delete_contact(
    id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let pool = state.store.pool();
    let repo = ContactRepository::new(pool);
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.delete(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_contacts(
    query: String,
    state: State<'_, AppState>
) -> Result<Vec<Contact>, String> {
    let pool = state.store.pool();
    let repo = ContactRepository::new(pool);
    repo.search(&query)
        .await
        .map_err(|e| e.to_string())
}

// Group Commands
#[tauri::command]
pub async fn get_groups(state: State<'_, AppState>) -> Result<Vec<Group>, String> {
    let pool = state.store.pool();
    let repo = GroupRepository::new(pool);
    repo.list(100, 0)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_group(
    group: Group,
    state: State<'_, AppState>
) -> Result<Group, String> {
    let pool = state.store.pool();
    let repo = GroupRepository::new(pool);
    repo.create(&group)
        .await
        .map_err(|e| e.to_string())?;
    Ok(group)
}

#[tauri::command]
pub async fn update_group(
    id: String,
    group: Group,
    state: State<'_, AppState>
) -> Result<Group, String> {
    let pool = state.store.pool();
    let repo = GroupRepository::new(pool);
    repo.update(&group)
        .await
        .map_err(|e| e.to_string())?;
    Ok(group)
}

#[tauri::command]
pub async fn delete_group(
    id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let pool = state.store.pool();
    let repo = GroupRepository::new(pool);
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.delete(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_group_member(
    group_id: String,
    contact_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let pool = state.store.pool();
    let repo = GroupRepository::new(pool);
    let group_uuid = Uuid::parse_str(&group_id).map_err(|e| e.to_string())?;
    let contact_uuid = Uuid::parse_str(&contact_id).map_err(|e| e.to_string())?;
    repo.add_member(group_uuid, contact_uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_group_member(
    group_id: String,
    contact_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let pool = state.store.pool();
    let repo = GroupRepository::new(pool);
    let group_uuid = Uuid::parse_str(&group_id).map_err(|e| e.to_string())?;
    let contact_uuid = Uuid::parse_str(&contact_id).map_err(|e| e.to_string())?;
    repo.remove_member(group_uuid, contact_uuid)
        .await
        .map_err(|e| e.to_string())
}

// Project Commands
#[tauri::command]
pub async fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let pool = state.store.pool();
    let repo = ProjectRepository::new(pool);
    repo.list()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_project(
    id: String,
    state: State<'_, AppState>
) -> Result<Project, String> {
    let pool = state.store.pool();
    let repo = ProjectRepository::new(pool);
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.get_by_id(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_project(
    project: Project,
    state: State<'_, AppState>
) -> Result<Project, String> {
    let pool = state.store.pool();
    let repo = ProjectRepository::new(pool);
    repo.create(&project)
        .await
        .map_err(|e| e.to_string())?;
    Ok(project)
}

#[tauri::command]
pub async fn update_project(
    id: String,
    project: Project,
    state: State<'_, AppState>
) -> Result<Project, String> {
    let pool = state.store.pool();
    let repo = ProjectRepository::new(pool);
    repo.update(&project)
        .await
        .map_err(|e| e.to_string())?;
    Ok(project)
}

#[tauri::command]
pub async fn delete_project(
    id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let pool = state.store.pool();
    let repo = ProjectRepository::new(pool);
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.delete(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_project_contact(
    project_id: String,
    contact_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let pool = state.store.pool();
    let repo = ProjectRepository::new(pool);
    let proj_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let cont_uuid = Uuid::parse_str(&contact_id).map_err(|e| e.to_string())?;
    repo.add_contact(proj_uuid, cont_uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_project_contact(
    project_id: String,
    contact_id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let pool = state.store.pool();
    let repo = ProjectRepository::new(pool);
    let proj_uuid = Uuid::parse_str(&project_id).map_err(|e| e.to_string())?;
    let cont_uuid = Uuid::parse_str(&contact_id).map_err(|e| e.to_string())?;
    repo.remove_contact(proj_uuid, cont_uuid)
        .await
        .map_err(|e| e.to_string())
}

// Calendar Commands
#[tauri::command]
pub async fn get_calendar_events(
    state: State<'_, AppState>
) -> Result<Vec<CalendarEvent>, String> {
    let pool = state.store.pool();
    let repo = CalendarEventRepository::new(pool);
    repo.list(100, 0)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_event(
    event: CalendarEvent,
    state: State<'_, AppState>
) -> Result<CalendarEvent, String> {
    let pool = state.store.pool();
    let repo = CalendarEventRepository::new(pool);
    repo.create(&event)
        .await
        .map_err(|e| e.to_string())?;
    Ok(event)
}

#[tauri::command]
pub async fn update_event(
    id: String,
    event: CalendarEvent,
    state: State<'_, AppState>
) -> Result<CalendarEvent, String> {
    let pool = state.store.pool();
    let repo = CalendarEventRepository::new(pool);
    repo.update(&event)
        .await
        .map_err(|e| e.to_string())?;
    Ok(event)
}

#[tauri::command]
pub async fn delete_event(
    id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let pool = state.store.pool();
    let repo = CalendarEventRepository::new(pool);
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    repo.delete(uuid)
        .await
        .map_err(|e| e.to_string())
}

// Note Commands
#[tauri::command]
pub async fn get_notes(
    entity_type: Option<String>,
    entity_id: Option<String>,
    state: State<'_, AppState>
) -> Result<Vec<Note>, String> {
    let pool = state.store.pool();
    let repo = NoteRepository::new(pool);

    // If entity_type is "Contact" and we have a contact_id, fetch notes for that contact
    if let (Some(etype), Some(eid)) = (entity_type, entity_id) {
        if etype == "Contact" {
            let uuid = Uuid::parse_str(&eid).map_err(|e| e.to_string())?;
            return repo.list_by_contact(uuid)
                .await
                .map_err(|e| e.to_string());
        }
    }

    // For other cases, return empty for now
    Ok(vec![])
}

#[tauri::command]
pub async fn create_note(
    note: Note,
    state: State<'_, AppState>
) -> Result<Note, String> {
    let pool = state.store.pool();
    let repo = NoteRepository::new(pool);
    repo.create(&note)
        .await
        .map_err(|e| e.to_string())?;
    Ok(note)
}

// Communication Commands
#[tauri::command]
pub async fn get_all_communications(
    limit: i64,
    offset: i64,
    state: State<'_, AppState>
) -> Result<Vec<core_domain::Communication>, String> {
    let pool = state.store.pool();
    let repo = CommunicationRepository::new(pool);
    repo.list_all(limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_communications(
    contact_id: String,
    state: State<'_, AppState>
) -> Result<Vec<Communication>, String> {
    let pool = state.store.pool();
    let uuid = Uuid::parse_str(&contact_id).map_err(|e| e.to_string())?;

    // Get communications from communications table
    let comm_repo = CommunicationRepository::new(pool);
    let mut communications = comm_repo.get_communications_by_contact(uuid)
        .await
        .map_err(|e| e.to_string())?;

    // Get SMS messages from sms_history table and convert to Communication format
    let sms_repo = SmsHistoryRepository::new(pool);
    let sms_messages = sms_repo.get_by_contact(uuid)
        .await
        .map_err(|e| e.to_string())?;

    // Convert SMS messages to Communication format
    for sms in sms_messages {
        // message_type: 1=received, 2=sent, 3=draft, 4=outbox, 5=failed, 6=queued
        let direction = match sms.message_type {
            1 => CommunicationDirection::Inbound,
            2 => CommunicationDirection::Outbound,
            5 => CommunicationDirection::Outbound, // Failed sent message
            _ => CommunicationDirection::Outbound,
        };

        let status = match sms.message_type {
            5 => CommunicationHistoryStatus::Failed,
            _ => CommunicationHistoryStatus::Completed,
        };

        // Convert millisecond timestamp to DateTime
        let timestamp = DateTime::from_timestamp_millis(sms.message_date)
            .unwrap_or_else(|| Utc::now());

        communications.push(Communication {
            id: sms.id,
            contact_id: uuid,
            communication_type: CommunicationType::Sms,
            direction,
            timestamp,
            content: Some(sms.body),
            duration_seconds: None,
            phone_number: Some(sms.phone_number),
            thread_id: sms.thread_id.map(|id| id.to_string()),
            status,
            metadata: serde_json::json!({
                "imported_from": "sms_backup",
                "readable_date": sms.readable_date,
                "read_status": sms.read_status,
                "source_file": sms.source_file,
            }),
            created_at: sms.imported_at,
            updated_at: sms.imported_at,
        });
    }

    // Sort by timestamp (newest first)
    communications.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(communications)
}

#[tauri::command]
pub async fn queue_communication(
    attempt: CommunicationAttempt,
    state: State<'_, AppState>
) -> Result<(), String> {
    let pool = state.store.pool();
    let repo = CommunicationRepository::new(pool);
    repo.create(&attempt)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Import Commands

/// Response from analyze_import_file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportAnalysis {
    pub detected_format: String,
    pub confidence: f32,
    pub detected_fields: Vec<String>,
    pub suggested_mappings: HashMap<String, String>,
    pub structure_notes: String,
    pub warnings: Vec<String>,
}

/// Response from preview_import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub total_records: usize,
    pub sample_records: Vec<HashMap<String, String>>,
    pub field_mappings: HashMap<String, String>,
}

/// Analyze a file with SmartImporter - detects format and suggests mappings
#[tauri::command]
pub async fn analyze_import_file(
    file_path: String,
    state: State<'_, AppState>
) -> Result<ImportAnalysis, String> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Find appropriate connector
    let connector = state.import_registry.find_connector(&path)
        .ok_or_else(|| "No compatible import connector found for this file".to_string())?;

    // Parse the file to get analysis
    let result = connector.parse(&path)
        .await
        .map_err(|e| format!("Failed to analyze file: {}", e))?;

    // Extract analysis from metadata
    let detected_format = result.metadata
        .get("detected_format")
        .cloned()
        .unwrap_or_else(|| "Unknown".to_string());

    let confidence = result.metadata
        .get("confidence")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);

    let structure_notes = result.metadata
        .get("structure_notes")
        .cloned()
        .unwrap_or_default();

    // Convert suggested mappings to HashMap
    let suggested_mappings: HashMap<String, String> = result.suggested_mappings
        .into_iter()
        .collect();

    // Extract field names from mappings
    let detected_fields: Vec<String> = suggested_mappings.keys().cloned().collect();

    Ok(ImportAnalysis {
        detected_format,
        confidence,
        detected_fields,
        suggested_mappings,
        structure_notes,
        warnings: result.warnings,
    })
}

/// Preview import with specific mappings (shows sample data)
#[tauri::command]
pub async fn preview_import(
    file_path: String,
    field_mappings: HashMap<String, String>,
    state: State<'_, AppState>
) -> Result<ImportPreview, String> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Find appropriate connector
    let connector = state.import_registry.find_connector(&path)
        .ok_or_else(|| "No compatible import connector found for this file".to_string())?;

    // Get preview (limit to first 10 records)
    let result = connector.get_preview(&path, 10)
        .await
        .map_err(|e| format!("Failed to preview file: {}", e))?;

    Ok(ImportPreview {
        total_records: result.rows.len(),
        sample_records: result.rows,
        field_mappings,
    })
}

/// Execute import with confirmed mappings
#[tauri::command]
pub async fn execute_import(
    file_path: String,
    field_mappings: HashMap<String, String>,
    state: State<'_, AppState>
) -> Result<usize, String> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Find appropriate connector
    let connector = state.import_registry.find_connector(&path)
        .ok_or_else(|| "No compatible import connector found for this file".to_string())?;

    // Parse all records
    let result = connector.parse(&path)
        .await
        .map_err(|e| format!("Failed to parse file: {}", e))?;

    // Convert records to contacts using mappings
    let pool = state.store.pool();
    let contact_repo = ContactRepository::new(pool);

    let mut imported_count = 0;

    for record in result.rows {
        // Map fields to contact
        let first_name = field_mappings.get("first_name")
            .and_then(|key| record.get(key))
            .cloned()
            .unwrap_or_default();

        if first_name.is_empty() {
            continue; // Skip records without first_name
        }

        let last_name = field_mappings.get("last_name")
            .and_then(|key| record.get(key))
            .cloned();

        let email = field_mappings.get("email")
            .and_then(|key| record.get(key))
            .cloned();

        let phone = field_mappings.get("phone")
            .and_then(|key| record.get(key))
            .cloned();

        let organization = field_mappings.get("organization")
            .and_then(|key| record.get(key))
            .cloned();

        let title = field_mappings.get("title")
            .and_then(|key| record.get(key))
            .cloned();

        // Create contact
        let now = chrono::Utc::now();
        let contact = Contact {
            id: Uuid::new_v4(),
            first_name,
            last_name,
            email,
            phone,
            organization,
            title,
            notes: None,
            social_handles: vec![],
            tags: vec![],
            projects: vec![],
            groups: vec![],
            created_at: now,
            updated_at: now,
            created_by: Uuid::nil(), // System import
            version: 1,
            last_synced_at: None,
            metadata: serde_json::json!({"imported_via": "smart_importer"}),
        };

        // Save to database
        match contact_repo.create(&contact).await {
            Ok(_) => imported_count += 1,
            Err(e) => {
                eprintln!("Failed to import contact: {}", e);
                // Continue with next contact
            }
        }
    }

    Ok(imported_count)
}

/// Legacy CSV import (now uses SmartImporter)
#[tauri::command]
pub async fn import_csv(path: String, state: State<'_, AppState>) -> Result<usize, String> {
    // Analyze file first
    let analysis = analyze_import_file(path.clone(), state.clone()).await?;

    // Use suggested mappings for import
    execute_import(path, analysis.suggested_mappings, state).await
}

// Dashboard
#[tauri::command]
pub async fn get_dashboard(state: State<'_, AppState>) -> Result<DashboardData, String> {
    let pool = state.store.pool();

    let contact_repo = ContactRepository::new(pool);
    let project_repo = ProjectRepository::new(pool);
    let group_repo = GroupRepository::new(pool);
    let event_repo = CalendarEventRepository::new(pool);

    let contacts = contact_repo.list(10000, 0).await.map_err(|e| e.to_string())?;
    let projects = project_repo.list().await.map_err(|e| e.to_string())?;
    let groups = group_repo.list(10000, 0).await.map_err(|e| e.to_string())?;
    let events = event_repo.list(100, 0).await.map_err(|e| e.to_string())?;

    Ok(DashboardData {
        total_contacts: contacts.len(),
        total_projects: projects.len(),
        total_groups: groups.len(),
        upcoming_events: events.len(),
    })
}

// Settings
#[tauri::command]
pub async fn get_settings() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "sync_url": "http://localhost:3000/api",
        "sync_enabled": false,
        "notifications": true,
        "theme": "light"
    }))
}

#[tauri::command]
pub async fn update_settings(_settings: serde_json::Value) -> Result<(), String> {
    Ok(())
}

// Sync
#[tauri::command]
pub async fn sync_with_server() -> Result<SyncResult, String> {
    Err("Sync not configured - desktop app works offline".to_string())
}

#[tauri::command]
pub async fn check_online() -> Result<bool, String> {
    // Desktop app is offline-first
    Ok(false)
}

// Attachment stubs
#[tauri::command]
pub async fn get_attachments(_entity_type: String, _entity_id: String) -> Result<Vec<serde_json::Value>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn upload_attachment(_file_path: String, _entity_type: String, _entity_id: String) -> Result<serde_json::Value, String> {
    Err("Attachments not yet implemented".to_string())
}

#[tauri::command]
pub async fn download_attachment(_id: String) -> Result<String, String> {
    Err("Attachments not yet implemented".to_string())
}

#[tauri::command]
pub async fn delete_attachment(_id: String) -> Result<(), String> {
    Err("Attachments not yet implemented".to_string())
}

#[tauri::command]
pub async fn open_data_directory(_app: AppHandle) -> Result<String, String> {
    Ok("/home/robug/Projects/sagenscontact/alpha/data".to_string())
}

// SMS Import Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsImportResult {
    pub contacts_created: usize,
    pub contacts_found: usize,
    pub messages_imported: usize,
    pub total_phone_numbers: usize,
    pub failed: usize,
}

#[derive(Debug, Clone)]
struct ParsedSmsMessage {
    contact_name: Option<String>,
    body: String,
    date_ms: i64,
    readable_date: String,
    message_type: i32,
    thread_id: Option<i32>,
    read: i32,
    subscription_id: Option<String>,
}

/// Import SMS backup XML file - properly attaches messages to existing contacts
#[tauri::command]
pub async fn import_sms_file(
    file_path: String,
    state: State<'_, AppState>
) -> Result<SmsImportResult, String> {
    let path = PathBuf::from(&file_path);

    if !path.exists() {
        return Err(format!("File not found: {}", file_path));
    }

    // Parse SMS messages from XML
    let messages_by_phone = parse_sms_xml(&path)?;

    let total_phone_numbers = messages_by_phone.len();
    let mut contacts_created = 0;
    let mut contacts_found = 0;
    let mut messages_imported = 0;
    let mut failed = 0;

    let pool = state.store.pool();
    let contact_repo = ContactRepository::new(pool);
    let sms_repo = SmsHistoryRepository::new(pool);

    for (phone, messages) in messages_by_phone.iter() {
        // Look up existing contact by phone number
        let contact = match contact_repo.get_by_phone(phone).await {
            Ok(Some(existing)) => {
                contacts_found += 1;
                existing
            }
            Ok(None) => {
                // Create new contact
                let (first_name, last_name) = if let Some(first_msg) = messages.first() {
                    if let Some(name) = &first_msg.contact_name {
                        if name != "(Unknown)" {
                            parse_contact_name(name)
                        } else {
                            (phone.clone(), None)
                        }
                    } else {
                        (phone.clone(), None)
                    }
                } else {
                    (phone.clone(), None)
                };

                let contact = Contact {
                    id: Uuid::new_v4(),
                    first_name: first_name.clone(),
                    last_name,
                    email: None,
                    phone: Some(phone.clone()),
                    organization: None,
                    title: None,
                    notes: Some(format!(
                        "Imported from SMS backup - {} messages",
                        messages.len()
                    )),
                    social_handles: vec![],
                    tags: vec![],
                    projects: vec![],
                    groups: vec![],
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                    created_by: Uuid::nil(),
                    version: 1,
                    last_synced_at: None,
                    metadata: serde_json::json!({"import_source": "sms_backup", "message_count": messages.len()}),
                };

                match contact_repo.create(&contact).await {
                    Ok(_) => {
                        contacts_created += 1;
                        contact
                    }
                    Err(e) => {
                        eprintln!("Failed to create contact for {}: {}", phone, e);
                        failed += 1;
                        continue;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error looking up contact {}: {}", phone, e);
                failed += 1;
                continue;
            }
        };

        // Convert parsed SMS messages to SmsMessage structs
        // Use the default system user UUID for desktop app
        let system_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

        let sms_messages: Vec<SmsMessage> = messages
            .iter()
            .map(|msg| SmsMessage {
                id: Uuid::new_v4(),
                contact_id: Some(contact.id),
                phone_number: phone.clone(),
                contact_name: msg.contact_name.clone(),
                message_date: msg.date_ms,
                message_type: msg.message_type,
                subject: None,
                body: msg.body.clone(),
                readable_date: msg.readable_date.clone(),
                thread_id: msg.thread_id,
                read_status: if msg.read == 1 { 1 } else { 0 },
                subscription_id: msg.subscription_id.clone(),
                imported_at: Utc::now(),
                imported_by: system_user_id.to_string(),
                source_file: Some(file_path.clone()),
            })
            .collect();

        // Batch insert SMS messages
        match sms_repo.batch_create(&sms_messages).await {
            Ok(count) => {
                messages_imported += count;
            }
            Err(e) => {
                eprintln!("Failed to import SMS messages for {}: {}", phone, e);
                failed += 1;
            }
        }
    }

    Ok(SmsImportResult {
        contacts_created,
        contacts_found,
        messages_imported,
        total_phone_numbers,
        failed,
    })
}

#[tauri::command]
pub fn check_is_sms_file(file_path: String) -> Result<bool, String> {
    use std::io::Read;

    let path = PathBuf::from(&file_path);
    let mut file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;

    // Read only the first 4KB to check for SMS tags
    let mut buffer = vec![0u8; 4096];
    let bytes_read = file.read(&mut buffer).map_err(|e| format!("Failed to read file: {}", e))?;

    // Convert to string (lossy to handle any encoding issues)
    let content = String::from_utf8_lossy(&buffer[..bytes_read]);

    // Check for SMS backup XML markers
    Ok(content.contains("<smses") || content.contains("<sms "))
}

fn parse_sms_xml(path: &PathBuf) -> Result<HashMap<String, Vec<ParsedSmsMessage>>, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let file = BufReader::new(file);
    let mut reader = XmlReader::from_reader(file);

    let mut messages_by_phone: HashMap<String, Vec<ParsedSmsMessage>> = HashMap::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(e)) if e.name().as_ref() == b"sms" => {
                let mut address: Option<String> = None;
                let mut date_ms: Option<i64> = None;
                let mut contact_name: Option<String> = None;
                let mut body = String::new();
                let mut msg_type = 1;
                let mut thread_id: Option<i32> = None;
                let mut read = 1;
                let mut subscription_id: Option<String> = None;

                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"address" => {
                            address = attr.unescape_value().ok().map(|v| v.to_string());
                        }
                        b"date" => {
                            if let Ok(ms_str) = attr.unescape_value() {
                                date_ms = ms_str.parse::<i64>().ok();
                            }
                        }
                        b"contact_name" => {
                            contact_name = attr.unescape_value().ok().map(|v| v.to_string());
                        }
                        b"body" => {
                            body = attr.unescape_value().ok().map(|v| v.to_string()).unwrap_or_default();
                        }
                        b"type" => {
                            if let Ok(type_str) = attr.unescape_value() {
                                msg_type = type_str.parse::<i32>().unwrap_or(1);
                            }
                        }
                        b"thread_id" => {
                            if let Ok(thread_str) = attr.unescape_value() {
                                thread_id = thread_str.parse::<i32>().ok();
                            }
                        }
                        b"read" => {
                            if let Ok(read_str) = attr.unescape_value() {
                                read = read_str.parse::<i32>().unwrap_or(1);
                            }
                        }
                        b"sub_id" | b"subscription_id" => {
                            subscription_id = attr.unescape_value().ok().map(|v| v.to_string());
                        }
                        _ => {}
                    }
                }

                if let (Some(addr), Some(ms)) = (address, date_ms) {
                    let phone = normalize_phone_number(&addr);

                    let readable_date = DateTime::from_timestamp_millis(ms)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let message = ParsedSmsMessage {
                        contact_name,
                        body,
                        date_ms: ms,
                        readable_date,
                        message_type: msg_type,
                        thread_id,
                        read,
                        subscription_id,
                    };

                    messages_by_phone
                        .entry(phone)
                        .or_insert_with(Vec::new)
                        .push(message);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("XML parsing error at position {}: {}", reader.buffer_position(), e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(messages_by_phone)
}

fn normalize_phone_number(phone: &str) -> String {
    let phone = phone.replace(&[' ', '-', '(', ')', '.'][..], "");

    if phone.starts_with('+') {
        return phone;
    }

    if phone.chars().all(|c| c.is_numeric()) && phone.len() < 7 {
        return phone;
    }

    if phone.chars().all(|c| c.is_numeric()) && phone.len() == 10 {
        return format!("+1{}", phone);
    }

    phone
}

fn parse_contact_name(name: &str) -> (String, Option<String>) {
    let parts: Vec<&str> = name.split_whitespace().collect();
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

// Duplicate contact management commands

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub phone: String,
    pub contacts: Vec<ContactData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactData {
    pub id: String,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub has_notes: bool,
    pub tags_count: usize,
    pub projects_count: usize,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeStats {
    pub social_handles_merged: usize,
    pub tags_merged: usize,
    pub projects_merged: usize,
    pub groups_merged: usize,
    pub sms_messages_relinked: usize,
    pub communications_relinked: usize,
    pub notes_relinked: usize,
    pub ai_suggestions_relinked: usize,
    pub events_relinked: usize,
}

#[tauri::command]
pub async fn find_duplicate_contacts(
    state: State<'_, AppState>
) -> Result<Vec<DuplicateGroup>, String> {
    let pool = state.store.pool();
    let contact_repo = ContactRepository::new(pool);

    let duplicates = contact_repo.find_duplicates_by_phone()
        .await
        .map_err(|e| e.to_string())?;

    // Convert to serializable format
    let mut result: Vec<DuplicateGroup> = Vec::new();

    for (phone, contacts) in duplicates {
        let contact_data: Vec<ContactData> = contacts.into_iter().map(|c| ContactData {
            id: c.id.to_string(),
            first_name: c.first_name.clone(),
            last_name: c.last_name.clone(),
            email: c.email.clone(),
            phone: c.phone.clone(),
            has_notes: c.notes.is_some(),
            tags_count: c.tags.len(),
            projects_count: c.projects.len(),
            created_at: c.created_at.to_rfc3339(),
        }).collect();

        result.push(DuplicateGroup {
            phone,
            contacts: contact_data,
        });
    }

    // Sort by number of duplicates descending
    result.sort_by(|a, b| b.contacts.len().cmp(&a.contacts.len()));

    Ok(result)
}

#[tauri::command]
pub async fn merge_contacts_cmd(
    keep_id: String,
    merge_id: String,
    state: State<'_, AppState>
) -> Result<MergeStats, String> {
    let pool = state.store.pool();
    let contact_repo = ContactRepository::new(pool);

    let keep_uuid = Uuid::parse_str(&keep_id)
        .map_err(|e| format!("Invalid keep_id: {}", e))?;
    let merge_uuid = Uuid::parse_str(&merge_id)
        .map_err(|e| format!("Invalid merge_id: {}", e))?;

    let stats = contact_repo.merge_contacts(keep_uuid, merge_uuid)
        .await
        .map_err(|e| e.to_string())?;

    Ok(MergeStats {
        social_handles_merged: stats.social_handles_merged,
        tags_merged: stats.tags_merged,
        projects_merged: stats.projects_merged,
        groups_merged: stats.groups_merged,
        sms_messages_relinked: stats.sms_messages_relinked,
        communications_relinked: stats.communications_relinked,
        notes_relinked: stats.notes_relinked,
        ai_suggestions_relinked: stats.ai_suggestions_relinked,
        events_relinked: stats.events_relinked,
    })
}
