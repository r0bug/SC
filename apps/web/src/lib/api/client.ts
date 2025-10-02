export interface Contact {
	id: string;
	first_name: string;
	last_name?: string;
	email?: string;
	phone?: string;
	organization?: string;
	title?: string;
	notes?: string;
	social_handles: SocialHandle[];
	tags: string[];
	projects: string[];
	created_at: string;
	updated_at: string;
	metadata: Record<string, any>;
}

export interface SocialHandle {
	platform: string;
	handle: string;
	url?: string;
}

export interface Note {
	id: string;
	contact_id?: string;
	project_id?: string;
	title: string;
	content: string;
	attachments: string[];
	created_at: string;
	updated_at: string;
}

export interface Project {
	id: string;
	name: string;
	description?: string;
	status: string;
	created_at: string;
	updated_at: string;
}

export interface Tag {
	id: string;
	name: string;
	color?: string;
	created_at: string;
}

export interface CommunicationAttempt {
	id: string;
	contact_id: string;
	method: CommunicationMethod;
	subject?: string;
	message: string;
	status: CommunicationStatus;
	scheduled_at?: string;
	attempted_at?: string;
	retry_count: number;
	created_at: string;
}

export type CommunicationMethod = 'Email' | 'SMS' | { Social: { platform: string } };
export type CommunicationStatus = 'Pending' | 'Sent' | 'Retrying' | { Failed: { reason: string } };

export class ApiClient {
	private baseUrl = '/api';

	async getContacts(limit = 50, offset = 0): Promise<Contact[]> {
		const res = await fetch(`${this.baseUrl}/contacts?limit=${limit}&offset=${offset}`);
		if (!res.ok) throw new Error('Failed to fetch contacts');
		return res.json();
	}

	async getContact(id: string): Promise<Contact> {
		const res = await fetch(`${this.baseUrl}/contacts/${id}`);
		if (!res.ok) throw new Error('Contact not found');
		return res.json();
	}

	async createContact(contact: Partial<Contact>): Promise<Contact> {
		const res = await fetch(`${this.baseUrl}/contacts`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(contact)
		});
		if (!res.ok) throw new Error('Failed to create contact');
		return res.json();
	}

	async searchContacts(query: string): Promise<Contact[]> {
		const res = await fetch(`${this.baseUrl}/contacts/search`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ query })
		});
		if (!res.ok) throw new Error('Search failed');
		return res.json();
	}

	async getTags(): Promise<Tag[]> {
		const res = await fetch(`${this.baseUrl}/tags`);
		if (!res.ok) throw new Error('Failed to fetch tags');
		return res.json();
	}

	async createTag(name: string, color?: string): Promise<Tag> {
		const res = await fetch(`${this.baseUrl}/tags`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name, color })
		});
		if (!res.ok) throw new Error('Failed to create tag');
		return res.json();
	}

	async getProjects(): Promise<Project[]> {
		const res = await fetch(`${this.baseUrl}/projects`);
		if (!res.ok) throw new Error('Failed to fetch projects');
		return res.json();
	}

	async createProject(name: string, description?: string): Promise<Project> {
		const res = await fetch(`${this.baseUrl}/projects`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name, description })
		});
		if (!res.ok) throw new Error('Failed to create project');
		return res.json();
	}

	async getContactNotes(contactId: string): Promise<Note[]> {
		const res = await fetch(`${this.baseUrl}/notes/contact/${contactId}`);
		if (!res.ok) throw new Error('Failed to fetch notes');
		return res.json();
	}

	async createNote(note: Partial<Note>): Promise<Note> {
		const res = await fetch(`${this.baseUrl}/notes`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify(note)
		});
		if (!res.ok) throw new Error('Failed to create note');
		return res.json();
	}

	async queueCommunication(
		contactId: string,
		method: CommunicationMethod,
		message: string,
		subject?: string
	): Promise<CommunicationAttempt> {
		const res = await fetch(`${this.baseUrl}/communication`, {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ contact_id: contactId, method, message, subject })
		});
		if (!res.ok) throw new Error('Failed to queue communication');
		return res.json();
	}

	async getSuggestions(contactId: string): Promise<any> {
		const res = await fetch(`${this.baseUrl}/ai/suggestions/${contactId}`);
		if (!res.ok) throw new Error('Failed to get suggestions');
		return res.json();
	}
}

export const api = new ApiClient();