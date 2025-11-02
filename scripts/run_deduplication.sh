#!/bin/bash
# Safe deduplication script with automatic backup

set -e  # Exit on any error

DB_PATH="../data/contacts.db"
BACKUP_DIR="../data/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_PATH="${BACKUP_DIR}/contacts_pre_dedup_${TIMESTAMP}.db"

echo "========================================="
echo " SagensContact Deduplication Script"
echo "========================================="
echo

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
    echo "ERROR: Database not found at $DB_PATH"
    exit 1
fi

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Create backup
echo "Creating backup..."
cp "$DB_PATH" "$BACKUP_PATH"
echo "✓ Backup created at: $BACKUP_PATH"
echo

# Show current statistics
echo "Current database statistics:"
sqlite3 "$DB_PATH" <<'EOF'
SELECT
    'Total contacts: ' || COUNT(*) as stat FROM contacts
UNION ALL
SELECT
    'Phone numbers with duplicates: ' || COUNT(*)
FROM (
    SELECT phone FROM contacts
    WHERE phone IS NOT NULL AND phone <> ''
    GROUP BY phone HAVING COUNT(*) > 1
);
EOF
echo

# Ask for confirmation
echo "This will merge duplicate contacts by phone number."
echo "The oldest contact (by created_at) will be kept for each phone number."
echo
read -p "Do you want to proceed? (yes/no): " response

if [ "$response" != "yes" ]; then
    echo "Deduplication cancelled."
    exit 0
fi

echo
echo "Running deduplication..."
sqlite3 "$DB_PATH" < deduplicate_phone_numbers.sql

echo
echo "========================================="
echo " Deduplication Complete!"
echo "========================================="
echo
echo "Backup saved at: $BACKUP_PATH"
echo
echo "To restore from backup if needed:"
echo "  cp $BACKUP_PATH $DB_PATH"
echo
