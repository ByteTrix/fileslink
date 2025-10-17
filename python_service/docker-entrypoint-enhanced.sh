#!/bin/bash
# Enhanced entrypoint script for FastTelethon service with session backup/restore

set -e

SESSION_DIR="/app/sessions"
SESSION_FILE="/app/sessions/fasttelethon_session.session"
BACKUP_DIR="/app/sessions/backups"

# Create necessary directories
mkdir -p "$SESSION_DIR"
mkdir -p "$BACKUP_DIR"

echo "📁 Session directory: $SESSION_DIR"
echo "📄 Session file: $SESSION_FILE"

# 🔑 RESTORE SESSION FROM ENVIRONMENT VARIABLE (for free tier without persistent disk)
if [ ! -z "$TELETHON_SESSION_BASE64" ]; then
    echo "🔓 Restoring session from TELETHON_SESSION_BASE64 environment variable..."
    echo "📊 Session length: ${#TELETHON_SESSION_BASE64} chars"
    echo "$TELETHON_SESSION_BASE64" | base64 -d > "$SESSION_FILE"
    if [ $? -eq 0 ]; then
        echo "✅ Session restored successfully from environment variable!"
        echo "📏 Session file size: $(stat -c%s "$SESSION_FILE") bytes"
        chmod 600 "$SESSION_FILE"
    else
        echo "❌ Failed to decode session from environment variable"
    fi
else
    echo "⚠️  TELETHON_SESSION_BASE64 environment variable not set"
fi

# Check if session file exists
if [ -f "$SESSION_FILE" ]; then
    echo "✅ Found existing session file"
    echo "📏 Session file size: $(stat -c%s "$SESSION_FILE") bytes"
    
    # Create backup with timestamp
    BACKUP_FILE="$BACKUP_DIR/session_$(date +%Y%m%d_%H%M%S).session"
    cp "$SESSION_FILE" "$BACKUP_FILE"
    echo "💾 Backed up session to: $BACKUP_FILE"
    
    # Keep only last 5 backups
    cd "$BACKUP_DIR"
    ls -t session_*.session 2>/dev/null | tail -n +6 | xargs -r rm
    echo "🧹 Cleaned old backups (kept last 5)"
else
    echo "⚠️  No existing session file found at $SESSION_FILE"
    echo "   Authorization will be required on startup"
    echo "   Visit /auth endpoint after service starts"
fi

# Set proper permissions
chmod 600 "$SESSION_FILE" 2>/dev/null || true
chmod -R 700 "$SESSION_DIR"

# Ensure we're in the app directory
cd /app

echo "🚀 Starting FastTelethon service..."
echo "📍 Working directory: $(pwd)"
echo "📂 Files in /app:"
ls -la /app
echo "📂 Files in /app/sessions:"
ls -la /app/sessions || echo "   (sessions directory empty or doesn't exist)"

exec python -m uvicorn main:app --host 0.0.0.0 --port 8001
