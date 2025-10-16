# Quick Start Guide

Get FilesLink running in 5 minutes.

## Prerequisites

Before you start, obtain:
1. Telegram bot token from [@BotFather](https://t.me/botfather)
2. Private Telegram channel for storage
3. Channel ID from [@userinfobot](https://t.me/userinfobot)

## Step 1: Get Your Bot Token

1. Message [@BotFather](https://t.me/botfather) on Telegram
2. Send `/newbot`
3. Follow instructions to create your bot
4. Copy the token (format: `123456789:ABC-DEF...`)

## Step 2: Create Storage Channel

1. Create a new **private** Telegram channel
2. Add your bot as administrator
3. Give bot "Post Messages" permission
4. Forward any message from the channel to [@userinfobot](https://t.me/userinfobot)
5. Copy the channel ID (starts with `-100`)

## Step 3: Deploy with Docker

```bash
# Clone repository
git clone https://github.com/bytetrix/fileslink.git
cd fileslink

# Configure environment
cp .env.example .env
nano .env  # or use your favorite editor

# Add your credentials:
# BOT_TOKEN=your_token_here
# STORAGE_CHANNEL_ID=-1001234567890

# Start services
docker compose up -d
```

## Step 4: Test

1. Send a file to your bot on Telegram
2. Bot responds with a download link
3. Click the link to verify it works

**That's it!** Your FilesLink bot is now running at `http://localhost:8080`.

## What's Next?

- [Configure environment variables](CONFIGURATION.md)
- [Set up permissions](PERMISSIONS.md)
- [Deploy to the cloud](DEPLOYMENT.md)
- [Learn about CLI commands](CLI.md)

## Troubleshooting

### "Chat not found" error
- Verify channel ID is correct (use @userinfobot)
- Ensure bot is admin in the channel

### Docker won't start
```bash
docker compose logs fileslink-app
```

### Need help?
Check the [FAQ](FAQ.md) or open an issue on GitHub.
