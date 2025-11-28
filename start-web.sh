#!/bin/bash
cd "$(dirname "$0")"

# Check if sync_service is running (responds on port 3002)
if ! curl -s http://localhost:3002/health > /dev/null 2>&1; then
    echo "Starting sync_service..."
    ./target/release/sync_service &
    sleep 2
fi

# Start web dev server if not running
if ! pgrep -f "vite.*apps/web" > /dev/null; then
    cd apps/web
    pnpm dev &
    sleep 3
fi

# Open browser
xdg-open http://localhost:3001 2>/dev/null &

echo "SagensContact Web UI: http://localhost:3001"
echo "API Server: http://localhost:3002"
