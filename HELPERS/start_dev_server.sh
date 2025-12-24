#!/bin/bash

# Start a simple HTTP server for Tauri dev mode
cd "$(dirname "$0")/../ui"

# Try different methods to start a server
if command -v python3 &> /dev/null; then
    echo "Starting Python3 HTTP server on port 1420..."
    python3 -m http.server 1420
elif command -v python &> /dev/null; then
    echo "Starting Python HTTP server on port 1420..."
    python -m http.server 1420
elif command -v npx &> /dev/null; then
    echo "Starting serve via npx on port 1420..."
    npx --yes serve -p 1420 -s .
else
    echo "Error: No HTTP server found. Please install Python or Node.js."
    exit 1
fi

