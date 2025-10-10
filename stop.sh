#!/bin/bash

# SagensContact Stop Script
# Stops all running SagensContact services

echo "🛑 Stopping SagensContact..."

# Kill sync service
pkill -f sync_service && echo "✓ Stopped backend (sync_service)"

# Kill web UI
pkill -f "vite.*sagenscontact" && echo "✓ Stopped frontend (vite)"
pkill -f "node.*vite" && echo "✓ Stopped frontend (node)"

# Kill any processes on ports 3001 and 3002
fuser -k 3001/tcp 2>/dev/null && echo "✓ Freed port 3001"
fuser -k 3002/tcp 2>/dev/null && echo "✓ Freed port 3002"

sleep 1

echo ""
echo "✓ All services stopped"
