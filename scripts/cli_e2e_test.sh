#!/bin/bash

set -e

echo "🧪 Running CLI E2E tests..."

TEST_DB="data/test_e2e.db"
CLI_BIN="./target/release/sagenscontact"

rm -f "$TEST_DB"
export DATABASE_URL="sqlite:$TEST_DB"

echo "✅ Building CLI..."
cargo build --release --bin sagenscontact

echo "✅ Testing import CSV..."
$CLI_BIN import --csv sample_data/contacts.csv
OUTPUT=$($CLI_BIN list)
echo "$OUTPUT" | grep -q "John Doe" || { echo "❌ Failed to find John Doe"; exit 1; }

echo "✅ Testing import vCard..."
$CLI_BIN import --vcard sample_data/contacts.vcf

echo "✅ Testing import SMS..."
$CLI_BIN import --sms sample_data/sms_export.json

echo "✅ Testing search..."
OUTPUT=$($CLI_BIN search "john")
echo "$OUTPUT" | grep -q "john.doe@example.com" || { echo "❌ Failed to search"; exit 1; }

echo "✅ Testing add contact..."
$CLI_BIN add "Test" "User" --email test@example.com --phone "+1-555-9999"

echo "✅ Testing search for added contact..."
OUTPUT=$($CLI_BIN search "Test User")
echo "$OUTPUT" | grep -q "test@example.com" || { echo "❌ Failed to find added contact"; exit 1; }

echo "✅ Cleaning up test database..."
rm -f "$TEST_DB"

echo "🎉 All CLI E2E tests passed!"