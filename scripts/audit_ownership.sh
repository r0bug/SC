#!/bin/bash
# Ownership Audit Script for SagensContact Multi-User Isolation
# This script checks which tables have ownership columns and which repositories filter by user

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "========================================="
echo "SAGENSCONTACT OWNERSHIP AUDIT"
echo "========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check database schema for created_by/uploaded_by/user_id columns
echo "1. DATABASE SCHEMA ANALYSIS"
echo "-------------------------------------------"
echo ""
echo "Analyzing migrations.rs for ownership columns..."
echo ""

MIGRATIONS_FILE="$PROJECT_ROOT/crates/local_store/src/migrations.rs"

echo -e "${GREEN}Tables WITH ownership columns:${NC}"
grep -E "created_by|uploaded_by|user_id" "$MIGRATIONS_FILE" | grep "TEXT NOT NULL" | sed 's/^/  /'

echo ""
echo -e "${YELLOW}Tables WITHOUT ownership (checking all CREATE TABLE):${NC}"

# Extract all table names from CREATE TABLE statements
TABLES=$(grep -oP 'CREATE TABLE IF NOT EXISTS \K\w+' "$MIGRATIONS_FILE")

echo ""
for table in $TABLES; do
    # Check if table has created_by, uploaded_by, or user_id
    if ! grep -A 20 "CREATE TABLE IF NOT EXISTS $table" "$MIGRATIONS_FILE" | grep -qE "created_by|uploaded_by|user_id"; then
        echo -e "  ${RED}✗${NC} $table (no ownership column)"
    fi
done

echo ""
echo ""
echo "2. REPOSITORY ANALYSIS"
echo "-------------------------------------------"
echo ""

REPO_DIR="$PROJECT_ROOT/crates/local_store/src/repositories"

echo "Checking which repositories filter by user_id in list/search methods..."
echo ""

for repo_file in "$REPO_DIR"/*.rs; do
    filename=$(basename "$repo_file")

    # Skip test files and mod.rs
    if [[ "$filename" == *"_tests.rs" ]] || [[ "$filename" == "mod.rs" ]]; then
        continue
    fi

    repo_name="${filename%.rs}"

    # Check if repository has list() or search() methods
    has_list=$(grep -c "pub async fn list" "$repo_file" || true)
    has_search=$(grep -c "pub async fn search" "$repo_file" || true)

    if [[ $has_list -gt 0 ]] || [[ $has_search -gt 0 ]]; then
        # Check if methods filter by user_id or created_by
        filters_by_user=$(grep -E "WHERE created_by|WHERE.*user_id|user_id: Uuid" "$repo_file" || true)

        if [[ -n "$filters_by_user" ]]; then
            echo -e "  ${GREEN}✓${NC} $repo_name (filters by user)"
        else
            echo -e "  ${RED}✗${NC} $repo_name (NO user filtering)"
        fi
    fi
done

echo ""
echo ""
echo "3. FOREIGN KEY CONSTRAINTS"
echo "-------------------------------------------"
echo ""
echo "Checking for FOREIGN KEY constraints to users table..."
echo ""

FK_COUNT=$(grep -c "REFERENCES users(id)" "$MIGRATIONS_FILE" || true)
echo "Found $FK_COUNT foreign key constraints to users table"

echo ""
grep "FOREIGN KEY.*created_by.*REFERENCES users" "$MIGRATIONS_FILE" | sed 's/^/  /' || echo "  (none found)"
grep "FOREIGN KEY.*uploaded_by.*REFERENCES users" "$MIGRATIONS_FILE" | sed 's/^/  /' || echo "  (none found)"
grep "FOREIGN KEY.*user_id.*REFERENCES users" "$MIGRATIONS_FILE" | sed 's/^/  /' || echo "  (none found)"

echo ""
echo ""
echo "4. SYSTEM USER CHECK"
echo "-------------------------------------------"
echo ""
echo "Searching for system user references..."
echo ""

# Check for Uuid::nil() usage
NIL_UUID_COUNT=$(find "$PROJECT_ROOT/crates" -name "*.rs" -exec grep -l "Uuid::nil()" {} \; | wc -l)
echo "Files using Uuid::nil(): $NIL_UUID_COUNT"

if [[ $NIL_UUID_COUNT -gt 0 ]]; then
    echo ""
    echo "Files with Uuid::nil() references:"
    find "$PROJECT_ROOT/crates" -name "*.rs" -exec grep -l "Uuid::nil()" {} \; | sed 's/^/  /'
fi

# Check for system user constant
SYSTEM_USER_CONST=$(find "$PROJECT_ROOT/crates" -name "*.rs" -exec grep -l "SYSTEM_USER" {} \; | wc -l)
echo ""
echo "Files defining SYSTEM_USER constant: $SYSTEM_USER_CONST"

echo ""
echo ""
echo "5. CLI AND SERVICE COMPONENTS"
echo "-------------------------------------------"
echo ""

echo "Checking CLI client for user context loading..."
CLI_FILE="$PROJECT_ROOT/crates/cli_client/src/main.rs"
if grep -q "session.json\|user_id\|User context" "$CLI_FILE" 2>/dev/null; then
    echo -e "  ${GREEN}✓${NC} CLI has user context code"
else
    echo -e "  ${RED}✗${NC} CLI missing user context loading"
fi

echo ""
echo "Checking import service for user_id parameter..."
IMPORT_DIR="$PROJECT_ROOT/crates/import_service/src"
if grep -rq "user_id: Uuid" "$IMPORT_DIR" 2>/dev/null; then
    echo -e "  ${GREEN}✓${NC} Import service has user_id parameter"
else
    echo -e "  ${RED}✗${NC} Import service missing user_id parameter"
fi

echo ""
echo "Checking worker for system user context..."
WORKER_FILE="$PROJECT_ROOT/crates/communication_queue/src/bin/worker.rs"
if [[ -f "$WORKER_FILE" ]]; then
    if grep -q "SYSTEM_USER\|system user" "$WORKER_FILE" 2>/dev/null; then
        echo -e "  ${GREEN}✓${NC} Worker has system user context"
    else
        echo -e "  ${RED}✗${NC} Worker missing system user context"
    fi
else
    echo -e "  ${YELLOW}?${NC} Worker file not found"
fi

echo ""
echo ""
echo "========================================="
echo "AUDIT COMPLETE"
echo "========================================="
echo ""
echo "RECOMMENDATIONS:"
echo "  1. Add created_by to tables lacking ownership"
echo "  2. Update repositories to filter by user_id in list/search"
echo "  3. Create system user with fixed UUID"
echo "  4. Replace Uuid::nil() with SYSTEM_USER_ID"
echo "  5. Add user context to CLI and importers"
echo ""
