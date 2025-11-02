// Complete type definitions for all entities

export interface User {
	id: string;
	email: string;
	name: string;
	email_verified: boolean;
	active: boolean;
	preferences: Record<string, any>;
	created_at: string;
	updated_at: string;
	last_login_at?: string;
}

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
	groups: string[];
	created_at: string;
	updated_at: string;
	created_by: string;
	version: number;
	last_synced_at?: string;
	metadata: Record<string, any>;
}

export interface SocialHandle {
	platform: string;
	handle: string;
	url?: string;
}

export interface Group {
	id: string;
	name: string;
	description?: string;
	contact_ids: string[];
	created_at: string;
	updated_at: string;
	created_by: string;
}

export interface Concept {
	id: string;
	name: string;
	description?: string;
	related_contacts: string[];
	related_projects: string[];
	tags: string[];
	created_at: string;
	updated_at: string;
	created_by: string;
}

export interface Project {
	id: string;
	name: string;
	description?: string;
	status: ProjectStatus;
	contacts: string[];
	tags: string[];
	attachment_ids: string[];
	created_at: string;
	updated_at: string;
	created_by: string;
	version: number;
	last_synced_at?: string;
}

export type ProjectStatus = 'Active' | 'Completed' | 'Archived' | 'OnHold';

export interface CalendarEvent {
	id: string;
	title: string;
	description?: string;
	start_time: string;
	end_time: string;
	all_day: boolean;
	contacts: string[];
	location?: string;
	recurrence?: RecurrenceRule;
	reminders: Reminder[];
	attachment_ids: string[];
	created_at: string;
	updated_at: string;
	created_by: string;
	version: number;
}

export interface RecurrenceRule {
	frequency: 'Daily' | 'Weekly' | 'Monthly' | 'Yearly';
	interval: number;
	count?: number;
	until?: string;
	by_day: string[];
}

export interface Reminder {
	id: string;
	minutes_before: number;
	method: 'Notification' | 'Email' | 'SMS';
}

export interface Note {
	id: string;
	contact_id?: string;
	project_id?: string;
	title: string;
	content: string;
	attachment_ids: string[];
	created_at: string;
	updated_at: string;
	created_by: string;
	version: number;
	last_synced_at?: string;
}

export interface Attachment {
	id: string;
	filename: string;
	content_type: string;
	size_bytes: number;
	storage_path: string;
	thumbnail_path?: string;
	entity_type: AttachmentEntityType;
	entity_id: string;
	uploaded_by: string;
	metadata: Record<string, any>;
	created_at: string;
}

export type AttachmentEntityType = 'Contact' | 'Project' | 'Note' | 'CalendarEvent' | 'Communication';

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

export interface Communication {
	id: string;
	contact_id: string;
	communication_type: CommunicationType;
	direction: CommunicationDirection;
	timestamp: string;
	content?: string;
	duration_seconds?: number;
	phone_number?: string;
	thread_id?: string;
	status: CommunicationHistoryStatus;
	metadata: Record<string, any>;
	created_at: string;
	updated_at: string;
}

export type CommunicationType = 'Sms' | 'Call' | 'Email' | 'VideoCall' | 'VoiceMail';
export type CommunicationDirection = 'Inbound' | 'Outbound' | 'Missed';
export type CommunicationHistoryStatus = 'Completed' | 'Failed' | 'Blocked' | 'Rejected';

export interface ShareInvite {
	id: string;
	entity_type: ShareEntityType;
	entity_id: string;
	shared_by: string;
	shared_with_email: string;
	shared_with_user?: string;
	permissions: Permission[];
	accepted: boolean;
	accepted_at?: string;
	revoked: boolean;
	revoked_at?: string;
	created_at: string;
	expires_at?: string;
}

export type ShareEntityType = 'Contact' | 'Project' | 'Note' | 'CalendarEvent' | 'Group';
export type Permission = 'Read' | 'Write' | 'Share' | 'Delete';

export interface AiInsight {
	id: string;
	entity_type: AiInsightEntityType;
	entity_id: string;
	insight_type: AiInsightType;
	content: string;
	confidence: number;
	prompt_template: string;
	response_cached: boolean;
	feedback?: AiInsightFeedback;
	applied: boolean;
	applied_at?: string;
	created_at: string;
	created_by: string;
}

export type AiInsightEntityType = 'Contact' | 'Project' | 'Note' | 'Communication' | 'CalendarEvent';
export type AiInsightType = 'TagSuggestion' | 'ChannelRecommendation' | 'NextAction' | 'RelationshipStrength' | 'ContentSummary';

export interface AiInsightFeedback {
	helpful: boolean;
	comment?: string;
	submitted_at: string;
}

export interface SearchHistory {
	id: string;
	user_id: string;
	query: string;
	filters: Record<string, any>;
	result_count: number;
	clicked_result_id?: string;
	created_at: string;
}

export interface WorkerMetrics {
	id: string;
	task_name: string;
	success_count: number;
	failure_count: number;
	last_run_at?: string;
	last_success_at?: string;
	last_failure_at?: string;
	last_error?: string;
	average_duration_ms?: number;
	created_at: string;
	updated_at: string;
}

export interface ImportPreview {
	headers: string[];
	rows: string[][];
	field_mappings: Record<string, string>;
	validation_errors: ValidationError[];
}

export interface ValidationError {
	row: number;
	field: string;
	error: string;
}

export interface AuthResponse {
	user: User;
	token: string;
}

export interface DashboardSummary {
	contacts_count: number;
	projects_active: number;
	upcoming_events: CalendarEvent[];
	recent_communications: CommunicationAttempt[];
	pending_invites: ShareInvite[];
	ai_suggestions: AiInsight[];
	nag_reminders: NagReminder[];
}

export interface NagReminder {
	id: string;
	contact_id: string;
	contact_name: string;
	days_since_contact: number;
	last_contact_date?: string;
}