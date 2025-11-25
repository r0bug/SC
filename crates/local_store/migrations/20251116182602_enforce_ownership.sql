-- Migration: Enforce Multi-User Ownership
-- Date: 2025-11-16
-- Purpose: Add missing created_by columns, create system user, and enforce FK constraints

-- Step 1: Create system user with fixed UUID
-- This user will own system-generated entities and legacy data
INSERT OR IGNORE INTO users (
    id,
    email,
    name,
    password_hash,
    api_token,
    email_verified,
    active,
    preferences,
    created_at,
    updated_at,
    last_login_at
) VALUES (
    '00000000-0000-0000-0000-000000000001',
    'system@sagenscontact.local',
    'System',
    '',  -- No password (cannot login)
    NULL,
    1,
    1,
    '{}',
    datetime('now'),
    datetime('now'),
    NULL
);

-- Step 2: Add created_by to communication_attempts table
-- First add the column as nullable
ALTER TABLE communication_attempts ADD COLUMN created_by TEXT;

-- Update existing records to use system user
UPDATE communication_attempts SET created_by = '00000000-0000-0000-0000-000000000001' WHERE created_by IS NULL;

-- Note: SQLite doesn't support adding NOT NULL constraint to existing column directly
-- We'll enforce it at the application level and in new schema

-- Step 3: Add created_by to communications table
ALTER TABLE communications ADD COLUMN created_by TEXT;

-- Update existing records to use system user
UPDATE communications SET created_by = '00000000-0000-0000-0000-000000000001' WHERE created_by IS NULL;

-- Step 4: Create indexes for new created_by columns
CREATE INDEX IF NOT EXISTS idx_communication_attempts_created_by ON communication_attempts(created_by);
CREATE INDEX IF NOT EXISTS idx_communications_created_by ON communications(created_by);

-- Step 5: Update tags table to have created_by (tags should have owners)
ALTER TABLE tags ADD COLUMN created_by TEXT;

-- Update existing tags to be owned by system user
UPDATE tags SET created_by = '00000000-0000-0000-0000-000000000001' WHERE created_by IS NULL;

CREATE INDEX IF NOT EXISTS idx_tags_created_by ON tags(created_by);

-- Note: We cannot add FOREIGN KEY constraints to existing tables in SQLite without recreating them
-- The FK constraints will be enforced in the application layer and for new records

-- Step 6: Create a view to identify orphaned records (for monitoring)
CREATE VIEW IF NOT EXISTS orphaned_records AS
SELECT 'contacts' as table_name, id, created_by FROM contacts WHERE created_by NOT IN (SELECT id FROM users)
UNION ALL
SELECT 'projects' as table_name, id, created_by FROM projects WHERE created_by NOT IN (SELECT id FROM users)
UNION ALL
SELECT 'groups' as table_name, id, created_by FROM groups WHERE created_by NOT IN (SELECT id FROM users)
UNION ALL
SELECT 'concepts' as table_name, id, created_by FROM concepts WHERE created_by NOT IN (SELECT id FROM users)
UNION ALL
SELECT 'calendar_events' as table_name, id, created_by FROM calendar_events WHERE created_by NOT IN (SELECT id FROM users)
UNION ALL
SELECT 'notes' as table_name, id, created_by FROM notes WHERE created_by NOT IN (SELECT id FROM users)
UNION ALL
SELECT 'ai_insights' as table_name, id, created_by FROM ai_insights WHERE created_by NOT IN (SELECT id FROM users);
