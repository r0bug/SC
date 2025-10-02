-- Enhance attachments table with new security and metadata fields
-- Drop the old table and recreate with all fields
DROP TABLE IF EXISTS attachments;

CREATE TABLE attachments (
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
    created_at TEXT NOT NULL
);

CREATE INDEX idx_attachments_entity ON attachments(entity_type, entity_id);
CREATE INDEX idx_attachments_uploaded_by ON attachments(uploaded_by);
CREATE INDEX idx_attachments_scan_status ON attachments(scan_status);

-- Create AI interactions table
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
    feedback_at TEXT
);

CREATE INDEX idx_ai_interactions_user ON ai_interactions(user_id);
CREATE INDEX idx_ai_interactions_entity ON ai_interactions(entity_type, entity_id);
CREATE INDEX idx_ai_interactions_type ON ai_interactions(interaction_type);
CREATE INDEX idx_ai_interactions_created ON ai_interactions(created_at);

-- Enhance search_history table
CREATE TABLE IF NOT EXISTS search_history_new (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    query TEXT NOT NULL,
    filters TEXT NOT NULL DEFAULT '{}',
    result_count INTEGER NOT NULL,
    result_ids TEXT NOT NULL DEFAULT '[]',
    clicked_result_id TEXT,
    privacy_mode INTEGER NOT NULL DEFAULT 0,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL
);

-- Copy existing search history if it exists
INSERT INTO search_history_new (id, user_id, query, result_count, clicked_result_id, created_at)
SELECT id, user_id, query, result_count, clicked_result_id, created_at
FROM search_history WHERE EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='search_history');

-- Drop old table and rename new one
DROP TABLE IF EXISTS search_history;
ALTER TABLE search_history_new RENAME TO search_history;

CREATE INDEX idx_search_history_user ON search_history(user_id);
CREATE INDEX idx_search_history_created ON search_history(created_at);
CREATE INDEX idx_search_history_privacy ON search_history(privacy_mode);