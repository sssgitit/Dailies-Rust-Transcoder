#!/bin/bash

# Industrial Transcoder Launch Script
# Double-click this file to start the app

cd "$(dirname "$0")"

echo "=========================================="
echo "  Industrial Transcoder"
echo "  Starting application..."
echo "=========================================="
echo ""

# Check if node_modules exists
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies (first time only)..."
    npm install
fi

echo "🚀 Launching Industrial Transcoder..."
echo ""
echo "The app will open in a new window."
echo "You can close this terminal window once the app opens."
echo ""

npm run tauri:dev

