#!/bin/bash

# Function to check if a port is available
check_port() {
    local port=$1
    local max_attempts=30
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if ! lsof -i :$port > /dev/null; then
            return 0
        fi
        echo "Port $port is not available, waiting... (attempt $attempt/$max_attempts)"
        sleep 1
        ((attempt++))
    done
    return 1
}

# Function to check if a service is healthy
check_service_health() {
    local url=$1
    local max_attempts=60  # Increased max attempts
    local attempt=1
    local sleep_duration=2 # Increased sleep duration

    while [ $attempt -le $max_attempts ]; do
        if curl -s -o /dev/null -w "%{http_code}" "$url" | grep -q "200"; then
            echo "Service at $url is healthy"
            return 0
        fi
        echo "Service at $url is not healthy, waiting... (attempt $attempt/$max_attempts)"
        sleep $sleep_duration
        ((attempt++))
    done
    echo "Service at $url is not healthy after multiple attempts"
    return 1
}

# Function to cleanup processes
cleanup() {
    echo "Cleaning up..."
    if [ -n "$FRONTEND_PID" ]; then
        echo "Killing frontend server (PID: $FRONTEND_PID)"
        kill $FRONTEND_PID 2>/dev/null || true
    fi
    if [ -n "$API_PID" ]; then
        echo "Killing API server (PID: $API_PID)"
        kill $API_PID 2>/dev/null || true
    fi
    if [ -n "$XVFB_PID" ]; then
        echo "Killing Xvfb (PID: $XVFB_PID)"
        kill $XVFB_PID 2>/dev/null || true
    fi
}

# Set up trap for cleanup
trap cleanup EXIT INT TERM

# Check if ports are available
check_port 3000 || { echo "Port 3000 is in use"; exit 1; }
check_port 8080 || { echo "Port 8080 is in use"; exit 1; }

# Start Xvfb only if it's not already running
if ! pgrep -x Xvfb > /dev/null; then
    echo "Starting Xvfb..."
    Xvfb :99 -screen 0 1024x768x24 &
    XVFB_PID=$!
    export DISPLAY=:99
else
    echo "Xvfb is already running."
    export DISPLAY=:99
fi

# Start mock frontend server
npm run start:mock-frontend &
FRONTEND_PID=$!
echo "Starting mock frontend server..."

# Start mock API server and redirect output to a log file
node tests/e2e/mock-api/server.js > api-server.log 2>&1 &
API_PID=$!
echo "Starting mock API server..."

# Wait for services to become healthy
echo "Waiting for services to start..."
check_service_health http://localhost:3000 || { echo "Frontend service failed to start"; exit 1; }
check_service_health http://localhost:8080/test/reset || { echo "API service failed to start"; exit 1; }

# Run E2E tests
echo "Running E2E tests..."
npx cypress run

# Store the exit code
EXIT_CODE=$?

# Exit with the test result
exit $EXIT_CODE