#!/bin/bash

# SagensContact Startup Script
# Starts the sync service backend and web UI frontend

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🚀 Starting SagensContact..."
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to check if port is in use
check_port() {
    lsof -i :$1 >/dev/null 2>&1
    return $?
}

# Function to kill process on port
kill_port() {
    local port=$1
    echo -e "${YELLOW}Killing process on port $port...${NC}"
    fuser -k $port/tcp 2>/dev/null || true
    sleep 1
}

# Check and kill existing processes
echo -e "${BLUE}Checking for existing processes...${NC}"
if check_port 3002; then
    kill_port 3002
fi
if check_port 3001; then
    kill_port 3001
fi

# Fix permissions on web directory
echo -e "${BLUE}Fixing permissions...${NC}"
cd apps/web
sudo chown -R $USER:$USER . || true
chmod -R u+w . 2>/dev/null || true

# Clean and reinstall if node_modules is corrupted
if [ ! -d "node_modules" ] || [ ! -f "node_modules/.pnpm/lock.yaml" ]; then
    echo -e "${YELLOW}Installing web dependencies...${NC}"
    rm -rf node_modules .svelte-kit 2>/dev/null || true
    pnpm install
fi

cd "$SCRIPT_DIR"

# Start sync service
echo ""
echo -e "${GREEN}Starting Sync Service (Backend API)...${NC}"
cd crates/sync_service
cargo run --release > /tmp/sagenscontact-backend.log 2>&1 &
BACKEND_PID=$!
echo -e "${GREEN}✓ Backend started (PID: $BACKEND_PID)${NC}"

# Wait for backend to start
echo -e "${BLUE}Waiting for backend to be ready...${NC}"
for i in {1..30}; do
    if curl -s http://localhost:3002/api/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Backend is ready!${NC}"
        break
    fi
    sleep 1
    echo -n "."
done
echo ""

# Start web UI
cd "$SCRIPT_DIR/apps/web"
echo ""
echo -e "${GREEN}Starting Web UI (Frontend)...${NC}"
pnpm dev > /tmp/sagenscontact-frontend.log 2>&1 &
FRONTEND_PID=$!
echo -e "${GREEN}✓ Frontend started (PID: $FRONTEND_PID)${NC}"

# Wait for frontend to start
echo -e "${BLUE}Waiting for frontend to be ready...${NC}"
for i in {1..30}; do
    if curl -s http://localhost:3001 > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Frontend is ready!${NC}"
        break
    fi
    sleep 1
    echo -n "."
done
echo ""

# Print status
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}   ✓ SagensContact is now running!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${BLUE}📱 Web UI:${NC}      http://localhost:3001"
echo -e "${BLUE}🔌 API Backend:${NC} http://localhost:3002"
echo ""
echo -e "${YELLOW}📝 Logs:${NC}"
echo -e "   Backend:  tail -f /tmp/sagenscontact-backend.log"
echo -e "   Frontend: tail -f /tmp/sagenscontact-frontend.log"
echo ""
echo -e "${YELLOW}🛑 To stop:${NC}"
echo -e "   kill $BACKEND_PID $FRONTEND_PID"
echo -e "   OR run: pkill -f 'sync_service|vite'"
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${BLUE}🌐 Opening browser...${NC}"
sleep 2

# Try to open browser
if command -v xdg-open > /dev/null; then
    xdg-open http://localhost:3001 2>/dev/null &
elif command -v open > /dev/null; then
    open http://localhost:3001 2>/dev/null &
fi

echo ""
echo -e "${GREEN}✨ Ready! Visit http://localhost:3001 to get started${NC}"
echo ""

# Keep script running and show logs
echo -e "${BLUE}Showing live logs (Ctrl+C to exit):${NC}"
echo ""
tail -f /tmp/sagenscontact-frontend.log /tmp/sagenscontact-backend.log
