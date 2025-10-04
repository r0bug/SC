#!/bin/bash

# Performance Baseline Test - Week 5
# Tests 10k, 50k (with duplicates), and 100k datasets
# Captures runtime, memory peaks, and detailed metrics

BASE_URL="http://localhost:3002"
TEST_DATA_DIR="sample_data/load_tests"
RESULTS_DIR="test_results/performance_baselines"

mkdir -p "$RESULTS_DIR"

# Get process ID of sync_service
SYNC_PID=$(pgrep -f "sync_service")

echo "==========================================="
echo "Performance Baseline Test"
echo "==========================================="
echo "Server PID: $SYNC_PID"
echo "Test Data Directory: $TEST_DATA_DIR"
echo "Results Directory: $RESULTS_DIR"
echo ""

# Function to run import test
run_import_test() {
  local DATASET_NAME=$1
  local DATASET_FILE=$2
  local TEST_NAME=$3

  echo "==========================================="
  echo "TEST: $TEST_NAME"
  echo "==========================================="
  echo "Dataset: $DATASET_FILE"
  echo ""

  # Get initial memory
  INITIAL_MEM=$(ps -o rss= -p $SYNC_PID 2>/dev/null)
  if [ -z "$INITIAL_MEM" ]; then
    INITIAL_MEM=0
  fi
  INITIAL_MEM_MB=$((INITIAL_MEM / 1024))

  # Prepare config
  IMPORT_CONFIG='{
    "connector_id": "generic_csv",
    "dedupe_strategy": "skip",
    "match_criteria": "email",
    "dry_run": false
  }'

  # Start timer
  START_TIME=$(date +%s%N)

  # Execute import
  EXEC_RESPONSE=$(curl -s -X POST "$BASE_URL/api/import/execute" \
    -F "file=@$DATASET_FILE" \
    -F "config=$IMPORT_CONFIG")

  JOB_ID=$(echo "$EXEC_RESPONSE" | grep -o '"job_id":"[^"]*"' | grep -o ':"[^"]*"' | tr -d ':"')

  if [ -z "$JOB_ID" ]; then
    echo "✗ Failed to create import job"
    echo "$EXEC_RESPONSE"
    return 1
  fi

  echo "Job ID: $JOB_ID"

  # Poll for completion
  MAX_POLLS=120
  POLL_COUNT=0
  STATUS="pending"
  PEAK_MEM_MB=$INITIAL_MEM_MB

  while [ "$STATUS" != "completed" ] && [ "$STATUS" != "failed" ] && [ $POLL_COUNT -lt $MAX_POLLS ]; do
    sleep 1

    # Get current memory
    CURRENT_MEM=$(ps -o rss= -p $SYNC_PID 2>/dev/null)
    if [ ! -z "$CURRENT_MEM" ]; then
      CURRENT_MEM_MB=$((CURRENT_MEM / 1024))
      if [ $CURRENT_MEM_MB -gt $PEAK_MEM_MB ]; then
        PEAK_MEM_MB=$CURRENT_MEM_MB
      fi
    fi

    # Get job status
    JOB_STATUS=$(curl -s "$BASE_URL/api/import/jobs/$JOB_ID")
    STATUS=$(echo "$JOB_STATUS" | grep -o '"status":"[^"]*"' | grep -o ':"[^"]*"' | tr -d ':"')
    PHASE=$(echo "$JOB_STATUS" | grep -o '"phase":"[^"]*"' | grep -o ':"[^"]*"' | tr -d ':"')
    PROGRESS=$(echo "$JOB_STATUS" | grep -o '"current":[0-9]*' | grep -o '[0-9]*')

    POLL_COUNT=$((POLL_COUNT + 1))
    echo -ne "\r  Polling... Status: $STATUS | Phase: $PHASE | Progress: $PROGRESS | Mem: ${CURRENT_MEM_MB}MB"
  done

  echo ""

  # End timer
  END_TIME=$(date +%s%N)
  TOTAL_MS=$(( ($END_TIME - $START_TIME) / 1000000 ))
  TOTAL_SECONDS=$(echo "scale=3; $TOTAL_MS / 1000" | bc)

  # Extract results
  if [ "$STATUS" = "completed" ]; then
    IMPORTED=$(echo "$JOB_STATUS" | grep -o '"imported":[0-9]*' | grep -o '[0-9]*')
    SKIPPED=$(echo "$JOB_STATUS" | grep -o '"skipped":[0-9]*' | grep -o '[0-9]*')
    FAILED=$(echo "$JOB_STATUS" | grep -o '"failed":[0-9]*' | grep -o '[0-9]*')
    ELAPSED=$(echo "$JOB_STATUS" | grep -o '"elapsed_seconds":[0-9.]*' | grep -o '[0-9.]*')

    # Calculate throughput
    if [ ! -z "$IMPORTED" ] && [ "$IMPORTED" -gt 0 ]; then
      THROUGHPUT=$(echo "scale=0; $IMPORTED / $ELAPSED" | bc)
    else
      THROUGHPUT=0
    fi

    MEM_DELTA=$((PEAK_MEM_MB - INITIAL_MEM_MB))

    echo ""
    echo "✓ Import completed successfully"
    echo "-------------------------------------------"
    echo "Total time (wall clock):  ${TOTAL_SECONDS}s"
    echo "Processing time:          ${ELAPSED}s"
    echo "Imported:                 $IMPORTED"
    echo "Skipped:                  $SKIPPED"
    echo "Failed:                   $FAILED"
    echo "Throughput:               ${THROUGHPUT} records/sec"
    echo "Initial memory:           ${INITIAL_MEM_MB}MB"
    echo "Peak memory:              ${PEAK_MEM_MB}MB"
    echo "Memory delta:             +${MEM_DELTA}MB"
    echo ""

    # Save detailed results
    cat > "$RESULTS_DIR/${DATASET_NAME}_results.txt" <<EOF
Performance Baseline: $TEST_NAME
================================

Dataset:              $DATASET_FILE
Job ID:               $JOB_ID
Status:               $STATUS

TIMING
------
Total time:           ${TOTAL_SECONDS}s
Processing time:      ${ELAPSED}s

RESULTS
-------
Imported:             $IMPORTED
Skipped:              $SKIPPED
Failed:               $FAILED
Throughput:           ${THROUGHPUT} records/sec

MEMORY
------
Initial:              ${INITIAL_MEM_MB}MB
Peak:                 ${PEAK_MEM_MB}MB
Delta:                +${MEM_DELTA}MB

RAW JSON
--------
$(echo "$JOB_STATUS" | jq '.' 2>/dev/null || echo "$JOB_STATUS")
EOF

    # Append to summary
    echo "$DATASET_NAME,$IMPORTED,$ELAPSED,$THROUGHPUT,$INITIAL_MEM_MB,$PEAK_MEM_MB,$MEM_DELTA" >> "$RESULTS_DIR/summary.csv"

  else
    echo ""
    echo "✗ Import failed or timed out"
    echo "Status: $STATUS"
    echo "Job response: $JOB_STATUS" > "$RESULTS_DIR/${DATASET_NAME}_error.json"
  fi

  echo ""

  # Wait for cleanup
  sleep 2
}

# Initialize summary CSV
echo "dataset,imported,elapsed_sec,throughput_per_sec,initial_mem_mb,peak_mem_mb,mem_delta_mb" > "$RESULTS_DIR/summary.csv"

# Run tests
run_import_test "10k" "$TEST_DATA_DIR/contacts_10k.csv" "10,000 Contacts Import"
run_import_test "50k_duplicates" "$TEST_DATA_DIR/contacts_50k_with_duplicates.csv" "50,000 Contacts with 20% Duplicates"
run_import_test "100k" "$TEST_DATA_DIR/contacts_100k.csv" "100,000 Contacts Import"

# Final summary
echo "==========================================="
echo "PERFORMANCE BASELINE SUMMARY"
echo "==========================================="
echo ""
cat "$RESULTS_DIR/summary.csv" | column -t -s ','
echo ""
echo "Detailed results saved to: $RESULTS_DIR"
echo ""

# Check database size
DB_SIZE=$(du -h data/contacts.db | cut -f1)
CONTACT_COUNT=$(sqlite3 data/contacts.db "SELECT COUNT(*) FROM contacts;" 2>/dev/null)

echo "DATABASE STATUS"
echo "-------------------------------------------"
echo "Database size:   $DB_SIZE"
echo "Total contacts:  $CONTACT_COUNT"
echo ""
