# Configuration Reference

Complete guide to all FilesLink environment variables.

## Quick Reference

```bash
# Required
BOT_TOKEN=              # Telegram bot token
STORAGE_CHANNEL_ID=-    # Channel ID for storage

# Server
SERVER_PORT=8080
APP_FILE_DOMAIN=http://localhost:8080/files

# Telegram API
TELEGRAM_API_URL=https://api.telegram.org
TELEGRAM_LOCAL=false
TELEGRAM_API_ID=        # From my.telegram.org
TELEGRAM_API_HASH=      # From my.telegram.org

# Optional
RUST_LOG=info
ENABLE_FILES_ROUTE=false
FILESLINK_PIPE_PATH=/tmp/fileslink.pipe
```

## Required Variables

### `BOT_TOKEN` ⚠️

Your Telegram bot token from [@BotFather](https://t.me/botfather).

- **Format**: `123456789:ABC-DEF1234ghIkl-zyx57W2v1u123ew11`
- **Example**: `BOT_TOKEN=123456789:abcdefghijklmnop`

**How to get:**
1. Message @BotFather
2. Send `/newbot`
3. Follow instructions
4. Copy token provided

###STORAGE_CHANNEL_ID` ⚠️

ID of your private Telegram channel where files are stored.

- **Format**: `-1001234567890` (always starts with `-100`)
- **Example**: `STORAGE_CHANNEL_ID=-1001234567890`

**How to get:**
1. Create private channel
2. Add bot as admin
3. Forward message to [@userinfobot](https://t.me/userinfobot)
4. Copy the channel ID

See [Telegram Setup Guide](TELEGRAM_STORAGE_SETUP.md) for details.

## Server Configuration

### `SERVER_PORT`

Port where the application listens.

- **Default**: `8080`
- **Example**: `SERVER_PORT=8080`
- **Note**: Render/Railway use port 8080 by default

### `APP_FILE_DOMAIN`

Public URL where files are accessible. Used to generate download links.

- **Default**: `http://localhost:8080/files`
- **Local**: `http://localhost:8080/files`
- **Production**: `https://yourdomain.com/files`
- **Railway**: `https://yourapp.up.railway.app/files`
- **Render**: `https://yourapp.onrender.com/files`

**Important:** Must end with `/files`

## Telegram API Configuration

### `TELEGRAM_API_URL`

Telegram Bot API server URL.

- **Official API**: `https://api.telegram.org`
- **Local (Docker)**: `http://nginx:80` or `http://localhost:8088`
- **Default**: `https://api.telegram.org`

**When to use local API:**
- Large files (>20MB)
- High volume uploads
- Need caching

**When to use official API:**
- Cloud hosting (Render, Railway, Fly.io)
- Simple deployments
- No local storage

### `TELEGRAM_LOCAL`

Whether using local Telegram Bot API server.

- **Values**: `true` or `false`
- **Default**: `false`
- **Set to `true`** when using local API in Docker
- **Set to `false`** for official API or cloud hosting

### `TELEGRAM_API_ID`

Your Telegram API ID from [my.telegram.org](https://my.telegram.org/).

- **Required for**: Local Telegram Bot API server
- **Optional for**: Official API
- **Format**: Integer
- **Example**: `TELEGRAM_API_ID=1234567`

### `TELEGRAM_API_HASH`

Your Telegram API hash from [my.telegram.org](https://my.telegram.org/).

- **Required for**: Local Telegram Bot API server
- **Optional for**: Official API
- **Format**: String (32 characters)
- **Example**: `TELEGRAM_API_HASH=abcdef1234567890abcdef1234567890`

## Logging & Debug

### `RUST_LOG`

Application log level.

- **Default**: `info`
- **Values**: `error`, `warn`, `info`, `debug`, `trace`
- **Production**: Use `warn` or `error`
- **Development**: Use `info` or `debug`
- **Example**: `RUST_LOG=info`

## Advanced Options

### `ENABLE_FILES_ROUTE`

Enable `/files` endpoint to list all files.

- **Default**: `false`
- **Values**: `true` or `false`
- **Warning**: Not recommended for production (security risk)
- **Use case**: Development/debugging only

### `FILESLINK_PIPE_PATH`

Path to FIFO (named pipe) for CLI communication.

- **Default**: `/tmp/fileslink.pipe`
- **Docker**: `/app/fileslink.pipe`
- **Example**: `FILESLINK_PIPE_PATH=/tmp/fileslink.pipe`

**Note:** CLI uses this path. Must match between server and CLI.

## Environment Templates

### Local Development with Docker
```bash
BOT_TOKEN=123456789:abcdefghijklmnop
STORAGE_CHANNEL_ID=-1001234567890
SERVER_PORT=8080
APP_FILE_DOMAIN=http://localhost:8080/files
TELEGRAM_API_URL=http://nginx:80
TELEGRAM_LOCAL=true
TELEGRAM_API_ID=1234567
TELEGRAM_API_HASH=abcdef1234567890abcdef1234567890
RUST_LOG=info
FILESLINK_PIPE_PATH=/app/fileslink.pipe
```

### Cloud Deployment (Render/Railway/Fly.io)
```bash
BOT_TOKEN=123456789:abcdefghijklmnop
STORAGE_CHANNEL_ID=-1001234567890
SERVER_PORT=8080
APP_FILE_DOMAIN=https://yourapp.onrender.com/files
TELEGRAM_API_URL=https://api.telegram.org
TELEGRAM_LOCAL=false
TELEGRAM_API_ID=1234567
TELEGRAM_API_HASH=abcdef1234567890abcdef1234567890
RUST_LOG=warn
FILESLINK_PIPE_PATH=/app/fileslink.pipe
```

### VPS/Self-Hosted
```bash
BOT_TOKEN=123456789:abcdefghijklmnop
STORAGE_CHANNEL_ID=-1001234567890
SERVER_PORT=8080
APP_FILE_DOMAIN=https://files.yourdomain.com/files
TELEGRAM_API_URL=https://api.telegram.org
TELEGRAM_LOCAL=false
TELEGRAM_API_ID=1234567
TELEGRAM_API_HASH=abcdef1234567890abcdef1234567890
RUST_LOG=warn
ENABLE_FILES_ROUTE=false
FILESLINK_PIPE_PATH=/app/fileslink.pipe
```

## Validation

### Check Configuration
```bash
# View environment
docker compose config

# Test connection
docker compose logs fileslink-app
```

### Common Mistakes

❌ **Wrong channel ID format**
```bash
STORAGE_CHANNEL_ID=1234567890  # Missing -100 prefix
```

✅ **Correct format**
```bash
STORAGE_CHANNEL_ID=-1001234567890
```

❌ **Wrong domain format**
```bash
APP_FILE_DOMAIN=http://localhost:8080  # Missing /files
```

✅ **Correct format**
```bash
APP_FILE_DOMAIN=http://localhost:8080/files
```

## Security Best Practices

1. **Never commit `.env`** - Add to `.gitignore`
2. **Use environment secrets** on cloud platforms
3. **Rotate tokens** regularly
4. **Use HTTPS** in production
5. **Restrict `ENABLE_FILES_ROUTE`** to dev only

## Next Steps

- [Installation Guide](INSTALLATION.md)
- [Telegram Setup](TELEGRAM_STORAGE_SETUP.md)
- [Deployment Guide](DEPLOYMENT.md)
- [Troubleshooting](FAQ.md)
