import type {
	Contact, Group, Concept, Project, CalendarEvent, Note, Attachment,
	Tag, CommunicationAttempt, CommunicationMethod, ShareInvite,
	AiInsight, SearchHistory, WorkerMetrics, AuthResponse,
	DashboardSummary, User, Permission, Connector, ImportPreviewResponse,
	ImportConfig, JobResponse, ImportJob, ImportLog, ImportHistoryFilters,
	RollbackResult, Pick, Location, RelationshipType, EntityRelationship,
	CommunicationConcept, MatchResult, ConceptKeyword
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

// --- Label Matcher types ---
export interface MatcherGroup {
	id: string;
	concept_id: string;
	position: number;
	matchers: Matcher[];
}

export interface Matcher {
	id: string;
	group_id: string;
	element_type: string;
	match_value: string;
	match_mode: string;
	negate: boolean;
	case_sensitive: boolean;
	position: number;
}

export interface MatcherGroupInput {
	matchers: MatcherInput[];
}

export interface MatcherInput {
	element_type: string;
	match_value: string;
	match_mode?: string;
	negate?: boolean;
	case_sensitive?: boolean;
}

export interface LabelListEntry {
	concept_id: string;
	name: string;
	description: string | null;
	concept_type: string | null;
	matcher_count: number;
	confirmed_count: number;
	suggested_count: number;
}

export interface LabelDetail {
	concept_id: string;
	name: string;
	description: string | null;
	matcher_groups: MatcherGroup[];
	confirmed_count: number;
	suggested_count: number;
	communications: CommunicationSummaryItem[];
}

export interface CommunicationSummaryItem {
	id: string;
	communication_type: string;
	sender: string;
	subject: string | null;
	preview: string | null;
	date: string;
	status: string;
	confidence: number | null;
	communication_concept_id: string;
}

export interface ScanResult {
	new_suggestions: number;
	total_scanned: number;
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

	async logoutAll(): Promise<void> {
		await this.request('/auth/logout-all', { method: 'POST' });
		this.logout();
	}

	async getCurrentUser(): Promise<User> {
		return this.request('/auth/me');
	}

	async updateUserProfile(payload: {
		name?: string;
		email?: string;
		preferences?: Record<string, any>;
	}): Promise<User> {
		return this.request('/auth/me', {
			method: 'PUT',
			body: JSON.stringify(payload)
		});
	}

	async changePassword(currentPassword: string, newPassword: string): Promise<void> {
		await this.request('/auth/change-password', {
			method: 'POST',
			body: JSON.stringify({
				current_password: currentPassword,
				new_password: newPassword
			})
		});
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
		// Backend only accepts query parameter currently
		// TODO: Enhance backend to support filters (tags, etc.)
		return this.request('/contacts/search', {
			method: 'POST',
			body: JSON.stringify({ query })
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

	async getConceptChildren(parentId: string): Promise<Concept[]> {
		return this.request(`/concepts/${parentId}/children`);
	}

	async addConceptKeyword(conceptId: string, keyword: string): Promise<ConceptKeyword> {
		return this.request(`/concepts/${conceptId}/keywords`, {
			method: 'POST',
			body: JSON.stringify({ keyword })
		});
	}

	async removeConceptKeyword(conceptId: string, keywordId: string): Promise<void> {
		return this.request(`/concepts/${conceptId}/keywords/${keywordId}`, {
			method: 'DELETE'
		});
	}

	// Picks
	async getPicks(): Promise<Pick[]> {
		return this.request('/picks');
	}

	async getPick(id: string): Promise<Pick> {
		return this.request(`/picks/${id}`);
	}

	async createPick(pick: Partial<Pick>): Promise<Pick> {
		return this.request('/picks', {
			method: 'POST',
			body: JSON.stringify(pick)
		});
	}

	async updatePick(id: string, updates: Partial<Pick>): Promise<Pick> {
		return this.request(`/picks/${id}`, {
			method: 'PUT',
			body: JSON.stringify(updates)
		});
	}

	async deletePick(id: string): Promise<void> {
		return this.request(`/picks/${id}`, { method: 'DELETE' });
	}

	// Locations
	async getLocations(): Promise<Location[]> {
		return this.request('/locations');
	}

	async getLocation(id: string): Promise<Location> {
		return this.request(`/locations/${id}`);
	}

	async createLocation(location: Partial<Location>): Promise<Location> {
		return this.request('/locations', {
			method: 'POST',
			body: JSON.stringify(location)
		});
	}

	async updateLocation(id: string, updates: Partial<Location>): Promise<Location> {
		return this.request(`/locations/${id}`, {
			method: 'PUT',
			body: JSON.stringify(updates)
		});
	}

	async deleteLocation(id: string): Promise<void> {
		return this.request(`/locations/${id}`, { method: 'DELETE' });
	}

	// Relationships
	async getRelationshipTypes(): Promise<RelationshipType[]> {
		return this.request('/relationships/types');
	}

	async createRelationshipType(name: string, description?: string): Promise<RelationshipType> {
		return this.request('/relationships/types', {
			method: 'POST',
			body: JSON.stringify({ name, description })
		});
	}

	async deleteRelationshipType(id: string): Promise<void> {
		return this.request(`/relationships/types/${id}`, { method: 'DELETE' });
	}

	async createRelationship(rel: {
		source_type: string;
		source_id: string;
		relationship_type_id: string;
		target_type: string;
		target_id: string;
		metadata?: Record<string, any>;
	}): Promise<EntityRelationship> {
		return this.request('/relationships', {
			method: 'POST',
			body: JSON.stringify(rel)
		});
	}

	async deleteRelationship(id: string): Promise<void> {
		return this.request(`/relationships/${id}`, { method: 'DELETE' });
	}

	async getRelationshipsBySource(sourceType: string, sourceId: string): Promise<EntityRelationship[]> {
		return this.request(`/relationships/source/${sourceType}/${sourceId}`);
	}

	async getRelationshipsByTarget(targetType: string, targetId: string): Promise<EntityRelationship[]> {
		return this.request(`/relationships/target/${targetType}/${targetId}`);
	}

	// Matches
	async getContactMatches(contactId: string): Promise<MatchResult[]> {
		return this.request(`/matches/contact/${contactId}`);
	}

	async getPickMatches(pickId: string): Promise<MatchResult[]> {
		return this.request(`/matches/pick/${pickId}`);
	}

	// Communication Concepts
	async getCommunicationConcepts(commType: string, commId: string): Promise<CommunicationConcept[]> {
		return this.request(`/communications/${commType}/${commId}/concepts`);
	}

	async detectConcepts(commType: string, commId: string, text: string): Promise<CommunicationConcept[]> {
		return this.request(`/communications/${commType}/${commId}/concepts/detect`, {
			method: 'POST',
			body: JSON.stringify({ text })
		});
	}

	async getPendingConceptSuggestions(): Promise<CommunicationConcept[]> {
		return this.request('/communication-concepts/pending');
	}

	async updateConceptSuggestion(id: string, status: 'confirmed' | 'denied'): Promise<CommunicationConcept> {
		return this.request(`/communication-concepts/${id}`, {
			method: 'PUT',
			body: JSON.stringify({ status })
		});
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
	// NOTE: Backend only supports GET /notes/contact/:id, not GET /notes (yet)
	async getNotes(): Promise<Note[]> {
		// TODO: Add backend endpoint for listing all notes
		// For now, return empty array - individual note fetching works via getContactNotes
		try {
			return await this.request('/notes');
		} catch {
			return [];
		}
	}

	async getContactNotes(contactId: string): Promise<Note[]> {
		return this.request(`/notes/contact/${contactId}`);
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
		entityId: string
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
		// NOTE: uploaded_by is now derived from authenticated session on server

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
	// NOTE: Backend has /shares/sent and /shares/received, not /shares
	async getShares(): Promise<ShareInvite[]> {
		const [sent, received] = await Promise.all([
			this.request('/shares/sent') as Promise<ShareInvite[]>,
			this.request('/shares/received') as Promise<ShareInvite[]>
		]);
		return [...sent, ...received];
	}

	async getSentShares(): Promise<ShareInvite[]> {
		return this.request('/shares/sent');
	}

	async getReceivedShares(): Promise<ShareInvite[]> {
		return this.request('/shares/received');
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
		return this.request('/search-history');
	}

	async clearSearchHistory(): Promise<void> {
		return this.request('/search-history', { method: 'DELETE' });
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

	// SMS History
	async importAndroidSms(file: File): Promise<{
		file_type: string;
		total_records: number;
		imported: number;
		skipped: number;
		elapsed_seconds: number;
	}> {
		const formData = new FormData();
		formData.append('file', file);

		const res = await fetch(`${this.baseUrl}/import/android-sms`, {
			method: 'POST',
			headers: this.getAuthHeaders(false),
			body: formData
		});

		if (!res.ok) {
			return await this.handleErrorResponse(res, '/import/android-sms');
		}

		return res.json();
	}

	async getSmsConversations(): Promise<{
		conversations: Array<{
			phone_number: string;
			contact_name: string | null;
			contact_id: string | null;
			message_count: number;
			last_message_date: number;
			last_message_preview: string;
		}>;
		total: number;
	}> {
		return this.request('/sms-history/conversations');
	}

	async getSmsConversation(phone: string): Promise<{
		messages: Array<{
			id: string;
			contact_id: string | null;
			phone_number: string;
			contact_name: string | null;
			message_date: number;
			message_type: number;
			subject: string | null;
			body: string;
			readable_date: string;
			thread_id: number | null;
			read_status: number;
			imported_at: string;
			source_file: string | null;
		}>;
		total: number;
		phone_number: string;
	}> {
		return this.request(`/sms-history/conversation/${encodeURIComponent(phone)}`);
	}

	async getSmsStats(): Promise<{
		total_messages: number;
		unique_conversations: number;
		earliest_message_date: number | null;
		latest_message_date: number | null;
		messages_with_contacts: number;
		messages_without_contacts: number;
	}> {
		return this.request('/sms-history/stats');
	}

	async importSmsDirectory(path: string): Promise<{
		path: string;
		total_files: number;
		files_imported: number;
		files_failed: number;
		total_records: number;
		total_imported: number;
		total_skipped: number;
		elapsed_seconds: number;
		results: Array<{
			file_name: string;
			file_type: string;
			total_records: number;
			imported: number;
			skipped: number;
			status: string;
			error: string | null;
		}>;
	}> {
		return this.request('/import/android-sms/directory', {
			method: 'POST',
			body: JSON.stringify({ path })
		});
	}

	async deleteSmsSource(filename: string): Promise<{ deleted: number; source_file: string }> {
		return this.request(`/sms-history/source/${encodeURIComponent(filename)}`, {
			method: 'DELETE'
		});
	}

	// Email History
	async fetchEmails(config: {
		server: string;
		port?: number;
		username: string;
		password: string;
		folder?: string;
		max_emails?: number;
	}): Promise<{
		fetched: number;
		stored: number;
		skipped: number;
		server: string;
		folder: string;
	}> {
		return this.request('/email/fetch', {
			method: 'POST',
			body: JSON.stringify(config)
		});
	}

	async getEmailConversations(): Promise<{
		conversations: Array<{
			from_address: string;
			from_name: string | null;
			contact_id: string | null;
			message_count: number;
			last_message_date: number;
			last_subject: string;
			last_body_preview: string;
		}>;
		total: number;
	}> {
		return this.request('/email/conversations');
	}

	async getEmailConversation(address: string): Promise<{
		messages: Array<{
			id: string;
			contact_id: string | null;
			from_address: string;
			from_name: string | null;
			to_address: string;
			cc: string | null;
			subject: string;
			body_text: string | null;
			body_html: string | null;
			message_date: number;
			message_type: number;
			message_id: string | null;
			read_status: number;
			has_attachments: number;
			folder: string | null;
			imported_at: string;
			source_server: string | null;
		}>;
		total: number;
		address: string;
	}> {
		return this.request(`/email/conversation/${encodeURIComponent(address)}`);
	}

	async getEmailStats(): Promise<{
		total_emails: number;
		unique_senders: number;
		earliest_message_date: number | null;
		latest_message_date: number | null;
		received_count: number;
		sent_count: number;
	}> {
		return this.request('/email/stats');
	}

	async searchEmails(q: string, limit?: number): Promise<{
		results: Array<{
			id: string;
			from_address: string;
			from_name: string | null;
			to_address: string;
			subject: string;
			body_preview: string | null;
			message_date: number;
			message_type: number;
			folder: string | null;
		}>;
		total: number;
		query: string;
	}> {
		const params = new URLSearchParams({ q });
		if (limit) params.set('limit', limit.toString());
		return this.request(`/email/search?${params.toString()}`);
	}

	// Email Triage
	async startEmailTriage(config: {
		server: string;
		folder?: string;
		block_size?: number;
	}): Promise<{
		session_id: string;
		status: string;
		total_fetched: number;
		block_size: number;
		total_blocks: number;
		current_block: number;
		proposals: Array<{
			concept_id: string;
			name: string;
			description: string;
			keywords: string[];
			email_count: number;
			sample_subjects: string[];
			sample_senders: string[];
			confidence: number;
		}>;
	}> {
		return this.request('/email/triage/start', {
			method: 'POST',
			body: JSON.stringify(config)
		});
	}

	async getTriageSession(sessionId: string): Promise<any> {
		return this.request(`/email/triage/${sessionId}`);
	}

	async getTriageProposals(sessionId: string): Promise<any> {
		return this.request(`/email/triage/${sessionId}/proposals`);
	}

	async reviewTriageDomains(sessionId: string, review: {
		approved: string[];
		denied: string[];
	}): Promise<any> {
		return this.request(`/email/triage/${sessionId}/review`, {
			method: 'POST',
			body: JSON.stringify(review)
		});
	}

	async continueTriage(sessionId: string, options: {
		auto_apply?: boolean;
	}): Promise<any> {
		return this.request(`/email/triage/${sessionId}/continue`, {
			method: 'POST',
			body: JSON.stringify(options)
		});
	}

	// Email Domains
	async getEmailDomains(): Promise<{
		domains: Array<{
			concept_id: string;
			name: string;
			description: string | null;
			keywords: string[];
			email_count: number;
			unique_senders: number;
			earliest_date: number | null;
			latest_date: number | null;
		}>;
		total: number;
	}> {
		return this.request('/email/domains');
	}

	async getDomainEmails(conceptId: string): Promise<{
		concept_id: string;
		emails: Array<{
			id: string;
			from_address: string;
			from_name: string | null;
			to_address: string;
			subject: string;
			body_text: string | null;
			body_html: string | null;
			message_date: number;
			message_type: number;
			read_status: number;
			has_attachments: number;
			folder: string | null;
		}>;
		total: number;
	}> {
		return this.request(`/email/domains/${conceptId}/emails`);
	}

	async getDomainStats(conceptId: string): Promise<{
		concept_id: string;
		name: string;
		email_count: number;
		unique_senders: number;
		earliest_date: number | null;
		latest_date: number | null;
		top_senders: Array<{
			address: string;
			name: string | null;
			count: number;
		}>;
	}> {
		return this.request(`/email/domains/${conceptId}/stats`);
	}

	async filterEmailsByDomains(filter: {
		domain_ids: string[];
		concept_ids?: string[];
		mode: string;
		limit?: number;
		offset?: number;
	}): Promise<{
		emails: Array<{
			id: string;
			from_address: string;
			from_name: string | null;
			subject: string;
			message_date: number;
			body_preview: string | null;
			message_type: number;
			folder: string | null;
			domains: Array<{
				concept_id: string;
				concept_name: string;
			}>;
		}>;
		total: number;
		mode: string;
		overlaps: Array<{
			domain_ids: string[];
			count: number;
		}>;
	}> {
		return this.request('/email/domains/filter', {
			method: 'POST',
			body: JSON.stringify(filter)
		});
	}

	// Manual Domain Assignment
	async searchEmailDomains(q: string): Promise<{
		domains: Array<{ concept_id: string; name: string; keywords: string[] }>;
	}> {
		return this.request(`/email/domains/search?q=${encodeURIComponent(q)}`);
	}

	async createEmailDomain(
		name: string,
		description?: string,
		keywords?: string[]
	): Promise<{ concept_id: string; name: string }> {
		return this.request('/email/domains/create', {
			method: 'POST',
			body: JSON.stringify({ name, description, keywords })
		});
	}

	async getEmailDomainAssignments(
		emailId: string
	): Promise<{
		domains: Array<{
			concept_id: string;
			name: string;
			status: string;
			detection_method: string;
		}>;
	}> {
		return this.request(`/email/${emailId}/domains`);
	}

	async assignDomainsToEmail(
		emailId: string,
		domainNames: string[]
	): Promise<{
		assigned: Array<{
			concept_id: string;
			name: string;
			status: string;
			detection_method: string;
		}>;
		created: string[];
	}> {
		return this.request(`/email/${emailId}/domains`, {
			method: 'POST',
			body: JSON.stringify({ domain_names: domainNames })
		});
	}

	async unlinkDomainFromEmail(emailId: string, conceptId: string): Promise<void> {
		return this.request(`/email/${emailId}/domains/${conceptId}`, {
			method: 'DELETE'
		});
	}

	async bulkAssignDomains(
		emailIds: string[],
		domainNames: string[]
	): Promise<{ assigned_count: number; created_domains: string[] }> {
		return this.request('/email/bulk-assign-domains', {
			method: 'POST',
			body: JSON.stringify({ email_ids: emailIds, domain_names: domainNames })
		});
	}

	// IMAP Account Management
	async getImapAccounts(): Promise<{
		accounts: Array<{
			id: string;
			name: string;
			server: string;
			port: number;
			username: string;
			default_folder: string;
			max_emails: number;
			last_fetched_at: string | null;
			created_at: string;
			updated_at: string;
		}>;
	}> {
		return this.request('/imap-accounts');
	}

	async createImapAccount(data: {
		name: string;
		server: string;
		port?: number;
		username: string;
		password: string;
		default_folder?: string;
		max_emails?: number;
	}): Promise<{
		id: string;
		name: string;
		server: string;
		port: number;
		username: string;
		default_folder: string;
		max_emails: number;
		last_fetched_at: string | null;
		created_at: string;
		updated_at: string;
	}> {
		return this.request('/imap-accounts', {
			method: 'POST',
			body: JSON.stringify(data)
		});
	}

	async updateImapAccount(id: string, data: {
		name?: string;
		server?: string;
		port?: number;
		username?: string;
		password?: string;
		default_folder?: string;
		max_emails?: number;
	}): Promise<any> {
		return this.request(`/imap-accounts/${id}`, {
			method: 'PUT',
			body: JSON.stringify(data)
		});
	}

	async deleteImapAccount(id: string): Promise<void> {
		await this.request(`/imap-accounts/${id}`, {
			method: 'DELETE'
		});
	}

	async fetchFromImapAccount(id: string, overrides?: {
		folder?: string;
		max_emails?: number;
	}): Promise<{
		fetched: number;
		stored: number;
		skipped: number;
		server: string;
		folder: string;
	}> {
		return this.request(`/imap-accounts/${id}/fetch`, {
			method: 'POST',
			body: overrides ? JSON.stringify(overrides) : undefined
		});
	}

	// Sender-level domain assignment
	async getSenderDomains(address: string): Promise<{
		domains: Array<{ concept_id: string; name: string; status: string; detection_method: string }>;
	}> {
		return this.request(`/email/sender/${encodeURIComponent(address)}/domains`);
	}

	async assignDomainsToSender(address: string, domainNames: string[]): Promise<{
		assigned_count: number;
		message_count: number;
		created_domains: string[];
	}> {
		return this.request(`/email/sender/${encodeURIComponent(address)}/domains`, {
			method: 'POST',
			body: JSON.stringify({ domain_names: domainNames })
		});
	}

	async getSmsSenderDomains(phone: string): Promise<{
		domains: Array<{ concept_id: string; name: string; status: string; detection_method: string }>;
	}> {
		return this.request(`/sms/sender/${encodeURIComponent(phone)}/domains`);
	}

	async assignDomainsToSmsSender(phone: string, domainNames: string[]): Promise<{
		assigned_count: number;
		message_count: number;
		created_domains: string[];
	}> {
		return this.request(`/sms/sender/${encodeURIComponent(phone)}/domains`, {
			method: 'POST',
			body: JSON.stringify({ domain_names: domainNames })
		});
	}

	// Contact Reconciliation
	async reconcileContacts(): Promise<{
		contacts_created: number;
		contacts_matched: number;
		email_messages_linked: number;
		sms_messages_linked: number;
		errors: string[];
	}> {
		return this.request('/contacts/reconcile', { method: 'POST' });
	}

	async getUnlinkedStats(): Promise<{
		unlinked_email_senders: number;
		unlinked_sms_senders: number;
		total_unlinked_senders: number;
	}> {
		return this.request('/contacts/unlinked-stats');
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

	// AI Settings
	async getAiStatus(): Promise<{ configured: boolean; provider: string; model: string }> {
		return this.request('/settings/ai');
	}

	async updateAiKey(apiKey: string): Promise<{ configured: boolean; message: string }> {
		return this.request('/settings/ai', {
			method: 'PUT',
			body: JSON.stringify({ api_key: apiKey })
		});
	}

	// System Updates
	async getVersion(): Promise<{ version: string; build_date: string; commit_hash?: string }> {
		return this.request('/system/version');
	}

	async checkUpdates(): Promise<{
		current_version: string;
		latest_version: string;
		update_available: boolean;
		release_url?: string;
		release_notes?: string;
		download_url?: string;
		published_at?: string;
	}> {
		return this.request('/system/updates/check');
	}

	// WebSocket
	connectWebSocket() {
		if (!this.token || this.ws) return;

		try {
			const wsUrl = this.baseUrl.replace('http', 'ws').replace('/api', '/ws');
			this.ws = new WebSocket(`${wsUrl}?token=${this.token}`);

			this.ws.onmessage = (event) => {
				try {
					const data = JSON.parse(event.data);
					const listeners = this.wsListeners.get(data.type) || new Set();
					listeners.forEach(listener => {
						try {
							listener(data.payload);
						} catch (err) {
							console.error('WebSocket listener error:', err);
						}
					});
				} catch (err) {
					console.error('WebSocket message parse error:', err);
				}
			};

			this.ws.onerror = (error) => {
				// Log but don't throw - WebSocket errors shouldn't block navigation
				console.warn('WebSocket error (non-critical):', error);
			};

			this.ws.onclose = () => {
				this.ws = null;
				// Attempt reconnection after 5 seconds
				setTimeout(() => {
					if (this.token) this.connectWebSocket();
				}, 5000);
			};
		} catch (error) {
			// WebSocket connection failed - continue without it
			console.warn('WebSocket connection failed (continuing without real-time updates):', error);
			this.ws = null;
		}
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

	// --- Label Matchers ---

	async getConceptMatchers(conceptId: string): Promise<{ groups: MatcherGroup[] }> {
		return this.request(`/concepts/${conceptId}/matchers`);
	}

	async setConceptMatchers(conceptId: string, groups: MatcherGroupInput[], autoScan?: boolean): Promise<{ groups: MatcherGroup[]; scan_count: number }> {
		const qs = autoScan ? '?auto_scan=true' : '';
		return this.request(`/concepts/${conceptId}/matchers${qs}`, {
			method: 'POST',
			body: JSON.stringify({ groups })
		});
	}

	async addMatcherGroup(conceptId: string, matchers: MatcherInput[]): Promise<{ group: MatcherGroup }> {
		return this.request(`/concepts/${conceptId}/matchers/groups`, {
			method: 'POST',
			body: JSON.stringify({ matchers })
		});
	}

	async deleteMatcherGroup(conceptId: string, groupId: string): Promise<void> {
		return this.request(`/concepts/${conceptId}/matchers/groups/${groupId}`, {
			method: 'DELETE'
		});
	}

	async addMatcherToGroup(conceptId: string, groupId: string, matcher: MatcherInput): Promise<any> {
		return this.request(`/concepts/${conceptId}/matchers/groups/${groupId}/matchers`, {
			method: 'POST',
			body: JSON.stringify(matcher)
		});
	}

	async updateMatcher(conceptId: string, groupId: string, matcherId: string, matcher: MatcherInput): Promise<any> {
		return this.request(`/concepts/${conceptId}/matchers/groups/${groupId}/matchers/${matcherId}`, {
			method: 'PUT',
			body: JSON.stringify(matcher)
		});
	}

	async deleteMatcher(conceptId: string, groupId: string, matcherId: string): Promise<void> {
		return this.request(`/concepts/${conceptId}/matchers/groups/${groupId}/matchers/${matcherId}`, {
			method: 'DELETE'
		});
	}

	async scanConcept(conceptId: string): Promise<ScanResult> {
		return this.request(`/concepts/${conceptId}/scan`, { method: 'POST' });
	}

	async scanAllLabels(): Promise<ScanResult> {
		return this.request('/labels/scan-all', { method: 'POST' });
	}

	async listLabels(): Promise<{ labels: LabelListEntry[] }> {
		return this.request('/labels');
	}

	async getLabelDetail(conceptId: string, params?: { status?: string; limit?: number; offset?: number; comm_type?: string }): Promise<LabelDetail> {
		const searchParams = new URLSearchParams();
		if (params?.status) searchParams.set('status', params.status);
		if (params?.limit) searchParams.set('limit', params.limit.toString());
		if (params?.offset) searchParams.set('offset', params.offset.toString());
		if (params?.comm_type) searchParams.set('comm_type', params.comm_type);
		const qs = searchParams.toString();
		return this.request(`/labels/${conceptId}${qs ? '?' + qs : ''}`);
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
				// Dispatch event instead of hard redirect to avoid navigation freeze
				if (typeof window !== 'undefined') {
					window.dispatchEvent(new CustomEvent('auth:logout'));
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
