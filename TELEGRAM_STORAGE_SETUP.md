# 📦 Telegram-as-Storage Setup Guide

This application now uses **Telegram as cloud storage** instead of local disk storage. All uploaded files are stored in a private Telegram channel.

## 🎯 Benefits

- ✅ **No local disk usage** - All files stored on Telegram's servers
- ✅ **Unlimited storage** - Telegram provides free cloud storage for files up to 2GB each
- ✅ **Automatic backups** - Files are backed up on Telegram's infrastructure
- ✅ **Remote access** - Access files from anywhere through Telegram
- ✅ **No file management** - No need to worry about disk cleanup or file organization

## 🔧 Setup Instructions

### Step 1: Create a Private Storage Channel

1. Open Telegram and create a **new channel** (not a group!)
2. Name it something like "FilesLink Storage" (private, for your use only)
3. Make sure it's set to **Private**
4. **Add your bot** as an administrator to the channel
   - Go to channel settings → Administrators → Add Administrator
   - Search for your bot username
   - Give it permission to **Post Messages**

### Step 2: Get the Channel ID

There are several ways to get your channel ID:

#### Method 1: Using the Bot
1. Forward any message from your storage channel to [@RawDataBot](https://t.me/RawDataBot)
2. It will show you the channel ID in the `"chat"` section
3. Look for `"id": -1001234567890` (it will be a negative number starting with -100)

#### Method 2: Using Web Telegram
1. Open [web.telegram.org](https://web.telegram.org)
2. Go to your storage channel
3. Look at the URL: `web.telegram.org/#/im?p=c1234567890`
4. Add `-100` prefix to the number: `-1001234567890`

#### Method 3: Using @userinfobot
1. Add [@userinfobot](https://t.me/userinfobot) to your channel as admin
2. Send any message to the channel
3. The bot will reply with the channel ID
4. Remove the bot after getting the ID

### Step 3: Configure Environment Variables

Add the following to your `.env` file:

```env
# Your Telegram bot token from BotFather
BOT_TOKEN=123456789:abcdefghijklmnop

# Port for the HTTP server
SERVER_PORT=8080

# Base URL for download links
APP_FILE_DOMAIN=http://yourdomain.com/files

# Telegram API URL (use official or local server)
TELEGRAM_API_URL=https://api.telegram.org

# Storage channel ID (REQUIRED - this is your private channel)
STORAGE_CHANNEL_ID=-1001234567890

# Optional: Enable file listing route
ENABLE_FILES_ROUTE=false
```

### Step 4: Verify Setup

1. Start the application:
   ```bash
   cargo run
   ```

2. Look for these log messages:
   ```
   File storage initialized
   Server is running at http://0.0.0.0:8080/
   ```

3. Send a test file to your bot

4. Check your storage channel - you should see:
   - The file posted by your bot
   - A caption with a unique ID (e.g., `a7k3m9x2`)

5. You'll receive a download link like:
   ```
   http://yourdomain.com/files/a7k3m9x2
   ```

## 🔄 How It Works

### Upload Flow:
```
User sends file → Bot receives → Forwards to storage channel → 
Saves metadata → Generates download link → Sends link to user
```

### Download Flow:
```
User clicks link → Server gets file metadata → Fetches file from Telegram → 
Streams to user's browser → File downloads
```

## 📊 File Metadata Storage

The app maintains a `file_mappings.json` file that maps unique IDs to Telegram file IDs:

```json
{
  "files": {
    "a7k3m9x2": {
      "unique_id": "a7k3m9x2",
      "telegram_file_id": "BQACAgIAAxkBAAIBB2...",
      "file_name": "document.pdf",
      "mime_type": "application/pdf",
      "file_size": 1234567,
      "uploaded_at": 1697472000
    }
  }
}
```

**Important:** Backup this file regularly! If lost, existing download links will break.

## 🔒 Security Considerations

1. **Keep your storage channel private** - Never share the invite link
2. **Backup `file_mappings.json`** - This is critical for link functionality
3. **Protect your `.env` file** - Contains sensitive credentials
4. **Use HTTPS in production** - Set `APP_FILE_DOMAIN` to `https://...`
5. **Monitor channel size** - Telegram has limits on channel storage

## 🐛 Troubleshooting

### Error: "Storage channel not configured"
- Make sure `STORAGE_CHANNEL_ID` is set in `.env`
- Verify the channel ID is correct (should start with `-100`)

### Error: "Failed to forward document"
- Check if the bot is an admin in the storage channel
- Verify the bot has "Post Messages" permission

### Error: "Failed to get file info from Telegram"
- The file might have been deleted from the storage channel
- Check if the channel still exists and bot is still an admin

### Files not downloading
- Verify `file_mappings.json` exists and contains the file entry
- Check if the file still exists in the storage channel
- Ensure `TELEGRAM_API_URL` is correct

### "File not found" error
- The file might have been uploaded before implementing Telegram storage
- The `file_mappings.json` might be corrupted or missing
- The unique ID in the URL might be incorrect

## 📝 Migration from Local Storage

If you were previously using local file storage:

1. **Old files won't be accessible** through the new system
2. You can manually upload important files through the bot again
3. Or keep the old `files/` directory for legacy links (requires code modification)

## 🎛️ Configuration Options

### Storage Channel Best Practices

- **One channel per bot instance** - Don't share channels between multiple bots
- **Regular backups** - Export `file_mappings.json` regularly
- **Monitor storage** - Keep an eye on the number of files in the channel
- **Private only** - Never make the storage channel public

### Performance Tuning

- For high-traffic scenarios, consider:
  - Using a local Telegram Bot API server for faster downloads
  - Implementing caching for frequently accessed files
  - Rate limiting download requests

## 🆘 Support

If you encounter issues:

1. Check the application logs for detailed error messages
2. Verify all environment variables are set correctly
3. Ensure your bot has proper permissions in the storage channel
4. Test with a small file first before uploading large files

## 📚 Additional Resources

- [Telegram Bot API Documentation](https://core.telegram.org/bots/api)
- [Telegram Channels Guide](https://telegram.org/tour/channels)
- [File Size Limits](https://core.telegram.org/bots/api#sending-files)
