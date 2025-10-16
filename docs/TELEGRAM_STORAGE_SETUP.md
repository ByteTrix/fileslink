# Telegram Storage Setup

## How to Set Up Your Storage Channel
1. Create a private Telegram channel
2. Add your bot as an administrator
3. Forward any message from your channel to [@userinfobot](https://t.me/userinfobot)
4. Copy the channel ID (starts with `-100`)
5. Add the channel ID to your `.env` as `STORAGE_CHANNEL_ID`

## Troubleshooting
- Make sure your bot has permission to post in the channel
- Channel ID must start with `-100`
