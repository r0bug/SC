#!/bin/bash
# Start SagensContact Sync Service
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

export DATABASE_URL="${DATABASE_URL:-sqlite:$APP_ROOT/data/sagenscontact.db}"
export ATTACHMENT_STORAGE_PATH="${ATTACHMENT_STORAGE_PATH:-$APP_ROOT/data/attachments}"
export PORT="${PORT:-3000}"

mkdir -p "$APP_ROOT/data/attachments"

echo "Starting SagensContact Sync Service..."
echo "Database: $DATABASE_URL"
echo "Attachments: $ATTACHMENT_STORAGE_PATH"
echo "Port: $PORT"
echo ""

cd "$APP_ROOT"
exec "$APP_ROOT/bin/sync_service"
