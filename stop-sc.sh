#!/bin/bash
# Stop all SagensContact processes
echo "Stopping SagensContact..."
pkill -f "target.*sync_service" 2>/dev/null
pkill -f "vite.*sagenscontact" 2>/dev/null
echo "Done."
