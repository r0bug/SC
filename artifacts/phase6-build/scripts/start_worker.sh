#!/bin/bash
# Start SagensContact Background Worker
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

export DATABASE_URL="${DATABASE_URL:-sqlite:$APP_ROOT/data/sagenscontact.db}"

echo "Starting SagensContact Background Worker..."
echo "Database: $DATABASE_URL"
echo ""

cd "$APP_ROOT"
exec "$APP_ROOT/bin/worker"
