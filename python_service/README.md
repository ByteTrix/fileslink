# FastTelethon Service

High-speed Telegram file upload/download service using MTProto Client API.

## Features

- ✅ **No file size limits** (up to 2GB per file)
- ✅ **Fast parallel upload/download** (10-20x faster than bot API)
- ✅ **Streaming downloads** (memory efficient)
- ✅ **Progress tracking**
- ✅ **REST API** for easy integration

## Setup

### 1. Get Telegram API Credentials

1. Go to https://my.telegram.org/apps
2. Create a new application
3. Note down your `API_ID` and `API_HASH`

### 2. Configure Environment

Add to your `.env` file:

```env
TELEGRAM_API_ID=your_api_id
TELEGRAM_API_HASH=your_api_hash
TELEGRAM_PHONE=+1234567890  # Your phone with country code
STORAGE_CHANNEL_ID=@your_channel_username
```

### 3. Start Service and Authorize

**New!** Authorization is now done through a web interface:

```bash
# Start the service
docker-compose up -d fasttelethon

# The service will automatically send a verification code to your phone
# Then visit: http://localhost:8001/auth

# Follow the web interface to:
# 1. Enter verification code from Telegram
# 2. Enter 2FA password (if enabled)
```

The session is saved automatically and persists across restarts.

### 3. Configure Environment

Add to your `.env` file:

```env
TELEGRAM_API_ID=your_api_id
TELEGRAM_API_HASH=your_api_hash
STORAGE_CHANNEL_ID=@your_channel_or_chat_id
```

### 4. Run with Docker

```bash
docker-compose up fasttelethon
```

## API Endpoints

### Health Check
```
GET /health
```

### Upload File
```
POST /upload
Content-Type: multipart/form-data

file: <binary file data>
channel_id: (optional) override default channel
```

Response:
```json
{
  "success": true,
  "message_id": 12345,
  "file_name": "video.mp4",
  "file_size": 164000000,
  "channel_id": "@mychannel"
}
```

### Download File
```
GET /download/{channel_id}/{message_id}
```

Returns file as streaming response with proper headers.

### Get File Info
```
GET /file-info/{channel_id}/{message_id}
```

Response:
```json
{
  "message_id": 12345,
  "file_name": "video.mp4",
  "file_size": 164000000,
  "mime_type": "video/mp4",
  "date": "2025-10-17T12:00:00"
}
```

## Integration with Rust Backend

The Rust server automatically proxies large file requests to this service:

1. When a file >20MB is uploaded to the bot, it's forwarded to FastTelethon
2. The download link uses the FastTelethon endpoint
3. Users get fast downloads without hitting bot API limits

## Performance

- **Upload speed**: 10-30 MB/s (depends on your connection)
- **Download speed**: 10-30 MB/s (depends on your connection)
- **File size limit**: 2GB (Telegram limit)

## Notes

- The service uses a userbot session, not a bot token
- Make sure to keep your `.session` file secure
- For production, mount the session file as a Docker volume
- Install `cryptg` for faster encryption (included in requirements.txt)
