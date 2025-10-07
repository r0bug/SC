-- Add communication history tables for imported Android backups

-- Call history table
CREATE TABLE IF NOT EXISTS call_history (
    id TEXT PRIMARY KEY,
    contact_id TEXT,
    phone_number TEXT NOT NULL,
    contact_name TEXT,
    call_date INTEGER NOT NULL,  -- Unix timestamp in milliseconds
    duration INTEGER NOT NULL,    -- Duration in seconds
    call_type INTEGER NOT NULL,   -- 1=incoming, 2=outgoing, 3=missed, 4=voicemail, 5=rejected, 6=blocked
    readable_date TEXT NOT NULL,
    subscription_id TEXT,
    imported_at TEXT NOT NULL,
    imported_by TEXT NOT NULL,
    source_file TEXT,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL,
    FOREIGN KEY (imported_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_call_history_contact_id ON call_history(contact_id);
CREATE INDEX IF NOT EXISTS idx_call_history_phone_number ON call_history(phone_number);
CREATE INDEX IF NOT EXISTS idx_call_history_call_date ON call_history(call_date);
CREATE INDEX IF NOT EXISTS idx_call_history_call_type ON call_history(call_type);
CREATE INDEX IF NOT EXISTS idx_call_history_imported_by ON call_history(imported_by);

-- SMS/MMS message history table
CREATE TABLE IF NOT EXISTS sms_history (
    id TEXT PRIMARY KEY,
    contact_id TEXT,
    phone_number TEXT NOT NULL,
    contact_name TEXT,
    message_date INTEGER NOT NULL,     -- Unix timestamp in milliseconds
    message_type INTEGER NOT NULL,     -- 1=received, 2=sent, 3=draft, 4=outbox, 5=failed, 6=queued
    subject TEXT,
    body TEXT NOT NULL,
    readable_date TEXT NOT NULL,
    thread_id INTEGER,
    read_status INTEGER DEFAULT 1,     -- 0=unread, 1=read
    subscription_id TEXT,
    imported_at TEXT NOT NULL,
    imported_by TEXT NOT NULL,
    source_file TEXT,
    FOREIGN KEY (contact_id) REFERENCES contacts(id) ON DELETE SET NULL,
    FOREIGN KEY (imported_by) REFERENCES users(id)
);

CREATE INDEX IF NOT EXISTS idx_sms_history_contact_id ON sms_history(contact_id);
CREATE INDEX IF NOT EXISTS idx_sms_history_phone_number ON sms_history(phone_number);
CREATE INDEX IF NOT EXISTS idx_sms_history_message_date ON sms_history(message_date);
CREATE INDEX IF NOT EXISTS idx_sms_history_message_type ON sms_history(message_type);
CREATE INDEX IF NOT EXISTS idx_sms_history_thread_id ON sms_history(thread_id);
CREATE INDEX IF NOT EXISTS idx_sms_history_body ON sms_history(body);
CREATE INDEX IF NOT EXISTS idx_sms_history_imported_by ON sms_history(imported_by);

-- MMS parts table (for attachments, multiple parts per MMS)
CREATE TABLE IF NOT EXISTS mms_parts (
    id TEXT PRIMARY KEY,
    sms_id TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    content_type TEXT NOT NULL,
    name TEXT,
    text TEXT,
    data_path TEXT,  -- Path to stored file if binary data
    FOREIGN KEY (sms_id) REFERENCES sms_history(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_mms_parts_sms_id ON mms_parts(sms_id);
