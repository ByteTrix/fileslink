# 📂 FilesLink

> **Secure file sharing powered by Telegram cloud storage**

FilesLink is a Telegram bot that transforms your private channel into unlimited cloud storage. Upload files, get instant download links, no local disk required.

[![Docker](https://img.shields.io/badge/docker-supported-blue)](https://hub.docker.com/r/kvnxo/fileslink)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE.md)
[![Rust](https://img.shields.io/badge/rust-1.90+-orange)](https://www.rust-lang.org)

## Features

- 📤 **Instant Upload** - Send files to bot, receive download links
- ☁️ **Telegram Storage** - Up to 2GB per file, unlimited storage
- ⚡ **FastTelethon** - Bypass 20MB bot API limit with MTProto (also 10-30 MB/s speeds)
- 🔗 **URL Download** - Fetch external files directly
- 🔒 **Access Control** - Granular per-chat permissions
- 🚀 **Fast Streaming** - Direct downloads from Telegram
- 🐳 **Docker Ready** - One-command deployment

## Quick Start

```bash
git clone https://github.com/bytetrix/fileslink.git
cd fileslink
cp .env.example .env  # Add your credentials
docker compose up -d
```

**Access:** `http://localhost:8080`

## Documentation

| Guide | Description |
|-------|-------------|
| [📦 Installation](docs/INSTALLATION.md) | Docker, local, and production setup |
| [⚙️ Configuration](docs/CONFIGURATION.md) | All environment variables explained |
| [📱 Telegram Setup](docs/TELEGRAM_STORAGE_SETUP.md) | Configure your storage channel |
| [⚡ FastTelethon Setup](docs/SETUP_FASTTELETHON.md) | Enable large file support (>20MB) |
| [🎯 Usage](docs/USAGE.md) | How to use the bot |
| [🛡️ Permissions](docs/PERMISSIONS.md) | Access control configuration |
| [🔧 CLI](docs/CLI.md) | Command-line interface |
| [🌐 Render Deployment](docs/RENDER_DEPLOYMENT.md) | Deploy to Render.com (free tier) |
| [❓ FAQ](docs/FAQ.md) | Common questions and troubleshooting |
| [🏗️ Architecture](docs/ARCHITECTURE.md) | Technical details |

## Requirements

**Required:**
- Telegram bot token (get from [BotFather](https://t.me/botfather))
- Private Telegram channel
- Channel ID (get via [@userinfobot](https://t.me/userinfobot))

**Optional:**
- Docker (recommended)
- Rust 1.90+ (for local development)

## Environment Setup

```bash
BOT_TOKEN=123456789:ABC-DEF...
STORAGE_CHANNEL_ID=-1001234567890
APP_FILE_DOMAIN=https://yourdomain.com/files
```

See [Configuration Guide](docs/CONFIGURATION.md) for all options.

## Cloud Deployment

Deploy to free hosting in minutes:

- **Render** - 100% free with [Cloudflare keep-alive](https://github.com/ByteTrix/cloudflare-render-ping) (no sleep!)
- **Railway** - easiest setup
- **Fly.io** - Global edge deployment
- **VPS** - Full control

See [Deployment Guide](docs/DEPLOYMENT.md) for details.

## Use Cases

- Team file sharing
- Cloud backup solution
- URL-to-file conversion
- Personal file hosting
- Telegram file manager

## Tech Stack

- **Backend:** Rust (Tokio, Axum)
- **Bot:** Teloxide  
- **Storage:** Telegram Cloud (2GB per file)
- **Large Files:** FastTelethon (Python MTProto client for files >20MB)
- **Deployment:** Docker

## License

MIT © [Bytetrix](https://github.com/bytetrix)

---

**Need help?** Check the [FAQ](docs/FAQ.md) or open an issue.
