#!/bin/bash

# SagensContact Web UI - Development Startup Script
# This script starts the web UI with improved stability and error handling

set -e

echo "🚀 SagensContact Web UI - Development Mode"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if sync_service is running
echo -n "Checking backend service... "
if curl -s http://localhost:3002/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${YELLOW}⚠ Not running${NC}"
    echo ""
    echo "Backend service is not running on port 3002."
    echo "Please start it in another terminal with:"
    echo "  cd /home/robug/Projects/sagenscontact/alpha"
    echo "  PORT=3002 cargo run --release --bin sync_service"
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check Node.js version
echo -n "Checking Node.js version... "
NODE_VERSION=$(node -v 2>/dev/null || echo "not found")
if [ "$NODE_VERSION" = "not found" ]; then
    echo -e "${RED}✗ Node.js not found${NC}"
    echo "Please install Node.js 20 LTS or later"
    exit 1
else
    echo -e "${GREEN}✓ $NODE_VERSION${NC}"
fi

# Check pnpm
echo -n "Checking pnpm... "
if command -v pnpm &> /dev/null; then
    PNPM_VERSION=$(pnpm -v)
    echo -e "${GREEN}✓ $PNPM_VERSION${NC}"
else
    echo -e "${RED}✗ pnpm not found${NC}"
    echo "Installing pnpm..."
    npm install -g pnpm
fi

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo ""
    echo "📦 Installing dependencies..."
    pnpm install
fi

# Clean previous build artifacts
echo ""
echo "🧹 Cleaning previous build..."
rm -rf .svelte-kit
rm -rf build

# Set environment variables
export NODE_ENV=development
export VITE_API_URL=http://localhost:3002

echo ""
echo "✨ Starting development server..."
echo ""
echo "  Web UI:      http://localhost:3001"
echo "  API Proxy:   http://localhost:3001/api -> http://localhost:3002/api"
echo "  WebSocket:   ws://localhost:3002/ws"
echo ""
echo "Features enabled:"
echo "  ✓ Hot Module Replacement (HMR)"
echo "  ✓ API retry logic (3 attempts)"
echo "  ✓ WebSocket auto-reconnect"
echo "  ✓ Error boundaries"
echo "  ✓ Graceful error recovery"
echo ""
echo "Press Ctrl+C to stop"
echo "=========================================="
echo ""

# Start the dev server with better error handling
pnpm dev --host 0.0.0.0 --strictPort false 2>&1 | while IFS= read -r line; do
    # Colorize important messages
    if [[ $line == *"error"* ]] || [[ $line == *"Error"* ]]; then
        echo -e "${RED}$line${NC}"
    elif [[ $line == *"warning"* ]] || [[ $line == *"Warning"* ]]; then
        echo -e "${YELLOW}$line${NC}"
    elif [[ $line == *"ready"* ]] || [[ $line == *"Local:"* ]]; then
        echo -e "${GREEN}$line${NC}"
    else
        echo "$line"
    fi
done
