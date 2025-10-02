#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

use commands::*;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize app data directory
            let app_dir = app.path_resolver()
                .app_data_dir()
                .expect("Failed to resolve app data dir");

            std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");

            let db_path = app_dir.join("sagenscontact.db");
            let db_path_str = db_path.to_str().expect("Invalid DB path");

            // Initialize app state with local store
            let state = AppState::new(db_path_str)
                .expect("Failed to initialize app state");

            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_contacts,
            create_contact,
            update_contact,
            delete_contact,
            search_contacts,
            get_groups,
            create_group,
            get_projects,
            create_project,
            get_calendar_events,
            create_event,
            get_notes,
            create_note,
            queue_communication,
            import_csv,
            get_dashboard,
            get_settings,
            update_settings,
            sync_with_server,
            check_online,
            get_attachments,
            upload_attachment,
            download_attachment,
            delete_attachment,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
