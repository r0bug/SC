#!/bin/bash

echo "🚀 Starting SagensContact with Docker..."
echo ""

# Stop any existing containers
docker-compose down 2>/dev/null

# Build and start
docker-compose up --build

# When stopped, clean up
docker-compose down
