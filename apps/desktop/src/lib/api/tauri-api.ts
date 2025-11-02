import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { sendNotification } from '@tauri-apps/plugin-notification';
import type { Contact, Group, Project, CalendarEvent, Note, CommunicationAttempt, Communication, Attachment } from './types';

export class TauriApiClient {
  // Contacts
  async getContacts(limit: number = 100, offset: number = 0): Promise<Contact[]> {
    return invoke('get_contacts', { limit, offset });
  }

  async getContact(id: string): Promise<Contact> {
    const contacts = await this.getContacts(1000, 0);
    const contact = contacts.find(c => c.id === id);
    if (!contact) throw new Error('Contact not found');
    return contact;
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

  async updateGroup(id: string, group: Partial<Group>): Promise<Group> {
    return invoke('update_group', { id, group });
  }

  async deleteGroup(id: string): Promise<void> {
    return invoke('delete_group', { id });
  }

  // Projects
  async getProjects(): Promise<Project[]> {
    return invoke('get_projects');
  }

  async getProject(id: string): Promise<Project> {
    return invoke('get_project', { id });
  }

  async createProject(project: Partial<Project>): Promise<Project> {
    return invoke('create_project', { project });
  }

  async updateProject(id: string, project: Partial<Project>): Promise<Project> {
    return invoke('update_project', { id, project });
  }

  async deleteProject(id: string): Promise<void> {
    return invoke('delete_project', { id });
  }

  async addProjectContact(projectId: string, contactId: string): Promise<void> {
    return invoke('add_project_contact', { projectId, contactId });
  }

  async removeProjectContact(projectId: string, contactId: string): Promise<void> {
    return invoke('remove_project_contact', { projectId, contactId });
  }

  // Calendar Events
  async getCalendarEvents(start?: string, end?: string): Promise<CalendarEvent[]> {
    return invoke('get_calendar_events', { start, end });
  }

  async createEvent(event: Partial<CalendarEvent>): Promise<CalendarEvent> {
    return invoke('create_event', { event });
  }

  async updateEvent(id: string, event: Partial<CalendarEvent>): Promise<CalendarEvent> {
    return invoke('update_event', { id, event });
  }

  async deleteEvent(id: string): Promise<void> {
    return invoke('delete_event', { id });
  }

  // Notes
  async getNotes(entityType?: string, entityId?: string): Promise<Note[]> {
    return invoke('get_notes', { entityType, entityId });
  }

  async createNote(note: Partial<Note>): Promise<Note> {
    return invoke('create_note', { note });
  }

  // Communications
  async getAllCommunications(limit: number = 100, offset: number = 0): Promise<Communication[]> {
    return invoke('get_all_communications', { limit, offset });
  }

  async getCommunications(contactId: string): Promise<Communication[]> {
    return invoke('get_communications', { contactId });
  }

  async queueCommunication(attempt: Partial<CommunicationAttempt>): Promise<CommunicationAttempt> {
    return invoke('queue_communication', { attempt });
  }

  // Attachments
  async getAttachments(entityType: string, entityId: string): Promise<Attachment[]> {
    return invoke('get_attachments', { entityType, entityId });
  }

  async uploadAttachment(
    filePath: string,
    entityType: string,
    entityId: string,
    uploadedBy: string
  ): Promise<Attachment> {
    return invoke('upload_attachment', {
      filePath,
      entityType,
      entityId,
      uploadedBy
    });
  }

  async downloadAttachment(id: string, savePath: string): Promise<void> {
    return invoke('download_attachment', { id, savePath });
  }

  async deleteAttachment(id: string): Promise<void> {
    return invoke('delete_attachment', { id });
  }

  // Import
  async importCsv(path: string): Promise<number> {
    return invoke('import_csv', { path });
  }

  async importCsvDialog(): Promise<number> {
    const selected = await open({
      multiple: false,
      filters: [{
        name: 'CSV',
        extensions: ['csv']
      }]
    });

    if (selected && typeof selected === 'string') {
      return this.importCsv(selected);
    }
    return 0;
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

  // File Dialog Helpers
  async selectFile(filters?: { name: string; extensions: string[] }[]): Promise<string | null> {
    const selected = await open({
      multiple: false,
      filters: filters || []
    });
    return typeof selected === 'string' ? selected : null;
  }

  async selectFiles(filters?: { name: string; extensions: string[] }[]): Promise<string[] | null> {
    const selected = await open({
      multiple: true,
      filters: filters || []
    });
    return Array.isArray(selected) ? selected : null;
  }

  async saveFile(
    defaultPath?: string,
    filters?: { name: string; extensions: string[] }[]
  ): Promise<string | null> {
    const selected = await save({
      defaultPath,
      filters: filters || []
    });
    return selected;
  }

  // Notification Helper
  async showNotification(title: string, body: string): Promise<void> {
    await sendNotification({ title, body });
  }
}

export const tauriApi = new TauriApiClient();

// Check if running in Tauri
export const isTauri = () => {
  return typeof window !== 'undefined' && (window as any).__TAURI__ !== undefined;
};
