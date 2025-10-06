import type {
	Contact, Group, Concept, Project, CalendarEvent, Note, Attachment,
	Tag, CommunicationAttempt, CommunicationMethod, ShareInvite,
	AiInsight, SearchHistory, WorkerMetrics, AuthResponse,
	DashboardSummary, User, Permission, Connector, ImportPreviewResponse,
	ImportConfig, JobResponse, ImportJob, ImportLog, ImportHistoryFilters,
	RollbackResult
} from './types';

export class ApiError extends Error {
	constructor(
		message: string,
		public code: string,
		public status: number,
		public details?: any
	) {
		super(message);
		this.name = 'ApiError';
	}

	isRetryable(): boolean {
		return this.status >= 500 || this.code === 'NETWORK_ERROR' || this.code === 'RATE_LIMIT';
	}

	getUserMessage(): string {
		return this.message;
	}
}

export class ApiClient {
	private baseUrl = '/api';
	private token: string | null = null;
	private ws: WebSocket | null = null;
	private wsListeners = new Map<string, Set<(data: any) => void>>();

	constructor() {
		// Load token from localStorage if available
		if (typeof window !== 'undefined') {
			this.token = localStorage.getItem('auth_token');
		}
	}

	// Auth
	async signup(email: string, password: string, name: string): Promise<AuthResponse> {
		const res = await this.request('/auth/signup', {
			method: 'POST',
			body: JSON.stringify({ email, password, name })
		});
		this.token = res.token;
		localStorage.setItem('auth_token', res.token);
		this.connectWebSocket();
		return res;
	}

	async login(email: string, password: string): Promise<AuthResponse> {
		const res = await this.request('/auth/login', {
			method: 'POST',
			body: JSON.stringify({ email, password })
		});
		this.token = res.token;
		localStorage.setItem('auth_token', res.token);
		this.connectWebSocket();
		return res;
	}

	logout() {
		this.token = null;
		localStorage.removeItem('auth_token');
		if (this.ws) {
			this.ws.close();
			this.ws = null;
		}
	}

	// Dashboard
	async getDashboard(): Promise<DashboardSummary> {
		return this.request('/dashboard');
	}

	// Contacts
	async getContacts(limit = 50, offset = 0): Promise<Contact[]> {
		return this.request(`/contacts?limit=${limit}&offset=${offset}`);
	}

	async getContact(id: string): Promise<Contact> {
		return this.request(`/contacts/${id}`);
	}

	async createContact(contact: Partial<Contact>): Promise<Contact> {
		return this.request('/contacts', {
			method: 'POST',
			body: JSON.stringify(contact)
		});
	}

	async updateContact(id: string, updates: Partial<Contact>): Promise<Contact> {
		return this.request(`/contacts/${id}`, {
			method: 'PUT',
			body: JSON.stringify(updates)
		});
	}

	async deleteContact(id: string): Promise<void> {
		return this.request(`/contacts/${id}`, { method: 'DELETE' });
	}

	async searchContacts(query: string, filters?: Record<string, any>): Promise<Contact[]> {
		return this.request('/contacts/search', {
			method: 'POST',
			body: JSON.stringify({ query, filters })
		});
	}

	// Groups
	async getGroups(): Promise<Group[]> {
		return this.request('/groups');
	}

	async getGroup(id: string): Promise<Group> {
		return this.request(`/groups/${id}`);
	}

	async createGroup(group: Partial<Group>): Promise<Group> {
		return this.request('/groups', {
			method: 'POST',
			body: JSON.stringify(group)
		});
	}

	async updateGroup(id: string, updates: Partial<Group>): Promise<Group> {
		return this.request(`/groups/${id}`, {
			method: 'PUT',
			body: JSON.stringify(updates)
		});
	}

	async deleteGroup(id: string): Promise<void> {
		return this.request(`/groups/${id}`, { method: 'DELETE' });
	}

	async addGroupMember(groupId: string, contactId: string): Promise<void> {
		return this.request(`/groups/${groupId}/members`, {
			method: 'POST',
			body: JSON.stringify({ contact_id: contactId })
		});
	}

	async removeGroupMember(groupId: string, contactId: string): Promise<void> {
		return this.request(`/groups/${groupId}/members/${contactId}`, {
			method: 'DELETE'
		});
	}

	// Concepts
	async getConcepts(): Promise<Concept[]> {
		return this.request('/concepts');
	}

	async getConcept(id: string): Promise<Concept> {
		return this.request(`/concepts/${id}`);
	}

	async createConcept(concept: Partial<Concept>): Promise<Concept> {
		return this.request('/concepts', {
			method: 'POST',
			body: JSON.stringify(concept)
		});
	}

	async updateConcept(id: string, updates: Partial<Concept>): Promise<Concept> {
		return this.request(`/concepts/${id}`, {
			method: 'PUT',
			body: JSON.stringify(updates)
		});
	}

	async deleteConcept(id: string): Promise<void> {
		return this.request(`/concepts/${id}`, { method: 'DELETE' });
	}

	// Projects
	async getProjects(): Promise<Project[]> {
		return this.request('/projects');
	}

	async getProject(id: string): Promise<Project> {
		return this.request(`/projects/${id}`);
	}

	async createProject(project: Partial<Project>): Promise<Project> {
		return this.request('/projects', {
			method: 'POST',
			body: JSON.stringify(project)
		});
	}

	async updateProject(id: string, updates: Partial<Project>): Promise<Project> {
		return this.request(`/projects/${id}`, {
			method: 'PUT',
			body: JSON.stringify(updates)
		});
	}

	async deleteProject(id: string): Promise<void> {
		return this.request(`/projects/${id}`, { method: 'DELETE' });
	}

	// Calendar
	async getEvents(start?: string, end?: string): Promise<CalendarEvent[]> {
		const params = new URLSearchParams();
		if (start) params.append('start', start);
		if (end) params.append('end', end);
		return this.request(`/calendar/events?${params}`);
	}

	async getEvent(id: string): Promise<CalendarEvent> {
		return this.request(`/calendar/events/${id}`);
	}

	async createEvent(event: Partial<CalendarEvent>): Promise<CalendarEvent> {
		return this.request('/calendar/events', {
			method: 'POST',
			body: JSON.stringify(event)
		});
	}

	async updateEvent(id: string, updates: Partial<CalendarEvent>): Promise<CalendarEvent> {
		return this.request(`/calendar/events/${id}`, {
			method: 'PUT',
			body: JSON.stringify(updates)
		});
	}

	async deleteEvent(id: string): Promise<void> {
		return this.request(`/calendar/events/${id}`, { method: 'DELETE' });
	}

	// Notes
	async getNotes(entityType?: string, entityId?: string): Promise<Note[]> {
		const params = new URLSearchParams();
		if (entityType) params.append('entity_type', entityType);
		if (entityId) params.append('entity_id', entityId);
		return this.request(`/notes?${params}`);
	}

	async getNote(id: string): Promise<Note> {
		return this.request(`/notes/${id}`);
	}

	async createNote(note: Partial<Note>): Promise<Note> {
		return this.request('/notes', {
			method: 'POST',
			body: JSON.stringify(note)
		});
	}

	async updateNote(id: string, updates: Partial<Note>): Promise<Note> {
		return this.request(`/notes/${id}`, {
			method: 'PUT',
			body: JSON.stringify(updates)
		});
	}

	async deleteNote(id: string): Promise<void> {
		return this.request(`/notes/${id}`, { method: 'DELETE' });
	}

	// Attachments
	async getAttachments(entityType: string, entityId: string): Promise<Attachment[]> {
		return this.request(`/attachments?entity_type=${entityType}&entity_id=${entityId}`);
	}

	async uploadAttachment(
		file: File,
		entityType: string,
		entityId: string,
		uploadedBy: string
	): Promise<Attachment> {
		// Validate file size (50MB max)
		const MAX_FILE_SIZE = 50 * 1024 * 1024;
		if (file.size > MAX_FILE_SIZE) {
			throw new ApiError(
				`File size exceeds maximum limit of 50MB. Your file is ${(file.size / 1024 / 1024).toFixed(1)}MB.`,
				'FILE_TOO_LARGE',
				413
			);
		}

		// Validate file type
		const allowedTypes = [
			'image/jpeg', 'image/png', 'image/gif', 'image/webp',
			'application/pdf',
			'application/msword', 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
			'application/vnd.ms-excel', 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
			'text/plain', 'text/csv'
		];

		if (!allowedTypes.includes(file.type)) {
			throw new ApiError(
				`File type "${file.type}" is not supported. Please upload images, PDFs, or common document formats.`,
				'INVALID_FILE_TYPE',
				400
			);
		}

		const formData = new FormData();
		formData.append('file', file);
		formData.append('entity_type', entityType);
		formData.append('entity_id', entityId);
		formData.append('uploaded_by', uploadedBy);

		try {
			const res = await fetch(`${this.baseUrl}/attachments/upload`, {
				method: 'POST',
				headers: this.getAuthHeaders(false),
				body: formData
			});

			if (!res.ok) {
				return await this.handleErrorResponse(res, '/attachments/upload');
			}

			const response = await res.json();
			return response.attachment;
		} catch (error) {
			if (error instanceof ApiError) throw error;
			if (error instanceof TypeError && error.message.includes('fetch')) {
				throw new ApiError(
					'Unable to upload file. Please check your internet connection.',
					'NETWORK_ERROR',
					0
				);
			}
			throw new ApiError(
				'Failed to upload attachment. Please try again.',
				'UPLOAD_ERROR',
				500
			);
		}
	}

	async downloadAttachment(id: string, filename: string): Promise<void> {
		const res = await fetch(`${this.baseUrl}/attachments/${id}`, {
			headers: this.getAuthHeaders()
		});
		if (!res.ok) throw new Error('Failed to download attachment');

		// Download as file
		const blob = await res.blob();
		const url = window.URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = filename;
		document.body.appendChild(a);
		a.click();
		window.URL.revokeObjectURL(url);
		document.body.removeChild(a);
	}

	async deleteAttachment(id: string): Promise<void> {
		return this.request(`/attachments/${id}`, { method: 'DELETE' });
	}

	// Tags
	async getTags(): Promise<Tag[]> {
		return this.request('/tags');
	}

	async createTag(name: string, color?: string): Promise<Tag> {
		return this.request('/tags', {
			method: 'POST',
			body: JSON.stringify({ name, color })
		});
	}

	async updateTag(id: string, updates: { name?: string; color?: string }): Promise<Tag> {
		return this.request(`/tags/${id}`, {
			method: 'PUT',
			body: JSON.stringify(updates)
		});
	}

	async deleteTag(id: string): Promise<void> {
		return this.request(`/tags/${id}`, { method: 'DELETE' });
	}

	// Communications
	async getCommunications(contactId?: string): Promise<CommunicationAttempt[]> {
		const path = contactId ? `/communications?contact_id=${contactId}` : '/communications';
		return this.request(path);
	}

	async queueCommunication(
		contactId: string,
		method: CommunicationMethod,
		message: string,
		subject?: string,
		scheduledAt?: string
	): Promise<CommunicationAttempt> {
		return this.request('/communications', {
			method: 'POST',
			body: JSON.stringify({
				contact_id: contactId,
				method,
				message,
				subject,
				scheduled_at: scheduledAt
			})
		});
	}

	async cancelCommunication(id: string): Promise<void> {
		return this.request(`/communications/${id}/cancel`, { method: 'POST' });
	}

	// Shares
	async getShares(): Promise<ShareInvite[]> {
		return this.request('/shares');
	}

	async createShare(
		entityType: string,
		entityId: string,
		email: string,
		permissions: Permission[]
	): Promise<ShareInvite> {
		return this.request('/shares', {
			method: 'POST',
			body: JSON.stringify({
				entity_type: entityType,
				entity_id: entityId,
				shared_with_email: email,
				permissions
			})
		});
	}

	async acceptShare(id: string): Promise<void> {
		return this.request(`/shares/${id}/accept`, { method: 'POST' });
	}

	async revokeShare(id: string): Promise<void> {
		return this.request(`/shares/${id}/revoke`, { method: 'POST' });
	}

	// AI Insights
	async getInsights(entityType?: string, entityId?: string): Promise<AiInsight[]> {
		const params = new URLSearchParams();
		if (entityType) params.append('entity_type', entityType);
		if (entityId) params.append('entity_id', entityId);
		return this.request(`/ai/insights?${params}`);
	}

	async applyInsight(id: string): Promise<void> {
		return this.request(`/ai/insights/${id}/apply`, { method: 'POST' });
	}

	async feedbackInsight(id: string, helpful: boolean, comment?: string): Promise<void> {
		return this.request(`/ai/insights/${id}/feedback`, {
			method: 'POST',
			body: JSON.stringify({ helpful, comment })
		});
	}

	// Search History
	async getSearchHistory(): Promise<SearchHistory[]> {
		return this.request('/search/history');
	}

	async clearSearchHistory(): Promise<void> {
		return this.request('/search/history', { method: 'DELETE' });
	}

	// Import - Real API integration
	async getConnectors(): Promise<Connector[]> {
		return this.request('/import/connectors');
	}

	async previewImport(file: File, limit: number = 10): Promise<ImportPreviewResponse> {
		const formData = new FormData();
		formData.append('file', file);

		const res = await fetch(`${this.baseUrl}/import/preview?limit=${limit}`, {
			method: 'POST',
			headers: this.getAuthHeaders(false),
			body: formData
		});

		if (!res.ok) {
			await this.handleErrorResponse(res, '/import/preview');
		}

		const data = await res.json();

		// Transform response to add detected_fields for UI compatibility
		const detected_fields = data.suggested_mappings?.map(([field_name, suggested_mapping]: [string, string]) => ({
			field_name,
			suggested_mapping,
			sample_values: [],
			confidence: 1.0
		})) || [];

		return {
			...data,
			detected_fields,
			validation_errors: []
		};
	}

	async executeImport(
		file: File,
		config: ImportConfig
	): Promise<JobResponse> {
		const formData = new FormData();
		formData.append('file', file);
		formData.append('config', JSON.stringify(config));

		const res = await fetch(`${this.baseUrl}/import/execute`, {
			method: 'POST',
			headers: this.getAuthHeaders(false),
			body: formData
		});

		if (!res.ok) {
			await this.handleErrorResponse(res, '/import/execute');
		}

		return res.json();
	}

	async getJobStatus(jobId: string): Promise<ImportJob> {
		return this.request(`/import/jobs/${jobId}`);
	}

	async listJobs(): Promise<ImportJob[]> {
		return this.request('/import/jobs');
	}

	async cancelJob(jobId: string): Promise<void> {
		return this.request(`/import/jobs/${jobId}/cancel`, { method: 'POST' });
	}

	async getImportHistory(filters?: ImportHistoryFilters): Promise<ImportLog[]> {
		try {
			const params = new URLSearchParams();
			if (filters?.status) params.append('status', filters.status);
			if (filters?.connector_id) params.append('connector_id', filters.connector_id);
			if (filters?.start_date) params.append('start_date', filters.start_date);
			if (filters?.end_date) params.append('end_date', filters.end_date);
			const query = params.toString() ? `?${params}` : '';
			return await this.request(`/import/history${query}`);
		} catch (error: any) {
			// Alpha: History endpoint not implemented yet, return empty array
			if (error.status === 404) {
				return [];
			}
			throw error;
		}
	}

	async rollbackImport(logId: string): Promise<RollbackResult> {
		return this.request(`/import/history/${logId}/rollback`, { method: 'POST' });
	}

	async downloadErrorReport(logId: string): Promise<void> {
		const res = await fetch(`${this.baseUrl}/import/history/${logId}/errors`, {
			headers: this.getAuthHeaders()
		});
		if (!res.ok) throw new Error('Failed to download error report');

		const blob = await res.blob();
		const url = window.URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `import-errors-${logId}.csv`;
		document.body.appendChild(a);
		a.click();
		window.URL.revokeObjectURL(url);
		document.body.removeChild(a);
	}

	// Worker Metrics
	async getWorkerMetrics(): Promise<WorkerMetrics[]> {
		return this.request('/metrics/worker');
	}

	async getWorkerHealth(): Promise<any> {
		return this.request('/health/worker');
	}

	// Settings
	async getSettings(): Promise<Record<string, any>> {
		return this.request('/settings');
	}

	async updateSettings(settings: Record<string, any>): Promise<void> {
		return this.request('/settings', {
			method: 'PUT',
			body: JSON.stringify(settings)
		});
	}

	// WebSocket
	connectWebSocket() {
		if (!this.token || this.ws) return;

		const wsUrl = this.baseUrl.replace('http', 'ws').replace('/api', '/ws');
		this.ws = new WebSocket(`${wsUrl}?token=${this.token}`);

		this.ws.onmessage = (event) => {
			const data = JSON.parse(event.data);
			const listeners = this.wsListeners.get(data.type) || new Set();
			listeners.forEach(listener => listener(data.payload));
		};

		this.ws.onerror = (error) => {
			console.error('WebSocket error:', error);
		};

		this.ws.onclose = () => {
			this.ws = null;
			// Attempt reconnection after 5 seconds
			setTimeout(() => {
				if (this.token) this.connectWebSocket();
			}, 5000);
		};
	}

	onWebSocketMessage(type: string, callback: (data: any) => void) {
		if (!this.wsListeners.has(type)) {
			this.wsListeners.set(type, new Set());
		}
		this.wsListeners.get(type)!.add(callback);

		// Return unsubscribe function
		return () => {
			this.wsListeners.get(type)?.delete(callback);
		};
	}

	// Private helpers
	private async request(path: string, options: RequestInit = {}): Promise<any> {
		try {
			const res = await fetch(`${this.baseUrl}${path}`, {
				...options,
				headers: {
					...this.getAuthHeaders(),
					'Content-Type': 'application/json',
					...options.headers
				}
			});

			if (!res.ok) {
				return await this.handleErrorResponse(res, path);
			}

			const text = await res.text();
			return text ? JSON.parse(text) : null;
		} catch (error) {
			if (error instanceof TypeError && error.message.includes('fetch')) {
				throw new ApiError(
					'Unable to connect to the server. Please check your internet connection.',
					'NETWORK_ERROR',
					0
				);
			}
			throw error;
		}
	}

	private async handleErrorResponse(res: Response, path: string): Promise<never> {
		let errorMessage = `Request failed: ${res.statusText}`;
		let errorCode = 'UNKNOWN_ERROR';
		let details: any = undefined;

		try {
			const contentType = res.headers.get('content-type');
			if (contentType?.includes('application/json')) {
				const errorData = await res.json();
				errorMessage = errorData.message || errorData.error || errorMessage;
				errorCode = errorData.code || errorCode;
				details = errorData.details;
			} else {
				const text = await res.text();
				if (text) errorMessage = text;
			}
		} catch (e) {
			// Failed to parse error response, use default message
		}

		// Provide specific, user-friendly error messages
		switch (res.status) {
			case 400:
				errorCode = 'BAD_REQUEST';
				if (!details) {
					errorMessage = `Invalid request: ${errorMessage}. Please check your input and try again.`;
				}
				break;
			case 401:
				errorCode = 'UNAUTHORIZED';
				errorMessage = 'Your session has expired. Please log in again.';
				// Auto-logout on 401
				this.logout();
				if (typeof window !== 'undefined') {
					window.location.href = '/auth/login';
				}
				break;
			case 403:
				errorCode = 'FORBIDDEN';
				errorMessage = 'You do not have permission to perform this action.';
				break;
			case 404:
				errorCode = 'NOT_FOUND';
				errorMessage = `The requested resource was not found. It may have been deleted or moved.`;
				break;
			case 409:
				errorCode = 'CONFLICT';
				errorMessage = `This action conflicts with existing data: ${errorMessage}`;
				break;
			case 413:
				errorCode = 'PAYLOAD_TOO_LARGE';
				errorMessage = 'The file or data you are trying to upload is too large. Please try a smaller file.';
				break;
			case 422:
				errorCode = 'VALIDATION_ERROR';
				if (!details) {
					errorMessage = `Validation failed: ${errorMessage}. Please check your input.`;
				}
				break;
			case 429:
				errorCode = 'RATE_LIMIT';
				errorMessage = 'Too many requests. Please wait a moment and try again.';
				break;
			case 500:
			case 502:
			case 503:
			case 504:
				errorCode = 'SERVER_ERROR';
				errorMessage = 'The server encountered an error. Please try again in a moment.';
				break;
		}

		throw new ApiError(errorMessage, errorCode, res.status, details);
	}

	private getAuthHeaders(includeContentType = true): Record<string, string> {
		const headers: Record<string, string> = {};
		if (this.token) {
			headers['Authorization'] = `Bearer ${this.token}`;
		}
		if (includeContentType) {
			headers['Content-Type'] = 'application/json';
		}
		return headers;
	}
}

export const api = new ApiClient();