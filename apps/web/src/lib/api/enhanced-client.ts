import type { Contact, Note, Project, Tag, CommunicationAttempt, CommunicationMethod } from './client';

interface RetryConfig {
	maxRetries: number;
	retryDelay: number;
	retryableStatuses: number[];
}

const defaultRetryConfig: RetryConfig = {
	maxRetries: 3,
	retryDelay: 1000,
	retryableStatuses: [408, 429, 500, 502, 503, 504]
};

export class EnhancedApiClient {
	private baseUrl = '/api';
	private retryConfig: RetryConfig;
	private abortControllers = new Map<string, AbortController>();

	constructor(retryConfig: Partial<RetryConfig> = {}) {
		this.retryConfig = { ...defaultRetryConfig, ...retryConfig };
	}

	private async fetchWithRetry(
		url: string,
		options: RequestInit = {},
		retryCount = 0
	): Promise<Response> {
		const requestId = `${url}-${Date.now()}`;
		const controller = new AbortController();
		this.abortControllers.set(requestId, controller);

		try {
			const response = await fetch(url, {
				...options,
				signal: controller.signal
			});

			this.abortControllers.delete(requestId);

			// If response is ok or not retryable, return it
			if (response.ok || !this.retryConfig.retryableStatuses.includes(response.status)) {
				return response;
			}

			// Retry logic
			if (retryCount < this.retryConfig.maxRetries) {
				const delay = this.retryConfig.retryDelay * Math.pow(2, retryCount);
				console.warn(`Request failed with status ${response.status}. Retrying in ${delay}ms... (${retryCount + 1}/${this.retryConfig.maxRetries})`);

				await new Promise(resolve => setTimeout(resolve, delay));
				return this.fetchWithRetry(url, options, retryCount + 1);
			}

			return response;
		} catch (error: any) {
			this.abortControllers.delete(requestId);

			// Network error - retry if not aborted
			if (error.name !== 'AbortError' && retryCount < this.retryConfig.maxRetries) {
				const delay = this.retryConfig.retryDelay * Math.pow(2, retryCount);
				console.warn(`Network error. Retrying in ${delay}ms... (${retryCount + 1}/${this.retryConfig.maxRetries})`);

				await new Promise(resolve => setTimeout(resolve, delay));
				return this.fetchWithRetry(url, options, retryCount + 1);
			}

			throw error;
		}
	}

	private async request<T>(
		endpoint: string,
		options: RequestInit = {}
	): Promise<T> {
		const url = `${this.baseUrl}${endpoint}`;

		try {
			const response = await this.fetchWithRetry(url, options);

			if (!response.ok) {
				let errorMessage = `Request failed with status ${response.status}`;
				try {
					const errorData = await response.json();
					errorMessage = errorData.message || errorData.error || errorMessage;
				} catch {
					// Ignore JSON parse errors
				}
				throw new Error(errorMessage);
			}

			// Handle empty responses
			const contentType = response.headers.get('content-type');
			if (!contentType || !contentType.includes('application/json')) {
				return {} as T;
			}

			return await response.json();
		} catch (error: any) {
			if (error.name === 'AbortError') {
				throw new Error('Request was cancelled');
			}

			// Provide user-friendly error messages
			if (error.message.includes('Failed to fetch')) {
				throw new Error('Cannot connect to server. Please check if the service is running.');
			}

			throw error;
		}
	}

	cancelAllRequests() {
		this.abortControllers.forEach(controller => controller.abort());
		this.abortControllers.clear();
	}

	// Contact endpoints
	async getContacts(limit = 50, offset = 0): Promise<Contact[]> {
		return this.request<Contact[]>(`/contacts?limit=${limit}&offset=${offset}`);
	}

	async getContact(id: string): Promise<Contact> {
		return this.request<Contact>(`/contacts/${id}`);
	}

	async createContact(contact: Partial<Contact>): Promise<Contact> {
		return this.request<Contact>('/contacts', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(contact)
		});
	}

	async updateContact(id: string, updates: Partial<Contact>): Promise<Contact> {
		return this.request<Contact>(`/contacts/${id}`, {
			method: 'PUT',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(updates)
		});
	}

	async deleteContact(id: string): Promise<void> {
		return this.request<void>(`/contacts/${id}`, {
			method: 'DELETE'
		});
	}

	async searchContacts(query: string): Promise<Contact[]> {
		return this.request<Contact[]>('/contacts/search', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ query })
		});
	}

	// Tag endpoints
	async getTags(): Promise<Tag[]> {
		return this.request<Tag[]>('/tags');
	}

	async createTag(name: string, color?: string): Promise<Tag> {
		return this.request<Tag>('/tags', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name, color })
		});
	}

	// Project endpoints
	async getProjects(): Promise<Project[]> {
		return this.request<Project[]>('/projects');
	}

	async createProject(name: string, description?: string): Promise<Project> {
		return this.request<Project>('/projects', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name, description })
		});
	}

	// Note endpoints
	async getContactNotes(contactId: string): Promise<Note[]> {
		return this.request<Note[]>(`/notes/contact/${contactId}`);
	}

	async createNote(note: Partial<Note>): Promise<Note> {
		return this.request<Note>('/notes', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(note)
		});
	}

	// Communication endpoints
	async queueCommunication(
		contactId: string,
		method: CommunicationMethod,
		message: string,
		subject?: string
	): Promise<CommunicationAttempt> {
		return this.request<CommunicationAttempt>('/communication', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ contact_id: contactId, method, message, subject })
		});
	}

	// AI endpoints
	async getSuggestions(contactId: string): Promise<any> {
		return this.request<any>(`/ai/suggestions/${contactId}`);
	}

	// Health check
	async healthCheck(): Promise<{ status: string }> {
		return this.request<{ status: string }>('/health');
	}
}

export const enhancedApi = new EnhancedApiClient();
