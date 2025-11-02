# Contact Deduplication Script

## Problem
When SMS messages or contacts are imported multiple times, duplicate contacts are created for the same phone number. This violates the principle that **there should only be one contact per phone number**.

## Current Situation
- **1,525 phone numbers** have duplicate contacts
- Most duplicates occurred during repeated SMS imports
- Each duplicate has SMS messages and other data that needs to be merged

## Solution
This script automatically merges all duplicate contacts by phone number:

### What it does:
1. **Identifies duplicates** - Finds all phone numbers with multiple contacts
2. **Keeps oldest contact** - For each phone number, keeps the contact with the earliest `created_at` timestamp
3. **Merges all data**:
   - SMS messages (`sms_history`)
   - Communication attempts
   - Notes
   - AI suggestions
   - Calendar events
   - Tags (deduplicated)
   - Projects (deduplicated)
   - Groups (deduplicated)
   - Social handles (deduplicated by platform)
4. **Deletes duplicates** - Removes duplicate contacts after merging

### Safety Features:
- ✅ **Automatic backup** - Creates timestamped backup before any changes
- ✅ **Confirmation required** - Asks "yes/no" before proceeding
- ✅ **Transaction-based** - All changes in one transaction (all-or-nothing)
- ✅ **Restoration instructions** - Shows how to restore backup if needed

## Usage

### Run the deduplication:
```bash
cd scripts
./run_deduplication.sh
```

### What you'll see:
```
=========================================
 SagensContact Deduplication Script
=========================================

Creating backup...
✓ Backup created at: ../data/backups/contacts_pre_dedup_20251102_123456.db

Current database statistics:
Total contacts: 8230
Phone numbers with duplicates: 1525

This will merge duplicate contacts by phone number.
The oldest contact (by created_at) will be kept for each phone number.

Do you want to proceed? (yes/no):
```

### After running:
```
Deduplication Complete!

Backup saved at: ../data/backups/contacts_pre_dedup_20251102_123456.db

To restore from backup if needed:
  cp ../data/backups/contacts_pre_dedup_20251102_123456.db ../data/contacts.db
```

## Expected Results

### Before:
- 8,230 total contacts
- 1,525 phone numbers with duplicates
- Many phone numbers have 8 duplicate contacts

### After:
- ~6,705 total contacts (1,525 unique phones kept, ~1,525 duplicates removed)
- 0 phone numbers with duplicates
- All SMS messages and data preserved on the correct (oldest) contact

## Manual Inspection

### Check specific duplicates before running:
```bash
# See Sara Shields duplicates
sqlite3 ../data/contacts.db "
SELECT id, first_name, last_name, phone, created_at
FROM contacts
WHERE phone = '+15099694479'
ORDER BY created_at;
"
```

### Check duplicates by phone number:
```bash
sqlite3 ../data/contacts.db "
SELECT phone, COUNT(*) as count
FROM contacts
WHERE phone IS NOT NULL AND phone <> ''
GROUP BY phone
HAVING count > 1
ORDER BY count DESC
LIMIT 10;
"
```

### After deduplication, verify no duplicates:
```bash
sqlite3 ../data/contacts.db "
SELECT COUNT(*) FROM (
  SELECT phone FROM contacts
  WHERE phone IS NOT NULL AND phone <> ''
  GROUP BY phone
  HAVING COUNT(*) > 1
);
"
# Should return: 0
```

## Restoration

If something goes wrong, restore from backup:

```bash
# Find your backup
ls -lh ../data/backups/

# Restore (replace with your backup timestamp)
cp ../data/backups/contacts_pre_dedup_YYYYMMDD_HHMMSS.db ../data/contacts.db
```

## Future Prevention

After running this script once, we need to:

1. **Add unique constraint on phone** - Prevent duplicates at database level
2. **Update import logic** - Match existing contacts by phone instead of creating new ones
3. **Update contact creation** - Check for existing phone before creating

These changes are tracked in the project TODO list.

## Technical Details

### Merge Strategy:
- **Phone number**: Primary key for deduplication
- **Keep contact**: Oldest by `created_at` timestamp
- **Merge approach**:
  - One-to-many relations (SMS, notes) → UPDATE to point to keep_id
  - Many-to-many relations (tags, projects) → INSERT OR IGNORE into keep_id
  - Social handles → INSERT only if platform doesn't exist for keep_id

### Database Tables Affected:
- `contacts` - Duplicates deleted
- `sms_history` - Re-linked to keep_id
- `communication_attempts` - Re-linked to keep_id
- `notes` - Re-linked to keep_id
- `ai_suggestions` - Re-linked to keep_id
- `event_contacts` - Re-linked to keep_id
- `contact_tags` - Merged (deduplicated)
- `project_contacts` - Merged (deduplicated)
- `contact_groups` - Merged (deduplicated)
- `social_handles` - Merged (deduplicated by platform)

---

**Last Updated:** 2025-11-02
**Version:** Alpha v0.1.0-alpha.3+
