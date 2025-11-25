-- Migration: Add E2E Encryption Support
-- Description: Creates tables for zero-knowledge end-to-end encryption
-- Date: 2025-11-16

-- Table: encryption_keys
-- Purpose: Stores user encryption keys (private key is encrypted with password-derived key)
CREATE TABLE IF NOT EXISTS encryption_keys (
    user_id TEXT PRIMARY KEY NOT NULL,
    encrypted_private_key TEXT NOT NULL, -- Base64-encoded encrypted private key
    public_key TEXT NOT NULL,             -- Base64-encoded public key (for future key sharing)
    salt TEXT NOT NULL,                   -- Base64-encoded salt for PBKDF2/Argon2
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Table: encrypted_entities
-- Purpose: Stores encrypted entity data with zero-knowledge guarantees
-- The server never has access to decryption keys
CREATE TABLE IF NOT EXISTS encrypted_entities (
    id TEXT PRIMARY KEY NOT NULL,         -- UUID of the entity
    user_id TEXT NOT NULL,                -- Owner of the encrypted data
    entity_type TEXT NOT NULL,            -- Type: 'contact', 'note', 'project', 'calendar_event'
    encrypted_data TEXT NOT NULL,         -- Base64-encoded EncryptedBlob (nonce + ciphertext)
    checksum TEXT NOT NULL,               -- SHA-256 hash for integrity verification
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    deleted_at TEXT,                      -- Soft delete timestamp
    FOREIGN KEY (user_id) REFERENCES encryption_keys(user_id) ON DELETE CASCADE
);

-- Table: key_rotation_log
-- Purpose: Audit log for key rotation events
CREATE TABLE IF NOT EXISTS key_rotation_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    old_key_fingerprint TEXT NOT NULL,    -- SHA-256 of old public key
    new_key_fingerprint TEXT NOT NULL,    -- SHA-256 of new public key
    rotation_reason TEXT,                 -- 'scheduled', 'compromised', 'user_request'
    rotated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (user_id) REFERENCES encryption_keys(user_id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_encrypted_entities_user_id
    ON encrypted_entities(user_id);

CREATE INDEX IF NOT EXISTS idx_encrypted_entities_entity_type
    ON encrypted_entities(entity_type);

CREATE INDEX IF NOT EXISTS idx_encrypted_entities_user_type
    ON encrypted_entities(user_id, entity_type);

CREATE INDEX IF NOT EXISTS idx_encrypted_entities_deleted
    ON encrypted_entities(deleted_at)
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_key_rotation_log_user_id
    ON key_rotation_log(user_id);

CREATE INDEX IF NOT EXISTS idx_key_rotation_log_rotated_at
    ON key_rotation_log(rotated_at);

-- Trigger: Update timestamp on encryption_keys
CREATE TRIGGER IF NOT EXISTS update_encryption_keys_timestamp
AFTER UPDATE ON encryption_keys
FOR EACH ROW
BEGIN
    UPDATE encryption_keys
    SET updated_at = datetime('now')
    WHERE user_id = NEW.user_id;
END;

-- Trigger: Update timestamp on encrypted_entities
CREATE TRIGGER IF NOT EXISTS update_encrypted_entities_timestamp
AFTER UPDATE ON encrypted_entities
FOR EACH ROW
BEGIN
    UPDATE encrypted_entities
    SET updated_at = datetime('now')
    WHERE id = NEW.id;
END;
