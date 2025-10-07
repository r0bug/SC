#!/bin/bash

# Stop all existing services
echo "Stopping existing services..."
pkill -9 -f "sync_service"
pkill -9 -f "pnpm dev"
pkill -9 -f "vite"
sleep 2

# Start backend on port 3002
echo "Starting backend on port 3002..."
cd /home/robug/Projects/sagenscontact/alpha
PORT=3002 DATABASE_URL="sqlite:data/contacts.db" ~/.cargo/bin/cargo run --release --bin sync_service > /tmp/backend.log 2>&1 &
BACKEND_PID=$!

# Wait for backend to start
sleep 5

# Start frontend on port 3001 (configured in vite.config.ts)
echo "Starting frontend on port 3001..."
cd /home/robug/Projects/sagenscontact/alpha/apps/web
pnpm dev > /tmp/frontend.log 2>&1 &
FRONTEND_PID=$!

# Wait for frontend to start
sleep 3

echo ""
echo "Services started:"
echo "  Backend PID: $BACKEND_PID (port 3002)"
echo "  Frontend PID: $FRONTEND_PID (port 3001)"
echo ""
echo "Access the app at: http://localhost:3001"
echo "Backend logs: tail -f /tmp/backend.log"
echo "Frontend logs: tail -f /tmp/frontend.log"
