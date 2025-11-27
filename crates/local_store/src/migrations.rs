/// SQLite schema (default)
pub const SCHEMA_SQLITE: &str = SCHEMA;

/// PostgreSQL schema with native types
pub const SCHEMA_POSTGRES: &str = r#"
-- Users table (PostgreSQL)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    api_token TEXT,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    active BOOLEAN NOT NULL DEFAULT true,
    preferences JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    last_login_at TIMESTAMPTZ
);

-- Contacts table with versioning (PostgreSQL)
CREATE TABLE IF NOT EXISTS contacts (
    id UUID PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT,
    email TEXT,
    phone TEXT,
    organization TEXT,
    title TEXT,
    notes TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    version INTEGER NOT NULL DEFAULT 1,
    last_synced_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS social_handles (
    id SERIAL PRIMARY KEY,
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    platform TEXT NOT NULL,
    handle TEXT NOT NULL,
    url TEXT
);

-- Tags
CREATE TABLE IF NOT EXISTS tags (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS contact_tags (
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (contact_id, tag_id)
);

-- Projects with status and versioning
CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'Active',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    version INTEGER NOT NULL DEFAULT 1,
    last_synced_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS project_contacts (
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    PRIMARY KEY (project_id, contact_id)
);

CREATE TABLE IF NOT EXISTS project_tags (
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (project_id, tag_id)
);

-- Groups
CREATE TABLE IF NOT EXISTS groups (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS group_members (
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    PRIMARY KEY (group_id, contact_id)
);

CREATE TABLE IF NOT EXISTS contact_groups (
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    group_id UUID NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    PRIMARY KEY (contact_id, group_id)
);

-- Concepts
CREATE TABLE IF NOT EXISTS concepts (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    tags JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS concept_contacts (
    concept_id UUID NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    PRIMARY KEY (concept_id, contact_id)
);

CREATE TABLE IF NOT EXISTS concept_projects (
    concept_id UUID NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    PRIMARY KEY (concept_id, project_id)
);

-- Calendar Events with recurrence and reminders
CREATE TABLE IF NOT EXISTS calendar_events (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    start_time TIMESTAMPTZ NOT NULL,
    end_time TIMESTAMPTZ NOT NULL,
    all_day BOOLEAN NOT NULL DEFAULT false,
    location TEXT,
    recurrence TEXT,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    version INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS event_contacts (
    event_id UUID NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    PRIMARY KEY (event_id, contact_id)
);

CREATE TABLE IF NOT EXISTS event_reminders (
    id UUID PRIMARY KEY,
    event_id UUID NOT NULL REFERENCES calendar_events(id) ON DELETE CASCADE,
    minutes_before INTEGER NOT NULL,
    method TEXT NOT NULL
);

-- Notes with versioning
CREATE TABLE IF NOT EXISTS notes (
    id UUID PRIMARY KEY,
    contact_id UUID REFERENCES contacts(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id),
    version INTEGER NOT NULL DEFAULT 1,
    last_synced_at TIMESTAMPTZ
);

-- Attachments (universal, linked to any entity)
CREATE TABLE IF NOT EXISTS attachments (
    id UUID PRIMARY KEY,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    storage_path TEXT NOT NULL,
    thumbnail_path TEXT,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    uploaded_by UUID NOT NULL REFERENCES users(id),
    checksum TEXT NOT NULL,
    encrypted BOOLEAN NOT NULL DEFAULT false,
    scan_status TEXT NOT NULL DEFAULT 'Pending',
    scan_details TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL
);

-- Communication attempts (for outgoing scheduled communications)
CREATE TABLE IF NOT EXISTS communication_attempts (
    id UUID PRIMARY KEY,
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    method TEXT NOT NULL,
    subject TEXT,
    message TEXT NOT NULL,
    status TEXT NOT NULL,
    scheduled_at TIMESTAMPTZ,
    attempted_at TIMESTAMPTZ,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL
);

-- Communications (historical records of SMS, calls, emails that occurred)
CREATE TABLE IF NOT EXISTS communications (
    id UUID PRIMARY KEY,
    contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    communication_type TEXT NOT NULL,
    direction TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    content TEXT,
    duration_seconds INTEGER,
    phone_number TEXT,
    thread_id TEXT,
    status TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Share invites with acceptance tracking
CREATE TABLE IF NOT EXISTS share_invites (
    id UUID PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    shared_by UUID NOT NULL REFERENCES users(id),
    shared_with_email TEXT NOT NULL,
    shared_with_user UUID REFERENCES users(id),
    permissions TEXT NOT NULL,
    accepted BOOLEAN NOT NULL DEFAULT false,
    accepted_at TIMESTAMPTZ,
    revoked BOOLEAN NOT NULL DEFAULT false,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ
);

-- Resource ACLs
CREATE TABLE IF NOT EXISTS resource_acls (
    id UUID PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    owner_id UUID NOT NULL REFERENCES users(id),
    grants JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Audit logs
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    action TEXT NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id),
    changes JSONB NOT NULL DEFAULT '{}',
    ip_address TEXT,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL
);

-- Conflict resolution
CREATE TABLE IF NOT EXISTS conflict_resolutions (
    id UUID PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    local_version INTEGER NOT NULL,
    remote_version INTEGER NOT NULL,
    local_updated_at TIMESTAMPTZ NOT NULL,
    remote_updated_at TIMESTAMPTZ NOT NULL,
    local_changes JSONB NOT NULL,
    remote_changes JSONB NOT NULL,
    resolution_strategy TEXT NOT NULL,
    resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL
);

-- Search history
CREATE TABLE IF NOT EXISTS search_history (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    query TEXT NOT NULL,
    filters JSONB NOT NULL DEFAULT '{}',
    result_count INTEGER NOT NULL,
    result_ids JSONB NOT NULL DEFAULT '[]',
    clicked_result_id UUID,
    privacy_mode BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL
);

-- AI Insights (expanded from suggestions)
CREATE TABLE IF NOT EXISTS ai_insights (
    id UUID PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id UUID NOT NULL,
    insight_type TEXT NOT NULL,
    content TEXT NOT NULL,
    confidence REAL NOT NULL,
    prompt_template TEXT NOT NULL,
    response_cached BOOLEAN NOT NULL DEFAULT false,
    feedback_helpful BOOLEAN,
    feedback_comment TEXT,
    feedback_submitted_at TIMESTAMPTZ,
    applied BOOLEAN NOT NULL DEFAULT false,
    applied_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL,
    created_by UUID NOT NULL REFERENCES users(id)
);

-- AI Interactions (logs all AI API calls with caching/retry metadata)
CREATE TABLE IF NOT EXISTS ai_interactions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    interaction_type TEXT NOT NULL,
    prompt TEXT NOT NULL,
    response TEXT NOT NULL,
    confidence REAL NOT NULL,
    model TEXT NOT NULL,
    entity_type TEXT,
    entity_id UUID,
    feedback_helpful BOOLEAN,
    feedback_applied BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL,
    feedback_at TIMESTAMPTZ
);

-- Legacy AI suggestions (kept for backward compat)
CREATE TABLE IF NOT EXISTS ai_suggestions (
    id UUID PRIMARY KEY,
    contact_id UUID REFERENCES contacts(id) ON DELETE CASCADE,
    suggestion_type TEXT NOT NULL,
    content TEXT NOT NULL,
    confidence REAL NOT NULL,
    applied BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL
);

-- SMS History table for imported Android SMS (PostgreSQL)
CREATE TABLE IF NOT EXISTS sms_history (
    id UUID PRIMARY KEY,
    contact_id UUID REFERENCES contacts(id) ON DELETE SET NULL,
    phone_number TEXT NOT NULL,
    normalized_phone TEXT NOT NULL,
    sender_name TEXT,
    message_type TEXT NOT NULL,
    content TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    read_status BOOLEAN NOT NULL DEFAULT false,
    thread_id TEXT,
    import_source TEXT NOT NULL DEFAULT 'manual',
    raw_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Email history table (PostgreSQL)
CREATE TABLE IF NOT EXISTS email_history (
    id UUID PRIMARY KEY,
    contact_id UUID REFERENCES contacts(id) ON DELETE SET NULL,
    from_address TEXT NOT NULL,
    to_addresses JSONB NOT NULL DEFAULT '[]',
    cc_addresses JSONB NOT NULL DEFAULT '[]',
    subject TEXT NOT NULL,
    body_text TEXT,
    body_html TEXT,
    message_id TEXT UNIQUE,
    in_reply_to TEXT,
    thread_id TEXT,
    timestamp TIMESTAMPTZ NOT NULL,
    folder TEXT NOT NULL DEFAULT 'INBOX',
    read_status BOOLEAN NOT NULL DEFAULT false,
    flagged BOOLEAN NOT NULL DEFAULT false,
    has_attachments BOOLEAN NOT NULL DEFAULT false,
    attachment_count INTEGER NOT NULL DEFAULT 0,
    importance_score INTEGER,
    is_important BOOLEAN NOT NULL DEFAULT false,
    is_urgent BOOLEAN NOT NULL DEFAULT false,
    ai_summary TEXT,
    ai_analysis JSONB,
    ai_analyzed_at TIMESTAMPTZ,
    raw_headers JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance (PostgreSQL)
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_api_token ON users(api_token);
CREATE INDEX IF NOT EXISTS idx_contacts_email ON contacts(email);
CREATE INDEX IF NOT EXISTS idx_contacts_phone ON contacts(phone);
CREATE INDEX IF NOT EXISTS idx_contacts_created_by ON contacts(created_by);
CREATE INDEX IF NOT EXISTS idx_social_handles_contact ON social_handles(contact_id);
CREATE INDEX IF NOT EXISTS idx_projects_created_by ON projects(created_by);
CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status);
CREATE INDEX IF NOT EXISTS idx_groups_created_by ON groups(created_by);
CREATE INDEX IF NOT EXISTS idx_concepts_created_by ON concepts(created_by);
CREATE INDEX IF NOT EXISTS idx_calendar_events_created_by ON calendar_events(created_by);
CREATE INDEX IF NOT EXISTS idx_calendar_events_start_time ON calendar_events(start_time);
CREATE INDEX IF NOT EXISTS idx_notes_contact ON notes(contact_id);
CREATE INDEX IF NOT EXISTS idx_notes_project ON notes(project_id);
CREATE INDEX IF NOT EXISTS idx_notes_created_by ON notes(created_by);
CREATE INDEX IF NOT EXISTS idx_attachments_entity ON attachments(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_attachments_uploaded_by ON attachments(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_comm_attempts_contact ON communication_attempts(contact_id);
CREATE INDEX IF NOT EXISTS idx_comm_attempts_status ON communication_attempts(status);
CREATE INDEX IF NOT EXISTS idx_communications_contact ON communications(contact_id);
CREATE INDEX IF NOT EXISTS idx_communications_timestamp ON communications(timestamp);
CREATE INDEX IF NOT EXISTS idx_communications_phone ON communications(phone_number);
CREATE INDEX IF NOT EXISTS idx_communications_thread ON communications(thread_id);
CREATE INDEX IF NOT EXISTS idx_communications_type ON communications(communication_type);
CREATE INDEX IF NOT EXISTS idx_share_invites_email ON share_invites(shared_with_email);
CREATE INDEX IF NOT EXISTS idx_share_invites_user ON share_invites(shared_with_user);
CREATE INDEX IF NOT EXISTS idx_share_invites_entity ON share_invites(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_resource_acls_entity ON resource_acls(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_conflict_resolutions_entity ON conflict_resolutions(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_search_history_user ON search_history(user_id);
CREATE INDEX IF NOT EXISTS idx_ai_insights_entity ON ai_insights(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_ai_insights_created_by ON ai_insights(created_by);
CREATE INDEX IF NOT EXISTS idx_ai_interactions_user ON ai_interactions(user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_ai_interactions_entity ON ai_interactions(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_sms_history_contact ON sms_history(contact_id);
CREATE INDEX IF NOT EXISTS idx_sms_history_phone ON sms_history(normalized_phone);
CREATE INDEX IF NOT EXISTS idx_sms_history_timestamp ON sms_history(timestamp);
CREATE INDEX IF NOT EXISTS idx_sms_history_thread ON sms_history(thread_id);
CREATE INDEX IF NOT EXISTS idx_email_history_contact ON email_history(contact_id);
CREATE INDEX IF NOT EXISTS idx_email_history_from ON email_history(from_address);
CREATE INDEX IF NOT EXISTS idx_email_history_timestamp ON email_history(timestamp);
CREATE INDEX IF NOT EXISTS idx_email_history_thread ON email_history(thread_id);
CREATE INDEX IF NOT EXISTS idx_email_history_message_id ON email_history(message_id);
"#;

/// Legacy schema constant for backwards compatibility
pub const SCHEMA: &str = r#"
-- Users table
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    api_token TEXT,
    email_verified INTEGER NOT NULL DEFAULT 0,
    active INTEGER NOT NULL DEFAULT 1,
    preferences TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_login_at TEXT
);

-- Contacts table with versioning
CREATE TABLE IF NOT EXISTS contacts (
    id TEXT PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT,
    email TEXT,
    phone TEXT,
    organization TEXT,
    title TEXT,
    notes TEXT,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    last_synced_at TEXT,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS social_handles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contact_id TEXT NOT NULL,
    platform TEXT NOT NULL,
    handle TEXT NOT NULL,
    url TEXT,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

-- Tags
CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    color TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS contact_tags (
    contact_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (contact_id, tag_id),
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Projects with status and versioning
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'Active',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    last_synced_at TEXT,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS project_contacts (
    project_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    PRIMARY KEY (project_id, contact_id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS project_tags (
    project_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (project_id, tag_id),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

-- Groups
CREATE TABLE IF NOT EXISTS groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS group_members (
    group_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    PRIMARY KEY (group_id, contact_id),
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS contact_groups (
    contact_id TEXT NOT NULL,
    group_id TEXT NOT NULL,
    PRIMARY KEY (contact_id, group_id),
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
);

-- Concepts
CREATE TABLE IF NOT EXISTS concepts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    tags TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS concept_contacts (
    concept_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    PRIMARY KEY (concept_id, contact_id),
    FOREIGN KEY (concept_id) REFERENCES concepts(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS concept_projects (
    concept_id TEXT NOT NULL,
    project_id TEXT NOT NULL,
    PRIMARY KEY (concept_id, project_id),
    FOREIGN KEY (concept_id) REFERENCES concepts(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Calendar Events with recurrence and reminders
CREATE TABLE IF NOT EXISTS calendar_events (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    all_day INTEGER NOT NULL DEFAULT 0,
    location TEXT,
    recurrence TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

CREATE TABLE IF NOT EXISTS event_contacts (
    event_id TEXT NOT NULL,
    contact_id TEXT NOT NULL,
    PRIMARY KEY (event_id, contact_id),
    FOREIGN KEY (event_id) REFERENCES calendar_events(id) ON DELETE CASCADE,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS event_reminders (
    id TEXT PRIMARY KEY,
    event_id TEXT NOT NULL,
    minutes_before INTEGER NOT NULL,
    method TEXT NOT NULL,
    FOREIGN KEY (event_id) REFERENCES calendar_events(id) ON DELETE CASCADE
);

-- Notes with versioning
CREATE TABLE IF NOT EXISTS notes (
    id TEXT PRIMARY KEY,
    contact_id TEXT,
    project_id TEXT,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    last_synced_at TEXT,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Attachments (universal, linked to any entity)
CREATE TABLE IF NOT EXISTS attachments (
    id TEXT PRIMARY KEY,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    storage_path TEXT NOT NULL,
    thumbnail_path TEXT,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    uploaded_by TEXT NOT NULL,
    checksum TEXT NOT NULL,
    encrypted INTEGER NOT NULL DEFAULT 0,
    scan_status TEXT NOT NULL DEFAULT 'Pending',
    scan_details TEXT,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    FOREIGN KEY (uploaded_by) REFERENCES users(id)
);

-- Communication attempts (for outgoing scheduled communications)
CREATE TABLE IF NOT EXISTS communication_attempts (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL,
    method TEXT NOT NULL,
    subject TEXT,
    message TEXT NOT NULL,
    status TEXT NOT NULL,
    scheduled_at TEXT,
    attempted_at TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

-- Communications (historical records of SMS, calls, emails that occurred)
CREATE TABLE IF NOT EXISTS communications (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL,
    communication_type TEXT NOT NULL,
    direction TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    content TEXT,
    duration_seconds INTEGER,
    phone_number TEXT,
    thread_id TEXT,
    status TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

-- Share invites with acceptance tracking
CREATE TABLE IF NOT EXISTS share_invites (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    shared_by TEXT NOT NULL,
    shared_with_email TEXT NOT NULL,
    shared_with_user TEXT,
    permissions TEXT NOT NULL,
    accepted INTEGER NOT NULL DEFAULT 0,
    accepted_at TEXT,
    revoked INTEGER NOT NULL DEFAULT 0,
    revoked_at TEXT,
    created_at TEXT NOT NULL,
    expires_at TEXT,
    FOREIGN KEY (shared_by) REFERENCES users(id),
    FOREIGN KEY (shared_with_user) REFERENCES users(id)
);

-- Resource ACLs
CREATE TABLE IF NOT EXISTS resource_acls (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    owner_id TEXT NOT NULL,
    grants TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (owner_id) REFERENCES users(id)
);

-- Audit logs
CREATE TABLE IF NOT EXISTS audit_logs (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    action TEXT NOT NULL,
    user_id TEXT NOT NULL,
    changes TEXT NOT NULL DEFAULT '{}',
    ip_address TEXT,
    user_agent TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Conflict resolution
CREATE TABLE IF NOT EXISTS conflict_resolutions (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    local_version INTEGER NOT NULL,
    remote_version INTEGER NOT NULL,
    local_updated_at TEXT NOT NULL,
    remote_updated_at TEXT NOT NULL,
    local_changes TEXT NOT NULL,
    remote_changes TEXT NOT NULL,
    resolution_strategy TEXT NOT NULL,
    resolved INTEGER NOT NULL DEFAULT 0,
    resolved_at TEXT,
    resolved_by TEXT,
    created_at TEXT NOT NULL,
    FOREIGN KEY (resolved_by) REFERENCES users(id)
);

-- Search history
CREATE TABLE IF NOT EXISTS search_history (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    query TEXT NOT NULL,
    filters TEXT NOT NULL DEFAULT '{}',
    result_count INTEGER NOT NULL,
    result_ids TEXT NOT NULL DEFAULT '[]',
    clicked_result_id TEXT,
    privacy_mode INTEGER NOT NULL DEFAULT 0,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- AI Insights (expanded from suggestions)
CREATE TABLE IF NOT EXISTS ai_insights (
    id TEXT PRIMARY KEY,
    entity_type TEXT NOT NULL,
    entity_id TEXT NOT NULL,
    insight_type TEXT NOT NULL,
    content TEXT NOT NULL,
    confidence REAL NOT NULL,
    prompt_template TEXT NOT NULL,
    response_cached INTEGER NOT NULL DEFAULT 0,
    feedback_helpful INTEGER,
    feedback_comment TEXT,
    feedback_submitted_at TEXT,
    applied INTEGER NOT NULL DEFAULT 0,
    applied_at TEXT,
    created_at TEXT NOT NULL,
    created_by TEXT NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- AI Interactions (Phase 6: logs all AI API calls with caching/retry metadata)
CREATE TABLE IF NOT EXISTS ai_interactions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    interaction_type TEXT NOT NULL,
    prompt TEXT NOT NULL,
    response TEXT NOT NULL,
    confidence REAL NOT NULL,
    model TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    feedback_helpful INTEGER,
    feedback_applied INTEGER NOT NULL DEFAULT 0,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    feedback_at TEXT,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Legacy AI suggestions (kept for backward compat)
CREATE TABLE IF NOT EXISTS ai_suggestions (
    id TEXT PRIMARY KEY,
    contact_id TEXT,
    suggestion_type TEXT NOT NULL,
    content TEXT NOT NULL,
    confidence REAL NOT NULL,
    applied INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_api_token ON users(api_token);
CREATE INDEX IF NOT EXISTS idx_contacts_email ON contacts(email);
CREATE INDEX IF NOT EXISTS idx_contacts_phone ON contacts(phone);
CREATE INDEX IF NOT EXISTS idx_contacts_created_by ON contacts(created_by);
CREATE INDEX IF NOT EXISTS idx_social_handles_contact ON social_handles(contact_id);
CREATE INDEX IF NOT EXISTS idx_projects_created_by ON projects(created_by);
CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status);
CREATE INDEX IF NOT EXISTS idx_groups_created_by ON groups(created_by);
CREATE INDEX IF NOT EXISTS idx_concepts_created_by ON concepts(created_by);
CREATE INDEX IF NOT EXISTS idx_calendar_events_created_by ON calendar_events(created_by);
CREATE INDEX IF NOT EXISTS idx_calendar_events_start_time ON calendar_events(start_time);
CREATE INDEX IF NOT EXISTS idx_notes_contact ON notes(contact_id);
CREATE INDEX IF NOT EXISTS idx_notes_project ON notes(project_id);
CREATE INDEX IF NOT EXISTS idx_notes_created_by ON notes(created_by);
CREATE INDEX IF NOT EXISTS idx_attachments_entity ON attachments(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_attachments_uploaded_by ON attachments(uploaded_by);
CREATE INDEX IF NOT EXISTS idx_comm_attempts_contact ON communication_attempts(contact_id);
CREATE INDEX IF NOT EXISTS idx_comm_attempts_status ON communication_attempts(status);
CREATE INDEX IF NOT EXISTS idx_communications_contact ON communications(contact_id);
CREATE INDEX IF NOT EXISTS idx_communications_timestamp ON communications(timestamp);
CREATE INDEX IF NOT EXISTS idx_communications_phone ON communications(phone_number);
CREATE INDEX IF NOT EXISTS idx_communications_thread ON communications(thread_id);
CREATE INDEX IF NOT EXISTS idx_communications_type ON communications(communication_type);
CREATE INDEX IF NOT EXISTS idx_share_invites_email ON share_invites(shared_with_email);
CREATE INDEX IF NOT EXISTS idx_share_invites_user ON share_invites(shared_with_user);
CREATE INDEX IF NOT EXISTS idx_share_invites_entity ON share_invites(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_resource_acls_entity ON resource_acls(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_entity ON audit_logs(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_user ON audit_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_conflict_resolutions_entity ON conflict_resolutions(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_search_history_user ON search_history(user_id);
CREATE INDEX IF NOT EXISTS idx_ai_insights_entity ON ai_insights(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_ai_insights_created_by ON ai_insights(created_by);
CREATE INDEX IF NOT EXISTS idx_ai_interactions_user ON ai_interactions(user_id, created_at);
CREATE INDEX IF NOT EXISTS idx_ai_interactions_entity ON ai_interactions(entity_type, entity_id);
"#;
