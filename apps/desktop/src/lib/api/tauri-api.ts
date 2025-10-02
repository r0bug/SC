import { invoke } from '@tauri-apps/api/tauri';
import type { Contact, Group, Project, CalendarEvent, Note, CommunicationAttempt } from './types';

export class TauriApiClient {
  // Contacts
  async getContacts(limit: number = 100, offset: number = 0): Promise<Contact[]> {
    return invoke('get_contacts', { limit, offset });
  }

  async createContact(contact: Partial<Contact>): Promise<Contact> {
    return invoke('create_contact', { contact });
  }

  async updateContact(id: string, updates: Partial<Contact>): Promise<Contact> {
    return invoke('update_contact', { id, updates });
  }

  async deleteContact(id: string): Promise<void> {
    return invoke('delete_contact', { id });
  }

  async searchContacts(query: string): Promise<Contact[]> {
    return invoke('search_contacts', { query });
  }

  // Groups
  async getGroups(): Promise<Group[]> {
    return invoke('get_groups');
  }

  async createGroup(group: Partial<Group>): Promise<Group> {
    return invoke('create_group', { group });
  }

  // Projects
  async getProjects(): Promise<Project[]> {
    return invoke('get_projects');
  }

  async createProject(project: Partial<Project>): Promise<Project> {
    return invoke('create_project', { project });
  }

  // Calendar Events
  async getCalendarEvents(start?: string, end?: string): Promise<CalendarEvent[]> {
    return invoke('get_calendar_events', { start, end });
  }

  async createEvent(event: Partial<CalendarEvent>): Promise<CalendarEvent> {
    return invoke('create_event', { event });
  }

  // Notes
  async getNotes(entityType?: string, entityId?: string): Promise<Note[]> {
    return invoke('get_notes', { entityType, entityId });
  }

  async createNote(note: Partial<Note>): Promise<Note> {
    return invoke('create_note', { note });
  }

  // Communications
  async queueCommunication(attempt: Partial<CommunicationAttempt>): Promise<CommunicationAttempt> {
    return invoke('queue_communication', { attempt });
  }

  // Import
  async importCsv(path: string): Promise<number> {
    return invoke('import_csv', { path });
  }

  // Dashboard
  async getDashboard(): Promise<any> {
    return invoke('get_dashboard');
  }

  // Settings
  async getSettings(): Promise<any> {
    return invoke('get_settings');
  }

  async updateSettings(settings: any): Promise<void> {
    return invoke('update_settings', { settings });
  }

  // Sync
  async syncWithServer(): Promise<any> {
    return invoke('sync_with_server');
  }

  async checkOnline(): Promise<boolean> {
    return invoke('check_online');
  }
}

export const tauriApi = new TauriApiClient();
