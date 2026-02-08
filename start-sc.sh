#!/bin/bash
# SagensContact Startup Script
# Starts the Rust backend and SvelteKit frontend

SC_DIR="/home/robug/sagenscontact"
LOG_DIR="$SC_DIR/logs"
mkdir -p "$LOG_DIR" "$SC_DIR/data"

export PATH="/home/robug/.cargo/bin:/usr/local/bin:/usr/bin:$PATH"

# Load environment variables from .env file
if [ -f "$SC_DIR/.env" ]; then
    set -a
    source "$SC_DIR/.env"
    set +a
    echo "Loaded .env configuration"
fi

# Kill any existing SC processes
pkill -f "target.*sync_service" 2>/dev/null
pkill -f "vite.*sagenscontact" 2>/dev/null
sleep 1

echo "========================================="
echo "  SagensContact - Starting Services"
echo "========================================="
echo ""

# Start backend
echo "[1/2] Starting backend API server..."
cd "$SC_DIR"
cargo run -p sync_service > "$LOG_DIR/backend.log" 2>&1 &
BACKEND_PID=$!
echo "       Backend PID: $BACKEND_PID"
echo "       Log: $LOG_DIR/backend.log"

# Wait for backend to be ready (up to 180s for first compilation)
echo "       Waiting for backend (compiling + starting)..."
for i in $(seq 1 180); do
    if curl -s http://localhost:3002 > /dev/null 2>&1; then
        echo "       Backend ready on port 3002!"
        break
    fi
    if ! kill -0 $BACKEND_PID 2>/dev/null; then
        echo "       ERROR: Backend failed to start. Check $LOG_DIR/backend.log"
        echo ""
        tail -20 "$LOG_DIR/backend.log"
        echo ""
        echo "Press Enter to close..."
        read
        exit 1
    fi
    sleep 1
done

# Start frontend
echo "[2/2] Starting frontend dev server..."
cd "$SC_DIR/apps/web"
pnpm dev > "$LOG_DIR/frontend.log" 2>&1 &
FRONTEND_PID=$!
echo "       Frontend PID: $FRONTEND_PID"
echo "       Log: $LOG_DIR/frontend.log"

# Wait for frontend
sleep 3
for i in $(seq 1 30); do
    if curl -s http://localhost:3001 > /dev/null 2>&1; then
        echo "       Frontend ready on port 3001!"
        break
    fi
    sleep 1
done

echo ""
echo "========================================="
echo "  SagensContact is running!"
echo "  Backend:  http://localhost:3002"
echo "  Frontend: http://localhost:3001"
echo "========================================="
echo ""
echo "Open http://localhost:3001 in your browser."
echo "Press Ctrl+C or close this window to stop."
echo ""

# Trap to clean up on exit
cleanup() {
    echo ""
    echo "Shutting down SagensContact..."
    kill $FRONTEND_PID 2>/dev/null
    kill $BACKEND_PID 2>/dev/null
    pkill -f "target.*sync_service" 2>/dev/null
    echo "Done."
}
trap cleanup EXIT INT TERM

# Keep running
wait $BACKEND_PID $FRONTEND_PID
