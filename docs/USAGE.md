# FilesLink Usage Guide

## Uploading Files
- Send a file to the bot
- Bot stores it in your Telegram channel
- You receive a download link

## Downloading from URLs
- Send `/url <link>` to the bot
- Bot downloads and stores the file
- You receive a download link

## Downloading Files
- Click the link provided by the bot
- File streams directly from Telegram

## Bot Commands

The bot supports the following commands:

- `/help` — Show help and available commands
- `/list` — List the 10 most recent files with links
	- Pagination: `/list 2` (page number)
- `/showqueue` — Show current processing queue
- `/clearqueue` — Clear the queue (admin only)
- `/delete <id>` — Delete a file mapping by unique id (admin only)
- `/edit <id> <new_name.ext>` — Change stored filename (admin only)
- `/find <query>` — Search files by filename (returns up to 10 matches)

Notes:
- The unique id is the prefix in the link (before the first underscore).
- Links are generated in the format: `https://your-domain/files/<id>_<filename.ext>`
