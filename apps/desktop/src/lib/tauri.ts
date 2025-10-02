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
