#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

use commands::*;
use secure_vault::load_from_env_and_export;
use tauri::Manager;

fn main() {
    if let Err(err) = load_from_env_and_export() {
        eprintln!("Failed to load secure vault: {}", err);
    }
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Use the existing database from the main project
            // This is the database that was populated via the web interface
            let db_path = "/home/robug/Projects/sagenscontact/alpha/data/contacts.db";

            // Initialize app state with local store
            let state = tokio::runtime::Runtime::new()
                .expect("Failed to create tokio runtime")
                .block_on(AppState::new(db_path))
                .expect("Failed to initialize app state");

            app.manage(state);

            // Setup system tray (if available on platform)
            #[cfg(desktop)]
            {
                use tauri::tray::{TrayIconBuilder, TrayIconEvent};
                use tauri::menu::{Menu, MenuItem};

                let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
                let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;

                let menu = Menu::with_items(app, &[&show_i, &hide_i, &quit_i])?;

                let _tray = TrayIconBuilder::new()
                    .menu(&menu)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "hide" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.hide();
                            }
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click { .. } = event {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app)?;
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                #[cfg(not(target_os = "macos"))]
                {
                    // On Windows/Linux, hide to tray instead of closing
                    window.hide().unwrap();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_contacts,
            create_contact,
            update_contact,
            delete_contact,
            search_contacts,
            get_groups,
            create_group,
            update_group,
            delete_group,
            add_group_member,
            remove_group_member,
            get_projects,
            get_project,
            create_project,
            update_project,
            delete_project,
            add_project_contact,
            remove_project_contact,
            get_calendar_events,
            create_event,
            update_event,
            delete_event,
            get_notes,
            create_note,
            get_all_communications,
            get_communications,
            queue_communication,
            import_csv,
            import_sms_file,
            analyze_import_file,
            preview_import,
            execute_import,
            get_dashboard,
            get_settings,
            update_settings,
            sync_with_server,
            check_online,
            get_attachments,
            upload_attachment,
            download_attachment,
            delete_attachment,
            open_data_directory,
            find_duplicate_contacts,
            merge_contacts_cmd,
            check_is_sms_file,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
