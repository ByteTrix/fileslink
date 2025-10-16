# FilesLink Installation & Setup

## Prerequisites
- Telegram bot token from [BotFather](https://core.telegram.org/bots#botfather)
- Private Telegram channel for storage
- Docker (recommended) or Rust toolchain

## Environment Variables
Copy `.env.example` to `.env` and fill in:

```text
BOT_TOKEN=your_bot_token
SERVER_PORT=8080
APP_FILE_DOMAIN=http://localhost:8080/files
TELEGRAM_API_URL=http://localhost:8088
FILESLINK_PIPE_PATH=/tmp/fileslink.pipe
STORAGE_CHANNEL_ID=-1001234567890
TELEGRAM_API_ID=your_api_id
TELEGRAM_API_HASH=your_api_hash
TELEGRAM_LOCAL=true
```

See [docs/TELEGRAM_STORAGE_SETUP.md](TELEGRAM_STORAGE_SETUP.md) for channel setup and how to get the channel ID using @userinfobot.

## Docker Setup
```bash
git clone https://github.com/kvnxo/fileslink.git
cd fileslink
cp .env.example .env   # Edit with your credentials
docker compose up -d
```

## Local Setup
1. Install Rust: https://rustup.rs/
2. Build and run:
   ```bash
   cargo build --release
   cargo run --release
   ```

## Access
- The bot will be available at `http://localhost:8080`
- Uploaded files are stored in your Telegram channel

## Next Steps
- [Telegram Storage Setup](TELEGRAM_STORAGE_SETUP.md)
- [CLI Usage](CLI.md)
- [Permissions](PERMISSIONS.md)
