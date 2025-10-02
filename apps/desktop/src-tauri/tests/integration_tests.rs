use sagenscontact_desktop::commands::*;
use core_domain::*;

#[tokio::test]
async fn test_app_state_initialization() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let state = AppState::new(db_path_str);
    assert!(state.is_ok(), "App state should initialize successfully");
}

#[tokio::test]
async fn test_contact_crud() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let state = AppState::new(db_path_str).unwrap();

    // Create contact
    let contact = Contact {
        id: "test-contact-1".to_string(),
        first_name: "John".to_string(),
        last_name: Some("Doe".to_string()),
        email: Some("john@example.com".to_string()),
        phone: Some("555-0123".to_string()),
        organization: None,
        title: None,
        notes: None,
        social_handles: vec![],
        tags: vec!["test".to_string()],
        projects: vec![],
        groups: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: "test-user".to_string(),
        version: 1,
        last_synced_at: None,
        metadata: serde_json::json!({}),
    };

    // Test create
    let created = create_contact(contact.clone(), tauri::State::new(&state))
        .await
        .expect("Should create contact");
    assert_eq!(created.first_name, "John");

    // Test list
    let contacts = get_contacts(100, 0, tauri::State::new(&state))
        .await
        .expect("Should list contacts");
    assert_eq!(contacts.len(), 1);

    // Test search
    let search_results = search_contacts("John".to_string(), tauri::State::new(&state))
        .await
        .expect("Should search contacts");
    assert!(search_results.len() > 0);

    // Test delete
    delete_contact("test-contact-1".to_string(), tauri::State::new(&state))
        .await
        .expect("Should delete contact");

    let contacts_after = get_contacts(100, 0, tauri::State::new(&state))
        .await
        .expect("Should list contacts");
    assert_eq!(contacts_after.len(), 0);
}

#[tokio::test]
async fn test_group_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let state = AppState::new(db_path_str).unwrap();

    let group = Group {
        id: "test-group-1".to_string(),
        name: "Test Group".to_string(),
        description: Some("A test group".to_string()),
        members: vec![],
        tags: vec![],
        rules: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: "test-user".to_string(),
        version: 1,
        last_synced_at: None,
    };

    // Create group
    let created = create_group(group, tauri::State::new(&state))
        .await
        .expect("Should create group");
    assert_eq!(created.name, "Test Group");

    // List groups
    let groups = get_groups(tauri::State::new(&state))
        .await
        .expect("Should list groups");
    assert_eq!(groups.len(), 1);
}

#[tokio::test]
async fn test_project_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let state = AppState::new(db_path_str).unwrap();

    let project = Project {
        id: "test-project-1".to_string(),
        name: "Test Project".to_string(),
        description: Some("A test project".to_string()),
        status: ProjectStatus::Active,
        contacts: vec![],
        tags: vec![],
        attachment_ids: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: "test-user".to_string(),
        version: 1,
        last_synced_at: None,
    };

    // Create project
    let created = create_project(project, tauri::State::new(&state))
        .await
        .expect("Should create project");
    assert_eq!(created.name, "Test Project");

    // List projects
    let projects = get_projects(tauri::State::new(&state))
        .await
        .expect("Should list projects");
    assert_eq!(projects.len(), 1);
}

#[tokio::test]
async fn test_note_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let state = AppState::new(db_path_str).unwrap();

    let note = Note {
        id: "test-note-1".to_string(),
        title: Some("Test Note".to_string()),
        content: "This is a test note".to_string(),
        contact_id: None,
        project_id: None,
        attachments: vec![],
        tags: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: "test-user".to_string(),
        version: 1,
        last_synced_at: None,
    };

    // Create note
    let created = create_note(note, tauri::State::new(&state))
        .await
        .expect("Should create note");
    assert_eq!(created.title.unwrap(), "Test Note");

    // List notes
    let notes = get_notes(None, None, tauri::State::new(&state))
        .await
        .expect("Should list notes");
    assert_eq!(notes.len(), 1);
}

#[tokio::test]
async fn test_dashboard_data() {
    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();

    let state = AppState::new(db_path_str).unwrap();

    // Create some test data
    let contact = Contact {
        id: "test-contact-1".to_string(),
        first_name: "John".to_string(),
        last_name: Some("Doe".to_string()),
        email: None,
        phone: None,
        organization: None,
        title: None,
        notes: None,
        social_handles: vec![],
        tags: vec![],
        projects: vec![],
        groups: vec![],
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        created_by: "test-user".to_string(),
        version: 1,
        last_synced_at: None,
        metadata: serde_json::json!({}),
    };

    create_contact(contact, tauri::State::new(&state))
        .await
        .expect("Should create contact");

    // Get dashboard
    let dashboard = get_dashboard(tauri::State::new(&state))
        .await
        .expect("Should get dashboard");

    assert_eq!(dashboard.contacts_count, 1);
}

#[tokio::test]
async fn test_settings_operations() {
    // Get settings
    let settings = get_settings()
        .await
        .expect("Should get settings");

    assert!(settings.get("theme").is_some());
    assert!(settings.get("notifications").is_some());

    // Update settings
    let new_settings = serde_json::json!({
        "theme": "dark",
        "notifications": false,
        "sync_url": "https://example.com/api"
    });

    let temp_dir = tempfile::tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db_path_str = db_path.to_str().unwrap();
    let state = AppState::new(db_path_str).unwrap();

    update_settings(new_settings, tauri::State::new(&state))
        .await
        .expect("Should update settings");
}
