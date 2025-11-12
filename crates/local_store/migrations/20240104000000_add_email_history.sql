-- Add email history table with AI importance monitoring
-- This table stores incoming and outgoing emails with AI-powered importance analysis

CREATE TABLE IF NOT EXISTS email_history (
    id TEXT PRIMARY KEY,
    contact_id TEXT,
    from_address TEXT NOT NULL,
    from_name TEXT,
    to_address TEXT NOT NULL,
    cc TEXT,                           -- Comma-separated email addresses
    bcc TEXT,                          -- Comma-separated email addresses
    subject TEXT NOT NULL,
    body_text TEXT,                    -- Plain text body
    body_html TEXT,                    -- HTML body
    message_date INTEGER NOT NULL,     -- Unix timestamp in milliseconds
    message_type INTEGER NOT NULL,     -- 1=received, 2=sent, 3=draft
    message_id TEXT,                   -- Email Message-ID header
    in_reply_to TEXT,                  -- In-Reply-To header for threading
    email_references TEXT,             -- References header for threading
    read_status INTEGER DEFAULT 0,     -- 0=unread, 1=read
    has_attachments INTEGER DEFAULT 0, -- 0=no, 1=yes
    folder TEXT,                       -- IMAP folder (INBOX, Sent, etc.)

    -- AI Importance Analysis
    importance_score INTEGER,          -- 0-10 scale, AI-generated
    is_important INTEGER DEFAULT 0,    -- 0=normal, 1=important (score >= 7)
    is_urgent INTEGER DEFAULT 0,       -- 0=not urgent, 1=urgent
    ai_summary TEXT,                   -- AI-generated summary
    ai_analysis TEXT,                  -- Full AI analysis JSON
    ai_analyzed_at TEXT,               -- When AI analysis was performed

    -- Metadata
    imported_at TEXT NOT NULL,
    imported_by TEXT NOT NULL,         -- 'imap_sync', 'smtp_send', 'manual', etc.
    source_server TEXT,                -- IMAP server hostname

    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_email_history_contact_id ON email_history(contact_id);
CREATE INDEX IF NOT EXISTS idx_email_history_from_address ON email_history(from_address);
CREATE INDEX IF NOT EXISTS idx_email_history_to_address ON email_history(to_address);
CREATE INDEX IF NOT EXISTS idx_email_history_message_date ON email_history(message_date);
CREATE INDEX IF NOT EXISTS idx_email_history_message_type ON email_history(message_type);
CREATE INDEX IF NOT EXISTS idx_email_history_message_id ON email_history(message_id);
CREATE INDEX IF NOT EXISTS idx_email_history_is_important ON email_history(is_important);
CREATE INDEX IF NOT EXISTS idx_email_history_is_urgent ON email_history(is_urgent);
CREATE INDEX IF NOT EXISTS idx_email_history_importance_score ON email_history(importance_score);
CREATE INDEX IF NOT EXISTS idx_email_history_subject ON email_history(subject);

-- Full-text search on email bodies
CREATE VIRTUAL TABLE IF NOT EXISTS email_history_fts USING fts5(
    subject,
    body_text,
    from_address,
    from_name,
    content='email_history',
    content_rowid='rowid'
);

-- Triggers to keep FTS index in sync
CREATE TRIGGER IF NOT EXISTS email_history_fts_insert AFTER INSERT ON email_history BEGIN
    INSERT INTO email_history_fts(rowid, subject, body_text, from_address, from_name)
    VALUES (NEW.rowid, NEW.subject, NEW.body_text, NEW.from_address, NEW.from_name);
END;

CREATE TRIGGER IF NOT EXISTS email_history_fts_delete AFTER DELETE ON email_history BEGIN
    DELETE FROM email_history_fts WHERE rowid = OLD.rowid;
END;

CREATE TRIGGER IF NOT EXISTS email_history_fts_update AFTER UPDATE ON email_history BEGIN
    DELETE FROM email_history_fts WHERE rowid = OLD.rowid;
    INSERT INTO email_history_fts(rowid, subject, body_text, from_address, from_name)
    VALUES (NEW.rowid, NEW.subject, NEW.body_text, NEW.from_address, NEW.from_name);
END;
