#!/bin/bash
# Simple entrypoint for FastTelethon service

set -e

# Create sessions directory
mkdir -p /app/sessions

echo "🚀 Starting FastTelethon service..."
exec python -m uvicorn main:app --host 0.0.0.0 --port 8001
