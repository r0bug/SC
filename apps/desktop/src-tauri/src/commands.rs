use tauri::{State, AppHandle};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use local_store::{LocalStore, ContactRepository, GroupRepository, ProjectRepository, CalendarEventRepository, NoteRepository, CommunicationRepository};
use core_domain::{Contact, Group, Project, CalendarEvent, Note, CommunicationAttempt};

pub struct AppState {
    store: LocalStore,
}

impl AppState {
    pub async fn new(db_path: &str) -> Result<Self, String> {
        let store = LocalStore::new(db_path)
            .await
            .map_err(|e| format!("Failed to initialize LocalStore: {}", e))?;

        Ok(AppState {
            store,
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
) -> Result<Vec<core_domain::Communication>, String> {
    let pool = state.store.pool();
    let repo = CommunicationRepository::new(pool);
    let uuid = Uuid::parse_str(&contact_id).map_err(|e| e.to_string())?;
    repo.get_communications_by_contact(uuid)
        .await
        .map_err(|e| e.to_string())
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
#[tauri::command]
pub async fn import_csv(_path: String, _state: State<'_, AppState>) -> Result<usize, String> {
    // TODO: Implement CSV import
    Err("CSV import not yet implemented in desktop app".to_string())
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
