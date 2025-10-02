-- Import audit log table
CREATE TABLE IF NOT EXISTS import_logs (
    id TEXT PRIMARY KEY NOT NULL,
    job_id TEXT NOT NULL,
    file_name TEXT NOT NULL,
    connector_id TEXT NOT NULL,
    connector_version TEXT NOT NULL,

    -- Counts
    total_rows INTEGER NOT NULL DEFAULT 0,
    imported INTEGER NOT NULL DEFAULT 0,
    skipped INTEGER NOT NULL DEFAULT 0,
    failed INTEGER NOT NULL DEFAULT 0,
    duplicates_found INTEGER NOT NULL DEFAULT 0,

    -- Configuration
    dedupe_strategy TEXT NOT NULL,
    match_criteria TEXT NOT NULL,
    dry_run BOOLEAN NOT NULL DEFAULT 0,

    -- Status
    status TEXT NOT NULL,
    error_message TEXT,

    -- Performance
    elapsed_seconds REAL NOT NULL,
    peak_memory_mb INTEGER,

    -- Timestamps
    started_at TEXT NOT NULL,
    completed_at TEXT,

    -- User context
    user_id TEXT,

    -- Metadata
    metadata TEXT, -- JSON

    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_import_logs_job_id ON import_logs(job_id);
CREATE INDEX IF NOT EXISTS idx_import_logs_status ON import_logs(status);
CREATE INDEX IF NOT EXISTS idx_import_logs_started_at ON import_logs(started_at DESC);

-- Import errors table (detailed error tracking)
CREATE TABLE IF NOT EXISTS import_errors (
    id TEXT PRIMARY KEY NOT NULL,
    log_id TEXT NOT NULL,
    row_number INTEGER NOT NULL,
    error_type TEXT NOT NULL,
    error_message TEXT NOT NULL,
    row_data TEXT, -- JSON of the problematic row

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (log_id) REFERENCES import_logs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_import_errors_log_id ON import_errors(log_id);
CREATE INDEX IF NOT EXISTS idx_import_errors_type ON import_errors(error_type);

-- Import decisions table (duplicate handling decisions)
CREATE TABLE IF NOT EXISTS import_decisions (
    id TEXT PRIMARY KEY NOT NULL,
    log_id TEXT NOT NULL,
    decision_type TEXT NOT NULL, -- 'skip', 'update', 'merge', 'keep_both'
    original_contact_id TEXT,
    imported_contact_id TEXT,
    match_score REAL,
    matched_on TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (log_id) REFERENCES import_logs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_import_decisions_log_id ON import_decisions(log_id);
CREATE INDEX IF NOT EXISTS idx_import_decisions_type ON import_decisions(decision_type);

-- Import rollback journal (for one-click rollback)
CREATE TABLE IF NOT EXISTS import_rollback_journal (
    id TEXT PRIMARY KEY NOT NULL,
    log_id TEXT NOT NULL,
    operation TEXT NOT NULL, -- 'insert', 'update', 'delete'
    table_name TEXT NOT NULL,
    record_id TEXT NOT NULL,
    old_data TEXT, -- JSON snapshot for updates/deletes
    new_data TEXT, -- JSON snapshot for inserts/updates

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (log_id) REFERENCES import_logs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_rollback_log_id ON import_rollback_journal(log_id);
CREATE INDEX IF NOT EXISTS idx_rollback_operation ON import_rollback_journal(operation);

-- Validation warnings table
CREATE TABLE IF NOT EXISTS import_warnings (
    id TEXT PRIMARY KEY NOT NULL,
    log_id TEXT NOT NULL,
    warning_type TEXT NOT NULL,
    warning_message TEXT NOT NULL,
    row_number INTEGER,
    severity TEXT NOT NULL DEFAULT 'warning', -- 'info', 'warning', 'error'

    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (log_id) REFERENCES import_logs(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_import_warnings_log_id ON import_warnings(log_id);
CREATE INDEX IF NOT EXISTS idx_import_warnings_severity ON import_warnings(severity);
