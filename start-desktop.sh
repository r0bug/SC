#!/bin/bash
cd "$(dirname "$0")"

# Check if sync_service is running (responds on port 3002)
if ! curl -s http://localhost:3002/health > /dev/null 2>&1; then
    echo "Starting sync_service..."
    ./target/release/sync_service &
    sleep 2
fi

echo "Starting SagensContact Desktop..."
echo "API Server: http://localhost:3002"

# Start Tauri desktop app
cd apps/desktop
pnpm run tauri dev
