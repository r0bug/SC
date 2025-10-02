#!/bin/bash
# Start SagensContact Web UI
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

export PORT="${PORT:-3001}"
export API_URL="${API_URL:-http://localhost:3000}"

cd "$APP_ROOT/web"

echo "Starting SagensContact Web UI..."
echo "Port: $PORT"
echo "API URL: $API_URL"
echo ""

# Check if node is available
if ! command -v node &> /dev/null; then
    echo "Error: Node.js is not installed"
    echo "Install Node.js 20+ from https://nodejs.org/"
    exit 1
fi

# Serve the built application
npx sirv build --port $PORT --host 0.0.0.0
