#!/bin/bash

# Test Import API End-to-End
# Week 5 - Backend Integration Testing

BASE_URL="http://localhost:3002"
TEST_DATA_DIR="sample_data/load_tests"
RESULTS_DIR="test_results/import_api"

mkdir -p "$RESULTS_DIR"

echo "============================================"
echo "Import API End-to-End Test"
echo "============================================"
echo ""

# Test 1: List connectors
echo "TEST 1: GET /api/import/connectors"
echo "-------------------------------------------"
CONNECTORS=$(curl -s "$BASE_URL/api/import/connectors")
CONNECTOR_COUNT=$(echo "$CONNECTORS" | grep -o '"id"' | wc -l)
echo "✓ Found $CONNECTOR_COUNT connectors"
echo "$CONNECTORS" > "$RESULTS_DIR/connectors.json"
echo ""

# Test 2: Preview 10k dataset
echo "TEST 2: POST /api/import/preview (10k dataset)"
echo "-------------------------------------------"
PREVIEW_START=$(date +%s%N)
PREVIEW_RESPONSE=$(curl -s -X POST "$BASE_URL/api/import/preview?limit=5" \
  -F "file=@$TEST_DATA_DIR/contacts_10k.csv")
PREVIEW_END=$(date +%s%N)
PREVIEW_MS=$(( ($PREVIEW_END - $PREVIEW_START) / 1000000 ))

echo "$PREVIEW_RESPONSE" > "$RESULTS_DIR/preview_10k.json"
TOTAL_ROWS=$(echo "$PREVIEW_RESPONSE" | grep -o '"total_rows":[0-9]*' | grep -o '[0-9]*')
CONNECTOR_ID=$(echo "$PREVIEW_RESPONSE" | grep -o '"id":"[^"]*"' | head -1 | grep -o ':"[^"]*"' | tr -d ':"')

echo "✓ Preview completed in ${PREVIEW_MS}ms"
echo "✓ Total rows: $TOTAL_ROWS"
echo "✓ Detected connector: $CONNECTOR_ID"
echo ""

# Test 3: Execute import (10k dataset)
echo "TEST 3: POST /api/import/execute (10k dataset)"
echo "-------------------------------------------"
IMPORT_CONFIG='{
  "connector_id": "generic_csv",
  "dedupe_strategy": "skip",
  "match_criteria": "email",
  "dry_run": false
}'

EXEC_START=$(date +%s)
EXEC_RESPONSE=$(curl -s -X POST "$BASE_URL/api/import/execute" \
  -F "file=@$TEST_DATA_DIR/contacts_10k.csv" \
  -F "config=$IMPORT_CONFIG")

echo "$EXEC_RESPONSE" > "$RESULTS_DIR/execute_10k.json"
JOB_ID=$(echo "$EXEC_RESPONSE" | grep -o '"job_id":"[^"]*"' | grep -o ':"[^"]*"' | tr -d ':"')

echo "✓ Import job created: $JOB_ID"
echo ""

# Test 4: Poll job status
echo "TEST 4: GET /api/import/jobs/:job_id (polling)"
echo "-------------------------------------------"
MAX_POLLS=60
POLL_COUNT=0
STATUS="pending"

while [ "$STATUS" != "completed" ] && [ "$STATUS" != "failed" ] && [ $POLL_COUNT -lt $MAX_POLLS ]; do
  sleep 2
  JOB_STATUS=$(curl -s "$BASE_URL/api/import/jobs/$JOB_ID")
  STATUS=$(echo "$JOB_STATUS" | grep -o '"status":"[^"]*"' | grep -o ':"[^"]*"' | tr -d ':"')
  PROGRESS=$(echo "$JOB_STATUS" | grep -o '"current":[0-9]*' | grep -o '[0-9]*')
  PHASE=$(echo "$JOB_STATUS" | grep -o '"phase":"[^"]*"' | grep -o ':"[^"]*"' | tr -d ':"')

  POLL_COUNT=$((POLL_COUNT + 1))
  echo "  [Poll $POLL_COUNT] Status: $STATUS | Phase: $PHASE | Progress: $PROGRESS"
done

EXEC_END=$(date +%s)
EXEC_DURATION=$((EXEC_END - EXEC_START))

echo "$JOB_STATUS" > "$RESULTS_DIR/job_status_10k.json"

if [ "$STATUS" = "completed" ]; then
  IMPORTED=$(echo "$JOB_STATUS" | grep -o '"imported":[0-9]*' | grep -o '[0-9]*')
  SKIPPED=$(echo "$JOB_STATUS" | grep -o '"skipped":[0-9]*' | grep -o '[0-9]*')
  FAILED=$(echo "$JOB_STATUS" | grep -o '"failed":[0-9]*' | grep -o '[0-9]*')
  ELAPSED=$(echo "$JOB_STATUS" | grep -o '"elapsed_seconds":[0-9.]*' | grep -o '[0-9.]*')

  echo ""
  echo "✓ Import completed in ${EXEC_DURATION}s (total wait time)"
  echo "✓ Imported: $IMPORTED"
  echo "✓ Skipped: $SKIPPED"
  echo "✓ Failed: $FAILED"
  echo "✓ Processing time: ${ELAPSED}s"
else
  echo ""
  echo "✗ Import failed or timed out with status: $STATUS"
fi
echo ""

# Test 5: List all jobs
echo "TEST 5: GET /api/import/jobs"
echo "-------------------------------------------"
ALL_JOBS=$(curl -s "$BASE_URL/api/import/jobs")
JOB_COUNT=$(echo "$ALL_JOBS" | grep -o '"id"' | wc -l)
echo "✓ Total jobs in queue: $JOB_COUNT"
echo "$ALL_JOBS" > "$RESULTS_DIR/all_jobs.json"
echo ""

# Summary
echo "============================================"
echo "Test Summary"
echo "============================================"
echo "Preview time: ${PREVIEW_MS}ms"
echo "Import time: ${EXEC_DURATION}s"
echo "Status: $STATUS"
echo ""
echo "Results saved to: $RESULTS_DIR"
echo ""
