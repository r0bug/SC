#!/bin/bash
# Run load tests on import system

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_DATA_DIR="$PROJECT_ROOT/sample_data/load_tests"
RESULTS_DIR="$PROJECT_ROOT/test_results"

mkdir -p "$RESULTS_DIR"

echo "🧪 SagensContact Load Testing"
echo "=============================="
echo ""

# Check if CLI exists
CLI_BIN="$PROJECT_ROOT/target/release/cli_client"
if [ ! -f "$CLI_BIN" ]; then
    echo "❌ CLI binary not found. Building..."
    cd "$PROJECT_ROOT"
    cargo build --release --bin cli_client
fi

# Function to run import and measure performance
run_import_test() {
    local file="$1"
    local name="$2"
    local result_file="$RESULTS_DIR/${name}_results.txt"

    echo "📊 Testing: $name"
    echo "  File: $(basename $file)"
    echo "  Size: $(du -h $file | cut -f1)"
    echo ""

    # Run with time measurement
    echo "Results for $name" > "$result_file"
    echo "File: $file" >> "$result_file"
    echo "Date: $(date)" >> "$result_file"
    echo "----------------------------------------" >> "$result_file"

    # Use /usr/bin/time for detailed metrics
    /usr/bin/time -v $CLI_BIN import --file "$file" 2>&1 | tee -a "$result_file"

    echo ""
    echo "  ✅ Results saved to: $result_file"
    echo ""
}

# Main test sequence
echo "Starting load tests..."
echo ""

# Test 1: 10k contacts
if [ -f "$TEST_DATA_DIR/contacts_10k.csv" ]; then
    run_import_test "$TEST_DATA_DIR/contacts_10k.csv" "10k_import"
fi

# Test 2: 50k with duplicates
if [ -f "$TEST_DATA_DIR/contacts_50k_with_duplicates.csv" ]; then
    run_import_test "$TEST_DATA_DIR/contacts_50k_with_duplicates.csv" "50k_dupes_import"
fi

# Test 3: 100k contacts
if [ -f "$TEST_DATA_DIR/contacts_100k.csv" ]; then
    run_import_test "$TEST_DATA_DIR/contacts_100k.csv" "100k_import"
fi

echo "📈 Load Testing Complete!"
echo ""
echo "Results Summary:"
echo "---------------"

for result in "$RESULTS_DIR"/*_results.txt; do
    if [ -f "$result" ]; then
        echo ""
        echo "$(basename $result):"
        # Extract key metrics
        grep -E "(Elapsed|Maximum resident|imported|skipped|failed)" "$result" || true
    fi
done

echo ""
echo "Full results available in: $RESULTS_DIR"
