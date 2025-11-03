import { invoke } from '@tauri-apps/api/core';
import { open, save } from '@tauri-apps/plugin-dialog';
import { sendNotification } from '@tauri-apps/plugin-notification';
import type { Contact, Group, Project, CalendarEvent, Note, CommunicationAttempt, Communication, Attachment, ImportAnalysis, SmartImportPreview } from './types';

// Utility function to generate UUID
function generateUUID(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    const r = Math.random() * 16 | 0;
    const v = c === 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}

// System user ID for desktop app
const SYSTEM_USER_ID = '00000000-0000-0000-0000-000000000000';

export class TauriApiClient {
  // Contacts
  async getContacts(limit: number = 100, offset: number = 0): Promise<Contact[]> {
    return invoke('get_contacts', { limit, offset });
  }

  async getContact(id: string): Promise<Contact> {
    const contacts = await this.getContacts(10000, 0);
    const contact = contacts.find(c => c.id === id);
    if (!contact) throw new Error('Contact not found');
    return contact;
  }

  async createContact(contact: Partial<Contact>): Promise<Contact> {
    const now = new Date().toISOString();
    const fullContact: Contact = {
      id: generateUUID(),
      first_name: contact.first_name || '',
      last_name: contact.last_name || null,
      email: contact.email || null,
      phone: contact.phone || null,
      organization: contact.organization || null,
      title: contact.title || null,
      notes: contact.notes || null,
      social_handles: contact.social_handles || [],
      tags: contact.tags || [],
      projects: contact.projects || [],
      groups: contact.groups || [],
      created_at: now,
      updated_at: now,
      created_by: SYSTEM_USER_ID,
      version: 1,
      last_synced_at: null,
      metadata: contact.metadata || {}
    };
    return invoke('create_contact', { contact: fullContact });
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

  async getGroup(id: string): Promise<Group> {
    const groups = await this.getGroups();
    const group = groups.find(g => g.id === id);
    if (!group) throw new Error('Group not found');
    return group;
  }

  async createGroup(group: Partial<Group>): Promise<Group> {
    const now = new Date().toISOString();
    const fullGroup: Group = {
      id: generateUUID(),
      name: group.name || '',
      description: group.description || null,
      contact_ids: group.contact_ids || [],
      created_at: now,
      updated_at: now,
      created_by: SYSTEM_USER_ID
    };
    return invoke('create_group', { group: fullGroup });
  }

  async updateGroup(id: string, group: Partial<Group>): Promise<Group> {
    return invoke('update_group', { id, group });
  }

  async deleteGroup(id: string): Promise<void> {
    return invoke('delete_group', { id });
  }

  async addGroupMember(groupId: string, contactId: string): Promise<void> {
    return invoke('add_group_member', { groupId, contactId });
  }

  async removeGroupMember(groupId: string, contactId: string): Promise<void> {
    return invoke('remove_group_member', { groupId, contactId });
  }

  // Projects
  async getProjects(): Promise<Project[]> {
    return invoke('get_projects');
  }

  async getProject(id: string): Promise<Project> {
    return invoke('get_project', { id });
  }

  async createProject(project: Partial<Project>): Promise<Project> {
    const now = new Date().toISOString();
    const fullProject: Project = {
      id: generateUUID(),
      name: project.name || '',
      description: project.description || null,
      status: project.status || 'Planning',
      contact_ids: project.contact_ids || [],
      start_date: project.start_date || null,
      end_date: project.end_date || null,
      created_at: now,
      updated_at: now,
      created_by: SYSTEM_USER_ID,
      metadata: project.metadata || {}
    };
    return invoke('create_project', { project: fullProject });
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

  async getEvent(id: string): Promise<CalendarEvent> {
    const events = await this.getCalendarEvents();
    const event = events.find(e => e.id === id);
    if (!event) throw new Error('Event not found');
    return event;
  }

  async createEvent(event: Partial<CalendarEvent>): Promise<CalendarEvent> {
    const now = new Date().toISOString();
    const fullEvent: CalendarEvent = {
      id: generateUUID(),
      title: event.title || '',
      description: event.description || null,
      location: event.location || null,
      start_time: event.start_time || now,
      end_time: event.end_time || null,
      all_day: event.all_day || false,
      recurrence_rule: event.recurrence_rule || null,
      attendees: event.attendees || [],
      created_at: now,
      updated_at: now,
      created_by: SYSTEM_USER_ID
    };
    return invoke('create_event', { event: fullEvent });
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
    const now = new Date().toISOString();
    const fullNote: Note = {
      id: generateUUID(),
      entity_type: note.entity_type || 'Contact',
      entity_id: note.entity_id || '',
      content: note.content || '',
      created_at: now,
      updated_at: now,
      created_by: SYSTEM_USER_ID
    };
    return invoke('create_note', { note: fullNote });
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

  // SMS Import - with proper contact matching and message storage
  async importSmsFile(filePath: string): Promise<{
    contacts_created: number;
    contacts_found: number;
    messages_imported: number;
    total_phone_numbers: number;
    failed: number;
  }> {
    return invoke('import_sms_file', { filePath });
  }

  async checkIsSmsFile(filePath: string): Promise<boolean> {
    return invoke('check_is_sms_file', { filePath });
  }

  // Smart Import - AI-powered format detection and mapping
  async analyzeImportFile(filePath: string): Promise<ImportAnalysis> {
    return invoke('analyze_import_file', { filePath });
  }

  async previewImport(filePath: string, fieldMappings: Record<string, string>): Promise<SmartImportPreview> {
    return invoke('preview_import', { filePath, fieldMappings });
  }

  async executeImport(filePath: string, fieldMappings: Record<string, string>): Promise<number> {
    return invoke('execute_import', { filePath, fieldMappings });
  }

  async smartImportDialog(): Promise<number | null> {
    const selected = await open({
      multiple: false,
      filters: [
        { name: 'All Supported', extensions: ['csv', 'json', 'xml', 'html', 'htm', 'txt'] },
        { name: 'CSV', extensions: ['csv'] },
        { name: 'JSON', extensions: ['json'] },
        { name: 'XML', extensions: ['xml'] },
        { name: 'HTML', extensions: ['html', 'htm'] },
        { name: 'Text', extensions: ['txt'] }
      ]
    });

    if (selected && typeof selected === 'string') {
      // This will be handled by the import wizard UI
      return null;
    }
    return null;
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

  // Deduplication
  async findDuplicateContacts(): Promise<any[]> {
    return invoke('find_duplicate_contacts');
  }

  async mergeContacts(keepId: string, mergeId: string): Promise<any> {
    return invoke('merge_contacts_cmd', { keepId, mergeId });
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
