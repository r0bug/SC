import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface UserSettings {
	theme: 'light' | 'dark' | 'auto';
	compactMode: boolean;
	notificationsEnabled: boolean;
	soundEnabled: boolean;
	pageSize: number;
	defaultView: 'list' | 'grid' | 'table';
	language: string;
}

const DEFAULT_SETTINGS: UserSettings = {
	theme: 'auto',
	compactMode: false,
	notificationsEnabled: true,
	soundEnabled: false,
	pageSize: 20,
	defaultView: 'list',
	language: 'en'
};

const STORAGE_KEY = 'sagenscontact_settings';

function loadSettings(): UserSettings {
	if (!browser) return DEFAULT_SETTINGS;

	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored) {
			return { ...DEFAULT_SETTINGS, ...JSON.parse(stored) };
		}
	} catch (error) {
		console.error('Failed to load settings:', error);
	}

	return DEFAULT_SETTINGS;
}

function createSettingsStore() {
	const { subscribe, set, update } = writable<UserSettings>(loadSettings());

	return {
		subscribe,
		set: (value: UserSettings) => {
			if (browser) {
				try {
					localStorage.setItem(STORAGE_KEY, JSON.stringify(value));
				} catch (error) {
					console.error('Failed to save settings:', error);
				}
			}
			set(value);
		},
		update: (updater: (value: UserSettings) => UserSettings) => {
			update(current => {
				const newValue = updater(current);
				if (browser) {
					try {
						localStorage.setItem(STORAGE_KEY, JSON.stringify(newValue));
					} catch (error) {
						console.error('Failed to save settings:', error);
					}
				}
				return newValue;
			});
		},
		reset: () => {
			if (browser) {
				try {
					localStorage.removeItem(STORAGE_KEY);
				} catch (error) {
					console.error('Failed to reset settings:', error);
				}
			}
			set(DEFAULT_SETTINGS);
		},
		updateField: <K extends keyof UserSettings>(key: K, value: UserSettings[K]) => {
			update(current => {
				const newValue = { ...current, [key]: value };
				if (browser) {
					try {
						localStorage.setItem(STORAGE_KEY, JSON.stringify(newValue));
					} catch (error) {
						console.error('Failed to save settings:', error);
					}
				}
				return newValue;
			});
		}
	};
}

export const settings = createSettingsStore();
