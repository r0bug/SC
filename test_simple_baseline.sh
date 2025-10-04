#!/bin/bash

# Simple Performance Baseline Test
# Tests 10k, 50k, 100k datasets and captures basic metrics

BASE_URL="http://localhost:3002"
TEST_DATA_DIR="sample_data/load_tests"
RESULTS_DIR="test_results/simple_baselines"

mkdir -p "$RESULTS_DIR"

echo "==========================================="
echo "Simple Performance Baseline Test"
echo "==========================================="
echo ""

# Function to run import test
run_import() {
  local NAME=$1
  local FILE=$2

  echo "Testing: $NAME"
  echo "-------------------------------------------"

  # Config
  CONFIG='{
    "connector_id": "generic_csv",
    "dedupe_strategy": "skip",
    "match_criteria": "email",
    "dry_run": false
  }'

  # Execute import
  START=$(date +%s)
  RESP=$(curl -s -X POST "$BASE_URL/api/import/execute" \
    -F "file=@$FILE" \
    -F "config=$CONFIG")

  JOB_ID=$(echo "$RESP" | grep -o '"job_id":"[^"]*"' | grep -o ':[^:]*$' | tr -d ':"')

  if [ -z "$JOB_ID" ]; then
    echo "✗ Failed: $RESP"
    return 1
  fi

  # Poll for completion
  STATUS=""
  while [ "$STATUS" != "completed" ] && [ "$STATUS" != "failed" ]; do
    sleep 0.5
    STATUS_JSON=$(curl -s "$BASE_URL/api/import/jobs/$JOB_ID")
    STATUS=$(echo "$STATUS_JSON" | grep -o '"status":"[^"]*"' | grep -o ':[^:]*$' | tr -d ':"')

    if [ -z "$STATUS" ]; then
      echo "✗ Failed to get status"
      break
    fi
  done

  END=$(date +%s)
  WALL_TIME=$((END - START))

  # Extract metrics
  IMPORTED=$(echo "$STATUS_JSON" | grep -o '"imported":[0-9]*' | grep -o '[0-9]*')
  SKIPPED=$(echo "$STATUS_JSON" | grep -o '"skipped":[0-9]*' | grep -o '[0-9]*')
  ELAPSED=$(echo "$STATUS_JSON" | grep -o '"elapsed_seconds":[0-9.]*' | grep -o '[0-9.]*')

  if [ ! -z "$IMPORTED" ] && [ "$IMPORTED" -gt 0 ]; then
    RATE=$(echo "scale=0; $IMPORTED / $ELAPSED" | bc)
  else
    RATE=0
  fi

  echo "✓ Status: $STATUS"
  echo "  Wall time: ${WALL_TIME}s"
  echo "  Proc time: ${ELAPSED}s"
  echo "  Imported:  $IMPORTED"
  echo "  Skipped:   $SKIPPED"
  echo "  Rate:      ${RATE}/sec"
  echo ""

  # Save to CSV
  echo "$NAME,$IMPORTED,$SKIPPED,$ELAPSED,$WALL_TIME,$RATE" >> "$RESULTS_DIR/summary.csv"
}

# Initialize
echo "dataset,imported,skipped,proc_sec,wall_sec,rate_per_sec" > "$RESULTS_DIR/summary.csv"

# Run tests
run_import "10k"          "$TEST_DATA_DIR/contacts_10k.csv"
run_import "50k_dupes"    "$TEST_DATA_DIR/contacts_50k_with_duplicates.csv"
run_import "100k"         "$TEST_DATA_DIR/contacts_100k.csv"

# Summary
echo "==========================================="
echo "SUMMARY"
echo "==========================================="
cat "$RESULTS_DIR/summary.csv" | column -t -s ','
echo ""

# DB status
DB_SIZE=$(du -h data/contacts.db | cut -f1)
COUNT=$(sqlite3 data/contacts.db "SELECT COUNT(*) FROM contacts;" 2>/dev/null)
echo "Database: $DB_SIZE ($COUNT contacts)"
echo ""
