"""
Authorization script for Telethon client.
Run this once to authorize the Telegram user account.
"""

import os
import asyncio
from telethon import TelegramClient

API_ID = int(os.getenv("TELEGRAM_API_ID", input("Enter your API ID: ")))
API_HASH = os.getenv("TELEGRAM_API_HASH", input("Enter your API HASH: "))
SESSION_NAME = os.getenv("SESSION_NAME", "fasttelethon_session")


async def main():
    client = TelegramClient(SESSION_NAME, API_ID, API_HASH)
    
    await client.start()
    
    print("Authorization successful!")
    print(f"Session saved to: {SESSION_NAME}.session")
    print("You can now use this session file with the FastTelethon service.")
    
    # Test the connection
    me = await client.get_me()
    print(f"Logged in as: {me.first_name} (@{me.username})")
    
    await client.disconnect()


if __name__ == "__main__":
    asyncio.run(main())
