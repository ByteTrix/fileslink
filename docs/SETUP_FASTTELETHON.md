# FilesLink with FastTelethon Integration

## Setup Guide for Large File Support

This guide explains how to set up FilesLink with FastTelethon to support downloading files larger than 20MB (up to 2GB).

### Prerequisites

1. **Telegram Bot Token** (from @BotFather)
2. **Telegram API Credentials** (API_ID and API_HASH from https://my.telegram.org/apps)
3. **Storage Channel** (a private Telegram channel or chat to store files)
4. **Docker and Docker Compose** installed

### Step 1: Get Telegram API Credentials

1. Go to https://my.telegram.org/apps
2. Log in with your phone number
3. Create a new application (if you don't have one)
4. Note down your `api_id` (numeric) and `api_hash` (string)

### Step 2: Create Storage Channel

1. Open Telegram and create a new private channel
2. Note the channel username (e.g., `@my_files_storage`) or ID
3. Make sure your user account is an admin of this channel

### Step 3: Configure Environment Variables

Create or update your `.env` file:

```env
# Existing bot configuration
BOT_TOKEN=your_bot_token_here
STORAGE_CHANNEL_ID=@your_channel_username_or_id

# FastTelethon configuration (NEW)
TELEGRAM_API_ID=your_api_id
TELEGRAM_API_HASH=your_api_hash
TELEGRAM_PHONE=+1234567890  # Your phone number with country code
STORAGE_CHANNEL_ID=@your_channel_username_or_id

# Service URLs
FASTTELETHON_URL=http://fasttelethon:8001
APP_FILE_DOMAIN=https://yourdomain.com/files
```

### Step 4: Build and Run with Docker

**No manual authorization needed!** The service will handle it automatically through a web interface.

```bash
# Build all services
docker-compose build

# Start services
docker-compose up -d

# Check logs
docker-compose logs -f fasttelethon
```

### Step 5: Complete Authorization (First Time Only)

The FastTelethon service will automatically send a verification code to your phone number.

1. **Open your browser**: Go to http://localhost:8001/auth
2. **Enter verification code**: Check your Telegram app for the code
3. **Enter 2FA password** (if you have 2FA enabled)
4. **Done!** The service is now authorized and ready to use

The session will be saved automatically and persists across restarts.

```bash
# Build all services
docker-compose build

# Start services
docker-compose up -d

# Check logs
docker-compose logs -f
```

### Step 6: Test Large File Upload

1. Send a file larger than 20MB to your bot
2. The bot will forward it to your storage channel
3. You'll receive a download link
4. When someone clicks the link:
   - Files <20MB: Downloaded directly via Bot API
   - Files ≥20MB: Downloaded via FastTelethon (fast parallel download)

### Re-authorization (If Needed)

If your session expires or you need to re-authorize:

1. Stop the service: `docker-compose down`
2. Delete the session file: `rm python_service/sessions/*.session`
3. Start the service: `docker-compose up -d`
4. Visit http://localhost:8001/auth again

## Architecture

```
User → Bot (teloxide)
       ↓
       File received
       ↓
    [Size Check]
       ↓
    <20MB? → Upload via Bot API → Storage Channel
       ↓
    ≥20MB? → Forward to FastTelethon → Upload via MTProto → Storage Channel
                                              (fast, parallel)

Download:
User → HTTP Server (Rust/Axum)
       ↓
    [Try Bot API]
       ↓
    Too large error?
       ↓
    Proxy to FastTelethon → Download via MTProto → Stream to User
                                (fast, parallel)
```

## API Endpoints

### FastTelethon Service (Port 8001)

- `GET /health` - Health check
- `POST /upload` - Upload large file
- `GET /download/{channel_id}/{message_id}` - Download large file
- `GET /file-info/{channel_id}/{message_id}` - Get file metadata

### Main Server (Port 8080)

- `GET /files/{unique_id}_{filename}` - Download file (auto-proxies large files)
- `GET /files` - List all files (if enabled)

## Performance

| Method | File Size Limit | Speed | Notes |
|--------|----------------|-------|-------|
| Bot API | 20MB | 1-3 MB/s | Limited by Telegram's bot API |
| FastTelethon | 2GB | 10-30 MB/s | Parallel download, MTProto client |

## Troubleshooting

### Session File Issues

If the session expires or has issues:

```bash
# Delete old session
rm python_service/sessions/fasttelethon_session.session

# Re-authorize
cd python_service
python authorize.py
```

### FastTelethon Not Responding

Check logs:

```bash
docker-compose logs fasttelethon
```

Common issues:
- Session file not found → Re-authorize
- API credentials invalid → Check `.env` file
- Channel not accessible → Make sure your user is admin

### Large File Download Fails

1. Check if FastTelethon service is running:
   ```bash
   curl http://localhost:8001/health
   ```

2. Check if message exists in channel:
   ```bash
   curl http://localhost:8001/file-info/{channel_id}/{message_id}
   ```

3. Check Rust server logs:
   ```bash
   docker-compose logs app
   ```

## Security Notes

- **Keep `.session` files secure** - They grant full access to your Telegram account
- **Use environment variables** - Don't commit credentials to git
- **Restrict channel access** - Make storage channels private
- **Use HTTPS** - In production, serve files over HTTPS

## Updating

```bash
# Pull latest changes
git pull

# Rebuild containers
docker-compose build

# Restart services
docker-compose up -d
```

## Logs and Monitoring

```bash
# View all logs
docker-compose logs -f

# View specific service
docker-compose logs -f fasttelethon
docker-compose logs -f app

# Check service status
docker-compose ps
```

## Support

For issues, check:
1. Docker logs: `docker-compose logs`
2. Session file exists: `ls python_service/sessions/`
3. Environment variables set: `cat .env`
4. FastTelethon health: `curl http://localhost:8001/health`
