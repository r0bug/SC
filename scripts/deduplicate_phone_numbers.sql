-- Deduplication script for contacts with duplicate phone numbers
-- This script keeps the oldest contact for each phone number and merges all data into it

BEGIN TRANSACTION;

-- Step 1: Create a temporary table with phone numbers that have duplicates
-- and identify which contact to keep (oldest by created_at)
CREATE TEMP TABLE contacts_to_keep AS
SELECT phone, MIN(created_at) as oldest_created_at
FROM contacts
WHERE phone IS NOT NULL AND phone <> ''
GROUP BY phone
HAVING COUNT(*) > 1;

-- Step 2: Create a table mapping phone numbers to the ID we want to keep
CREATE TEMP TABLE keep_ids AS
SELECT c.id as keep_id, c.phone
FROM contacts c
INNER JOIN contacts_to_keep ctk ON c.phone = ctk.phone AND c.created_at = ctk.oldest_created_at;

-- Step 3: Create a table of all duplicate IDs to merge/delete
CREATE TEMP TABLE merge_ids AS
SELECT c.id as merge_id, c.phone, ki.keep_id
FROM contacts c
INNER JOIN keep_ids ki ON c.phone = ki.phone
WHERE c.id != ki.keep_id;

-- Show statistics before merge
SELECT
    (SELECT COUNT(DISTINCT phone) FROM contacts_to_keep) as duplicate_phone_numbers,
    (SELECT COUNT(*) FROM merge_ids) as contacts_to_merge,
    (SELECT COUNT(*) FROM keep_ids) as contacts_to_keep;

-- Step 4: Merge SMS history
UPDATE sms_history
SET contact_id = (SELECT keep_id FROM merge_ids WHERE merge_id = sms_history.contact_id)
WHERE contact_id IN (SELECT merge_id FROM merge_ids);

-- Step 5: Merge communication attempts
UPDATE communication_attempts
SET contact_id = (SELECT keep_id FROM merge_ids WHERE merge_id = communication_attempts.contact_id)
WHERE contact_id IN (SELECT merge_id FROM merge_ids);

-- Step 6: Merge notes (re-link to keep_id)
UPDATE notes
SET contact_id = (SELECT keep_id FROM merge_ids WHERE merge_id = notes.contact_id)
WHERE contact_id IN (SELECT merge_id FROM merge_ids);

-- Step 7: Merge AI suggestions
UPDATE ai_suggestions
SET contact_id = (SELECT keep_id FROM merge_ids WHERE merge_id = ai_suggestions.contact_id)
WHERE contact_id IN (SELECT merge_id FROM merge_ids);

-- Step 8: Merge event contacts
UPDATE OR IGNORE event_contacts
SET contact_id = (SELECT keep_id FROM merge_ids WHERE merge_id = event_contacts.contact_id)
WHERE contact_id IN (SELECT merge_id FROM merge_ids);

-- Step 9: Merge tags (avoid duplicates with OR IGNORE)
INSERT OR IGNORE INTO contact_tags (contact_id, tag_id)
SELECT ki.keep_id, ct.tag_id
FROM contact_tags ct
INNER JOIN merge_ids mi ON ct.contact_id = mi.merge_id
INNER JOIN keep_ids ki ON mi.keep_id = ki.keep_id;

-- Step 10: Merge projects (avoid duplicates with OR IGNORE)
INSERT OR IGNORE INTO project_contacts (project_id, contact_id)
SELECT pc.project_id, ki.keep_id
FROM project_contacts pc
INNER JOIN merge_ids mi ON pc.contact_id = mi.merge_id
INNER JOIN keep_ids ki ON mi.keep_id = ki.keep_id;

-- Step 11: Merge groups (avoid duplicates with OR IGNORE)
INSERT OR IGNORE INTO contact_groups (group_id, contact_id)
SELECT cg.group_id, ki.keep_id
FROM contact_groups cg
INNER JOIN merge_ids mi ON cg.contact_id = mi.merge_id
INNER JOIN keep_ids ki ON mi.keep_id = ki.keep_id;

-- Step 12: Merge social handles (avoid duplicates by platform)
INSERT OR IGNORE INTO social_handles (contact_id, platform, handle, url)
SELECT ki.keep_id, sh.platform, sh.handle, sh.url
FROM social_handles sh
INNER JOIN merge_ids mi ON sh.contact_id = mi.merge_id
INNER JOIN keep_ids ki ON mi.keep_id = ki.keep_id
WHERE NOT EXISTS (
    SELECT 1 FROM social_handles
    WHERE contact_id = ki.keep_id AND platform = sh.platform
);

-- Step 13: Delete duplicate contacts (CASCADE will clean up related tables)
DELETE FROM contacts WHERE id IN (SELECT merge_id FROM merge_ids);

-- Show final statistics
SELECT
    (SELECT COUNT(*) FROM contacts) as total_contacts_after,
    (SELECT COUNT(DISTINCT phone) FROM contacts WHERE phone IS NOT NULL AND phone <> '') as unique_phone_numbers;

COMMIT;

-- Cleanup temp tables
DROP TABLE IF EXISTS contacts_to_keep;
DROP TABLE IF EXISTS keep_ids;
DROP TABLE IF EXISTS merge_ids;
