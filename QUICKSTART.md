# 🚀 Quick Start - Telegram Storage

## Prerequisites
- Rust installed
- Telegram bot token from [@BotFather](https://t.me/BotFather)
- Private Telegram channel created

## Setup Steps

### 1. Create Storage Channel
```
1. Open Telegram
2. Create new channel (private)
3. Add your bot as admin with "Post Messages" permission
```

### 2. Get Channel ID
Forward any message from your channel to [@RawDataBot](https://t.me/RawDataBot)
Copy the ID (looks like `-1001234567890`)

### 3. Configure Environment
Create `.env` file:
```env
BOT_TOKEN=your_bot_token_here
SERVER_PORT=8080
APP_FILE_DOMAIN=http://localhost:8080/files
TELEGRAM_API_URL=https://api.telegram.org
STORAGE_CHANNEL_ID=-1001234567890
ENABLE_FILES_ROUTE=false
```

### 4. Run
```bash
cargo run
```

### 5. Test
- Send a file to your bot
- Click the download link you receive
- File should download from Telegram storage!

## That's It! 🎉

Files are now stored on Telegram's servers, and your download links work perfectly.

See `TELEGRAM_STORAGE_SETUP.md` for detailed documentation.
