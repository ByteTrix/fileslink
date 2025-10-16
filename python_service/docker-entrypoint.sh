#!/bin/bash
# Entrypoint script for FastTelethon service with session backup/restore

set -e

SESSION_DIR="/app/sessions"
SESSION_FILE="${SESSION_NAME:-/app/sessions/fasttelethon_session}.session"
BACKUP_DIR="/app/sessions/backups"

# Create necessary directories
mkdir -p "$SESSION_DIR"
mkdir -p "$BACKUP_DIR"

echo "📁 Session directory: $SESSION_DIR"
echo "📄 Session file: $SESSION_FILE"

# 🔑 RESTORE SESSION FROM ENVIRONMENT VARIABLE (for free tier without persistent disk)
if [ ! -z "$TELETHON_SESSION_BASE64" ]; then
    echo "🔓 Restoring session from TELETHON_SESSION_BASE64 environment variable..."
    echo "$TELETHON_SESSION_BASE64" | base64 -d > "$SESSION_FILE"
    if [ $? -eq 0 ]; then
        echo "✅ Session restored successfully from environment variable!"
        chmod 600 "$SESSION_FILE"
    else
        echo "❌ Failed to decode session from environment variable"
    fi
fi

# Check if session file exists
if [ -f "$SESSION_FILE" ]; then
    echo "✅ Found existing session file"
    
    # Create backup with timestamp
    BACKUP_FILE="$BACKUP_DIR/session_$(date +%Y%m%d_%H%M%S).session"
    cp "$SESSION_FILE" "$BACKUP_FILE"
    echo "💾 Backed up session to: $BACKUP_FILE"
    
    # Keep only last 5 backups
    cd "$BACKUP_DIR"
    ls -t session_*.session 2>/dev/null | tail -n +6 | xargs -r rm
    echo "🧹 Cleaned old backups (kept last 5)"
else
    echo "⚠️  No existing session file found"
    echo "   Authorization will be required on startup"
    echo "   Visit /auth endpoint after service starts"
fi

# Set proper permissions
chmod 600 "$SESSION_FILE" 2>/dev/null || true
chmod -R 700 "$SESSION_DIR"

echo "🚀 Starting FastTelethon service..."
exec python -m uvicorn main:app --host 0.0.0.0 --port 8001
