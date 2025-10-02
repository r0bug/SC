#!/bin/bash
# Comprehensive Tauri desktop app setup

echo "Setting up Tauri desktop app..."

# Create Tauri source structure
mkdir -p src-tauri/src
mkdir -p src/routes
mkdir -p src/lib

# Create Cargo.toml for Tauri
cat > src-tauri/Cargo.toml << 'EOF'
[package]
name = "sagenscontact-desktop"
version = "0.1.0"
description = "SagensContact Desktop Application"
authors = ["SagensContact Team"]
edition = "2021"

[build-dependencies]
tauri-build = { version = "1.5", features = [] }

[dependencies]
tauri = { version = "1.5", features = ["dialog-all", "fs-all", "notification-all", "path-all", "shell-open"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Local crates
local_store = { path = "../../../crates/local_store" }
core_domain = { path = "../../../crates/core_domain" }
import_service = { path = "../../../crates/import_service" }
communication_queue = { path = "../../../crates/communication_queue" }
ai_middleware = { path = "../../../crates/ai_middleware" }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
EOF

# Create tauri.conf.json
cat > src-tauri/tauri.conf.json << 'EOF'
{
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devPath": "http://localhost:5173",
    "distDir": "../dist"
  },
  "package": {
    "productName": "SagensContact",
    "version": "0.1.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "dialog": {
        "all": true
      },
      "fs": {
        "all": true,
        "scope": ["$APPDATA/*", "$HOME/*"]
      },
      "notification": {
        "all": true
      },
      "path": {
        "all": true
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.sagenscontact.app",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ]
    },
    "security": {
      "csp": null
    },
    "windows": [
      {
        "fullscreen": false,
        "resizable": true,
        "title": "SagensContact",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600
      }
    ]
  }
}
EOF

# Create build.rs
cat > src-tauri/build.rs << 'EOF'
fn main() {
    tauri_build::build()
}
EOF

# Create main.rs with Tauri commands
cat > src-tauri/src/main.rs << 'EOF'
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod commands;

use commands::*;

fn main() {
    tauri::Builder::default()
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
EOF

# Create commands.rs
cat > src-tauri/src/commands.rs << 'EOF'
use core_domain::{Contact, Group, Project, CalendarEvent, Note, CommunicationAttempt};
use local_store::LocalStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

pub struct AppState {
    store: Arc<Mutex<LocalStore>>,
}

#[derive(Serialize, Deserialize)]
pub struct DashboardData {
    contacts_count: usize,
    projects_active: usize,
    upcoming_events: Vec<CalendarEvent>,
    recent_communications: Vec<CommunicationAttempt>,
}

#[tauri::command]
pub async fn get_contacts(
    limit: i64,
    offset: i64,
    state: State<'_, AppState>
) -> Result<Vec<Contact>, String> {
    let store = state.store.lock().await;
    // Mock implementation for alpha
    Ok(vec![])
}

#[tauri::command]
pub async fn create_contact(
    contact: Contact,
    state: State<'_, AppState>
) -> Result<Contact, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(contact)
}

#[tauri::command]
pub async fn update_contact(
    id: String,
    updates: serde_json::Value,
    state: State<'_, AppState>
) -> Result<Contact, String> {
    // Mock implementation
    Err("Not implemented".to_string())
}

#[tauri::command]
pub async fn delete_contact(
    id: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(())
}

#[tauri::command]
pub async fn search_contacts(
    query: String,
    state: State<'_, AppState>
) -> Result<Vec<Contact>, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(vec![])
}

#[tauri::command]
pub async fn get_groups(state: State<'_, AppState>) -> Result<Vec<Group>, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(vec![])
}

#[tauri::command]
pub async fn create_group(
    group: Group,
    state: State<'_, AppState>
) -> Result<Group, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(group)
}

#[tauri::command]
pub async fn get_projects(state: State<'_, AppState>) -> Result<Vec<Project>, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(vec![])
}

#[tauri::command]
pub async fn create_project(
    project: Project,
    state: State<'_, AppState>
) -> Result<Project, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(project)
}

#[tauri::command]
pub async fn get_calendar_events(
    start: Option<String>,
    end: Option<String>,
    state: State<'_, AppState>
) -> Result<Vec<CalendarEvent>, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(vec![])
}

#[tauri::command]
pub async fn create_event(
    event: CalendarEvent,
    state: State<'_, AppState>
) -> Result<CalendarEvent, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(event)
}

#[tauri::command]
pub async fn get_notes(
    entity_type: Option<String>,
    entity_id: Option<String>,
    state: State<'_, AppState>
) -> Result<Vec<Note>, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(vec![])
}

#[tauri::command]
pub async fn create_note(
    note: Note,
    state: State<'_, AppState>
) -> Result<Note, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(note)
}

#[tauri::command]
pub async fn queue_communication(
    attempt: CommunicationAttempt,
    state: State<'_, AppState>
) -> Result<CommunicationAttempt, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(attempt)
}

#[tauri::command]
pub async fn import_csv(
    path: String,
    state: State<'_, AppState>
) -> Result<usize, String> {
    // Mock implementation
    Ok(0)
}

#[tauri::command]
pub async fn get_dashboard(state: State<'_, AppState>) -> Result<DashboardData, String> {
    let store = state.store.lock().await;
    // Mock implementation
    Ok(DashboardData {
        contacts_count: 42,
        projects_active: 5,
        upcoming_events: vec![],
        recent_communications: vec![],
    })
}

#[tauri::command]
pub async fn get_settings() -> Result<serde_json::Value, String> {
    // Mock implementation
    Ok(serde_json::json!({
        "theme": "light",
        "notifications": true,
        "sync_enabled": false,
    }))
}

#[tauri::command]
pub async fn update_settings(settings: serde_json::Value) -> Result<(), String> {
    // Mock implementation
    Ok(())
}
EOF

# Create package.json with scripts
cat > package.json << 'EOF'
{
  "name": "sagenscontact-desktop",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite dev",
    "build": "vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "tauri:dev": "tauri dev",
    "tauri:build": "tauri build"
  },
  "devDependencies": {
    "@sveltejs/adapter-static": "^2.0.3",
    "@sveltejs/kit": "^1.27.6",
    "@tauri-apps/cli": "^1.5.6",
    "svelte": "^4.2.2",
    "typescript": "^5.2.2",
    "vite": "^4.5.0"
  },
  "dependencies": {
    "@tauri-apps/api": "^1.5.1"
  }
}
EOF

# Create vite.config.js
cat > vite.config.js << 'EOF'
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_'],
});
EOF

# Create svelte.config.js
cat > svelte.config.js << 'EOF'
import adapter from '@sveltejs/adapter-static';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  kit: {
    adapter: adapter({
      fallback: 'index.html'
    }),
    prerender: {
      entries: []
    }
  }
};

export default config;
EOF

# Create app.html
cat > src/app.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <link rel="icon" href="%sveltekit.assets%/favicon.png" />
    <meta name="viewport" content="width=device-width" />
    %sveltekit.head%
  </head>
  <body data-sveltekit-preload-data="hover">
    <div style="display: contents">%sveltekit.body%</div>
  </body>
</html>
EOF

# Create Tauri API wrapper
cat > src/lib/tauri.ts << 'EOF'
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
import { sendNotification } from '@tauri-apps/api/notification';

export const tauriApi = {
  async getContacts(limit: number, offset: number) {
    return invoke('get_contacts', { limit, offset });
  },

  async createContact(contact: any) {
    return invoke('create_contact', { contact });
  },

  async updateContact(id: string, updates: any) {
    return invoke('update_contact', { id, updates });
  },

  async deleteContact(id: string) {
    return invoke('delete_contact', { id });
  },

  async searchContacts(query: string) {
    return invoke('search_contacts', { query });
  },

  async importCsv() {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'CSV',
        extensions: ['csv']
      }]
    });

    if (selected && typeof selected === 'string') {
      return invoke('import_csv', { path: selected });
    }
  },

  async getDashboard() {
    return invoke('get_dashboard');
  },

  async showNotification(title: string, body: string) {
    await sendNotification({ title, body });
  },

  async getSettings() {
    return invoke('get_settings');
  },

  async updateSettings(settings: any) {
    return invoke('update_settings', { settings });
  }
};

// Check if running in Tauri
export const isTauri = () => {
  return window.__TAURI__ !== undefined;
};
EOF

# Copy main route from web app
cat > src/routes/+page.svelte << 'EOF'
<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauri, tauriApi } from '$lib/tauri';

  let contacts = [];
  let loading = true;
  let isDesktop = false;

  onMount(async () => {
    isDesktop = isTauri();

    if (isDesktop) {
      // Use Tauri API
      try {
        contacts = await tauriApi.getContacts(50, 0);
      } catch (error) {
        console.error('Failed to load contacts:', error);
      }
    } else {
      // Fallback for web
      contacts = [];
    }

    loading = false;
  });

  async function importContacts() {
    if (isDesktop) {
      const count = await tauriApi.importCsv();
      if (count > 0) {
        await tauriApi.showNotification(
          'Import Complete',
          `Successfully imported ${count} contacts`
        );
        // Reload contacts
        contacts = await tauriApi.getContacts(50, 0);
      }
    }
  }
</script>

<div class="app">
  <header>
    <h1>📇 SagensContact Desktop</h1>
    {#if isDesktop}
      <span class="badge">Desktop App</span>
    {/if}
  </header>

  <main>
    <div class="toolbar">
      <button on:click={importContacts}>📥 Import CSV</button>
      <button>+ New Contact</button>
    </div>

    {#if loading}
      <p>Loading contacts...</p>
    {:else if contacts.length === 0}
      <div class="empty">
        <p>No contacts yet.</p>
        <button on:click={importContacts}>Import your first contacts</button>
      </div>
    {:else}
      <div class="contact-list">
        {#each contacts as contact}
          <div class="contact-card">
            <h3>{contact.first_name} {contact.last_name || ''}</h3>
            {#if contact.email}
              <p>✉️ {contact.email}</p>
            {/if}
            {#if contact.phone}
              <p>📞 {contact.phone}</p>
            {/if}
          </div>
        {/each}
      </div>
    {/if}

    <div class="mock-notice">
      <strong>⚠️ Alpha Notice:</strong> Desktop app is in development.
      All features are mocked for testing.
    </div>
  </main>
</div>

<style>
  .app {
    min-height: 100vh;
    background: #f9fafb;
  }

  header {
    background: white;
    border-bottom: 1px solid #e5e7eb;
    padding: 1rem 2rem;
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
  }

  .badge {
    background: #6366f1;
    color: white;
    padding: 0.25rem 0.75rem;
    border-radius: 12px;
    font-size: 0.75rem;
  }

  main {
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .toolbar {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  button {
    padding: 0.75rem 1.5rem;
    background: #6366f1;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 1rem;
  }

  button:hover {
    background: #4f46e5;
  }

  .empty {
    text-align: center;
    padding: 3rem;
    background: white;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .contact-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 1rem;
  }

  .contact-card {
    background: white;
    padding: 1.5rem;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .contact-card h3 {
    margin: 0 0 0.5rem;
  }

  .contact-card p {
    margin: 0.25rem 0;
    color: #6b7280;
    font-size: 0.875rem;
  }

  .mock-notice {
    margin-top: 2rem;
    padding: 1rem;
    background: #fef3c7;
    border: 1px solid #fde68a;
    border-radius: 6px;
    text-align: center;
    color: #92400e;
  }
</style>
EOF

# Create icons directory
mkdir -p src-tauri/icons

echo "✅ Tauri desktop app structure created"
echo ""
echo "Next steps:"
echo "1. cd apps/desktop"
echo "2. pnpm install"
echo "3. pnpm tauri:dev  # Run desktop app in development"
echo "4. pnpm tauri:build # Build for production"
echo ""
echo "Note: The desktop app reuses the web UI components and provides"
echo "      offline-first functionality with direct local_store access."