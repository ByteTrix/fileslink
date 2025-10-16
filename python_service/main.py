"""
FastTelethon Service - HTTP API for large file upload/download via Telegram MTProto
This service uses Telethon client API to bypass Telegram Bot API's 20MB file limit.
"""

import os
import asyncio
import logging
from pathlib import Path
from typing import Optional
from contextlib import asynccontextmanager

from fastapi import FastAPI, HTTPException, Response, UploadFile, File, Form
from fastapi.responses import StreamingResponse, HTMLResponse
from telethon import TelegramClient, utils
from telethon.tl import types
from telethon.tl.types import PeerChannel
from telethon.errors import SessionPasswordNeededError, PhoneCodeInvalidError
import uvicorn

from FastTelethon import download_file, upload_file

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Configuration from environment
API_ID = int(os.getenv("TELEGRAM_API_ID", "0"))
API_HASH = os.getenv("TELEGRAM_API_HASH", "")
PHONE = os.getenv("TELEGRAM_PHONE", "")
SESSION_NAME = os.getenv("SESSION_NAME", "fasttelethon_session")
CHANNEL_ID = os.getenv("STORAGE_CHANNEL_ID", "")  # Channel to store files

# Global client and authorization state
client: Optional[TelegramClient] = None
auth_state = {
    "is_authorized": False,
    "needs_code": False,
    "needs_password": False,
    "phone_code_hash": None,
    "error": None
}


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Startup and shutdown events"""
    global client, auth_state
    
    if not API_ID or not API_HASH:
        logger.error("TELEGRAM_API_ID and TELEGRAM_API_HASH must be set!")
        raise ValueError("Missing Telegram API credentials")
    
    if not PHONE:
        logger.error("TELEGRAM_PHONE must be set in .env!")
        raise ValueError("Missing phone number")
    
    logger.info("Starting Telegram client...")
    client = TelegramClient(SESSION_NAME, API_ID, API_HASH)
    await client.connect()
    
    if not await client.is_user_authorized():
        logger.warning("Client not authorized. Sending code to phone...")
        try:
            # Send code automatically
            sent = await client.send_code_request(PHONE)
            auth_state["needs_code"] = True
            auth_state["phone_code_hash"] = sent.phone_code_hash
            logger.info(f"✅ Verification code sent to {PHONE}")
            logger.info("🌐 Visit http://localhost:8001/auth to complete authorization")
        except Exception as e:
            logger.error(f"Failed to send code: {e}")
            auth_state["error"] = str(e)
    else:
        auth_state["is_authorized"] = True
        logger.info("✅ Telegram client authorized and ready!")
    
    yield
    
    logger.info("Shutting down Telegram client...")
    await client.disconnect()


app = FastAPI(
    title="FastTelethon Service",
    description="High-speed Telegram file upload/download service",
    version="1.0.0",
    lifespan=lifespan
)


@app.get("/health")
async def health_check():
    """Health check endpoint"""
    return {
        "status": "healthy",
        "authorized": await client.is_user_authorized() if client else False,
        "needs_code": auth_state.get("needs_code", False),
        "needs_password": auth_state.get("needs_password", False)
    }


@app.get("/auth", response_class=HTMLResponse)
async def auth_page():
    """Web-based authorization page"""
    if auth_state["is_authorized"]:
        return """
        <!DOCTYPE html>
        <html>
        <head>
            <title>FastTelethon - Authorized</title>
            <style>
                body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }
                .success { color: green; font-size: 20px; }
            </style>
        </head>
        <body>
            <h1>✅ Already Authorized</h1>
            <p class="success">Your Telegram client is already authorized and working!</p>
            <p><a href="/health">Check health status</a></p>
        </body>
        </html>
        """
    
    if auth_state.get("error"):
        return f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>FastTelethon - Error</title>
            <style>
                body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
                .error {{ color: red; }}
            </style>
        </head>
        <body>
            <h1>❌ Authorization Error</h1>
            <p class="error">{auth_state["error"]}</p>
            <p>Please check your configuration and restart the service.</p>
        </body>
        </html>
        """
    
    if auth_state["needs_password"]:
        return """
        <!DOCTYPE html>
        <html>
        <head>
            <title>FastTelethon - 2FA Password</title>
            <style>
                body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }
                input[type="password"] { padding: 10px; font-size: 16px; width: 300px; }
                button { padding: 10px 20px; font-size: 16px; cursor: pointer; background: #0088cc; color: white; border: none; border-radius: 5px; }
                button:hover { background: #006699; }
                .info { background: #e7f3ff; padding: 15px; border-radius: 5px; margin: 20px 0; }
            </style>
        </head>
        <body>
            <h1>🔐 Enter 2FA Password</h1>
            <div class="info">
                <p>Your account has Two-Factor Authentication enabled.</p>
                <p>Please enter your cloud password (not the verification code).</p>
            </div>
            <form action="/auth/password" method="post">
                <input type="password" name="password" placeholder="2FA Password" required autofocus />
                <br><br>
                <button type="submit">Submit</button>
            </form>
        </body>
        </html>
        """
    
    if auth_state["needs_code"]:
        return f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>FastTelethon - Verification Code</title>
            <style>
                body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
                input[type="text"] {{ padding: 10px; font-size: 16px; width: 200px; text-align: center; letter-spacing: 5px; }}
                button {{ padding: 10px 20px; font-size: 16px; cursor: pointer; background: #0088cc; color: white; border: none; border-radius: 5px; }}
                button:hover {{ background: #006699; }}
                .info {{ background: #e7f3ff; padding: 15px; border-radius: 5px; margin: 20px 0; }}
                .phone {{ font-weight: bold; color: #0088cc; }}
            </style>
        </head>
        <body>
            <h1>📱 Enter Verification Code</h1>
            <div class="info">
                <p>A verification code has been sent to:</p>
                <p class="phone">{PHONE}</p>
                <p>Check your Telegram app and enter the code below.</p>
            </div>
            <form action="/auth/code" method="post">
                <input type="text" name="code" placeholder="12345" required autofocus maxlength="5" pattern="[0-9]*" />
                <br><br>
                <button type="submit">Verify</button>
            </form>
            <br>
            <p><small>Don't see the code? Check "Telegram" in your app for a message from Telegram.</small></p>
        </body>
        </html>
        """
    
    return """
    <!DOCTYPE html>
    <html>
    <head>
        <title>FastTelethon - Initializing</title>
        <meta http-equiv="refresh" content="3">
        <style>
            body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; text-align: center; }
        </style>
    </head>
    <body>
        <h1>⏳ Initializing...</h1>
        <p>Please wait while the service starts up.</p>
    </body>
    </html>
    """


@app.post("/auth/code")
async def submit_code(code: str = Form(...)):
    """Submit verification code"""
    global auth_state
    
    if not client or not auth_state["needs_code"]:
        raise HTTPException(status_code=400, detail="No code expected")
    
    try:
        await client.sign_in(PHONE, code, phone_code_hash=auth_state["phone_code_hash"])
        auth_state["is_authorized"] = True
        auth_state["needs_code"] = False
        auth_state["phone_code_hash"] = None
        logger.info("✅ Successfully authorized with verification code!")
        
        return HTMLResponse("""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Success</title>
            <meta http-equiv="refresh" content="2; url=/health">
            <style>
                body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; text-align: center; }
                .success { color: green; font-size: 24px; }
            </style>
        </head>
        <body>
            <h1 class="success">✅ Authorization Successful!</h1>
            <p>Redirecting to health check...</p>
        </body>
        </html>
        """)
        
    except SessionPasswordNeededError:
        auth_state["needs_code"] = False
        auth_state["needs_password"] = True
        logger.info("2FA password required")
        
        return HTMLResponse("""
        <!DOCTYPE html>
        <html>
        <head>
            <title>2FA Required</title>
            <meta http-equiv="refresh" content="2; url=/auth">
            <style>
                body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; text-align: center; }
            </style>
        </head>
        <body>
            <h1>🔐 2FA Password Required</h1>
            <p>Redirecting to password page...</p>
        </body>
        </html>
        """)
        
    except PhoneCodeInvalidError:
        auth_state["error"] = "Invalid verification code. Please try again."
        logger.error("Invalid verification code")
        
        return HTMLResponse("""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Invalid Code</title>
            <meta http-equiv="refresh" content="2; url=/auth">
            <style>
                body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; text-align: center; }
                .error { color: red; }
            </style>
        </head>
        <body>
            <h1 class="error">❌ Invalid Code</h1>
            <p>The verification code you entered is incorrect.</p>
            <p>Redirecting back...</p>
        </body>
        </html>
        """)
        
    except Exception as e:
        logger.error(f"Authorization error: {e}")
        auth_state["error"] = str(e)
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/auth/password")
async def submit_password(password: str = Form(...)):
    """Submit 2FA password"""
    global auth_state
    
    if not client or not auth_state["needs_password"]:
        raise HTTPException(status_code=400, detail="No password expected")
    
    try:
        await client.sign_in(password=password)
        auth_state["is_authorized"] = True
        auth_state["needs_password"] = False
        logger.info("✅ Successfully authorized with 2FA password!")
        
        return HTMLResponse("""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Success</title>
            <meta http-equiv="refresh" content="2; url=/health">
            <style>
                body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; text-align: center; }
                .success { color: green; font-size: 24px; }
            </style>
        </head>
        <body>
            <h1 class="success">✅ Authorization Successful!</h1>
            <p>Redirecting to health check...</p>
        </body>
        </html>
        """)
        
    except Exception as e:
        logger.error(f"2FA password error: {e}")
        auth_state["error"] = f"Invalid password: {str(e)}"
        
        return HTMLResponse(f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Invalid Password</title>
            <meta http-equiv="refresh" content="3; url=/auth">
            <style>
                body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; text-align: center; }}
                .error {{ color: red; }}
            </style>
        </head>
        <body>
            <h1 class="error">❌ Invalid Password</h1>
            <p>{str(e)}</p>
            <p>Redirecting back...</p>
        </body>
        </html>
        """)


@app.get("/session/backup")
async def backup_session():
    """
    Download session file as base64 (for free tier persistence)
    After authorization, call this endpoint to get your session file encoded.
    Save the output as TELETHON_SESSION_BASE64 environment variable in Render.
    """
    if not auth_state["is_authorized"]:
        raise HTTPException(
            status_code=401, 
            detail="Not authorized. Complete authorization first via /auth"
        )
    
    import base64
    session_file = f"{SESSION_NAME}.session"
    
    if not os.path.exists(session_file):
        raise HTTPException(status_code=404, detail="Session file not found")
    
    try:
        with open(session_file, 'rb') as f:
            session_data = f.read()
        
        encoded = base64.b64encode(session_data).decode('utf-8')
        
        return {
            "success": True,
            "message": "Copy this value and add it as TELETHON_SESSION_BASE64 environment variable in Render",
            "session_base64": encoded,
            "instructions": [
                "1. Copy the 'session_base64' value above",
                "2. Go to Render Dashboard → Your Service → Environment",
                "3. Add new environment variable:",
                "   Key: TELETHON_SESSION_BASE64",
                "   Value: <paste the session_base64 value>",
                "4. Save and redeploy",
                "5. Session will be restored automatically on every startup!"
            ]
        }
    except Exception as e:
        logger.error(f"Failed to backup session: {e}")
        raise HTTPException(status_code=500, detail=f"Failed to backup session: {str(e)}")


@app.post("/upload")
async def upload_large_file(
    file: UploadFile = File(...),
    channel_id: Optional[str] = None
):
    """
    Upload a large file to Telegram via MTProto (no size limit)
    Returns message_id for later retrieval
    """
    if not auth_state["is_authorized"]:
        raise HTTPException(status_code=503, detail="Telegram client not authorized. Visit /auth to authorize.")
    
    if not client or not await client.is_user_authorized():
        raise HTTPException(status_code=503, detail="Telegram client not ready")
    
    target_channel = channel_id or CHANNEL_ID
    if not target_channel:
        raise HTTPException(status_code=400, detail="Channel ID required")
    
    temp_path = None  # Initialize to avoid UnboundLocalError
    
    try:
        # Save uploaded file temporarily
        temp_path = f"/tmp/{file.filename}"
        with open(temp_path, "wb") as f:
            content = await file.read()
            f.write(content)
        
        logger.info(f"Uploading {file.filename} ({len(content)} bytes) to Telegram...")
        
        # Progress callback
        upload_progress = {"current": 0, "total": len(content)}
        
        async def progress_callback(current, total):
            upload_progress["current"] = current
            upload_progress["total"] = total
            if current % (5 * 1024 * 1024) == 0:  # Log every 5MB
                logger.info(f"Upload progress: {current}/{total} ({current*100//total}%)")
        
        # Upload using FastTelethon
        with open(temp_path, "rb") as f:
            result = await upload_file(client, f, progress_callback=progress_callback)
            
            # Get attributes and MIME type
            attributes, mime_type = utils.get_attributes(temp_path)
            
            # Create media object
            media = types.InputMediaUploadedDocument(
                file=result,
                mime_type=mime_type,
                attributes=attributes,
                force_file=True
            )
            
            # Send to channel
            message = await client.send_file(
                target_channel,
                file=media,
                caption=f"📁 {file.filename}"
            )
        
        # Cleanup
        os.remove(temp_path)
        
        logger.info(f"Successfully uploaded {file.filename}, message_id: {message.id}")
        
        return {
            "success": True,
            "message_id": message.id,
            "file_name": file.filename,
            "file_size": len(content),
            "channel_id": target_channel
        }
        
    except Exception as e:
        logger.error(f"Upload failed: {e}", exc_info=True)
        if temp_path and os.path.exists(temp_path):
            os.remove(temp_path)
        raise HTTPException(status_code=500, detail=f"Upload failed: {str(e)}")


@app.get("/download/{channel_id}/{message_id}")
async def download_large_file(channel_id: str, message_id: int):
    """
    Download a large file from Telegram via MTProto (no size limit)
    Returns the file as a streaming response
    """
    if not auth_state["is_authorized"]:
        raise HTTPException(status_code=503, detail="Telegram client not authorized. Visit /auth to authorize.")
    
    if not client or not await client.is_user_authorized():
        raise HTTPException(status_code=503, detail="Telegram client not ready")
    
    temp_path = None  # Initialize to avoid UnboundLocalError
    
    try:
        logger.info(f"Fetching message {message_id} from channel {channel_id}...")
        
        # Convert channel_id string to integer and create PeerChannel
        # This helps Telethon resolve the entity correctly
        channel_id_int = int(channel_id)
        peer = PeerChannel(utils.resolve_id(channel_id_int)[0])
        
        # Get the message
        message = await client.get_messages(peer, ids=message_id)
        
        if not message or not message.document:
            raise HTTPException(status_code=404, detail="File not found")
        
        # Get file info
        file_name = next(
            (attr.file_name for attr in message.document.attributes 
             if isinstance(attr, types.DocumentAttributeFilename)),
            f"file_{message_id}"
        )
        file_size = message.document.size
        mime_type = message.document.mime_type or "application/octet-stream"
        
        logger.info(f"Downloading {file_name} ({file_size} bytes)...")
        
        # Create a temporary file for download
        temp_path = f"/tmp/download_{message_id}_{file_name}"
        
        # Progress callback
        async def progress_callback(current, total):
            if current % (5 * 1024 * 1024) == 0:  # Log every 5MB
                logger.info(f"Download progress: {current}/{total} ({current*100//total}%)")
        
        # Download using FastTelethon
        with open(temp_path, "wb") as f:
            await download_file(client, message.document, f, progress_callback=progress_callback)
        
        logger.info(f"Download complete: {file_name}")
        
        # Stream the file back
        def file_iterator():
            with open(temp_path, "rb") as f:
                chunk_size = 1024 * 1024  # 1MB chunks
                while chunk := f.read(chunk_size):
                    yield chunk
            # Cleanup after streaming
            os.remove(temp_path)
        
        return StreamingResponse(
            file_iterator(),
            media_type=mime_type,
            headers={
                "Content-Disposition": f'attachment; filename="{file_name}"',
                "Content-Length": str(file_size)
            }
        )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Download failed: {e}", exc_info=True)
        if temp_path and os.path.exists(temp_path):
            os.remove(temp_path)
        raise HTTPException(status_code=500, detail=f"Download failed: {str(e)}")


@app.get("/file-info/{channel_id}/{message_id}")
async def get_file_info(channel_id: str, message_id: int):
    """Get information about a file without downloading it"""
    if not client or not await client.is_user_authorized():
        raise HTTPException(status_code=503, detail="Telegram client not ready")
    
    try:
        # Convert channel_id string to integer and create PeerChannel
        channel_id_int = int(channel_id)
        peer = PeerChannel(utils.resolve_id(channel_id_int)[0])
        
        message = await client.get_messages(peer, ids=message_id)
        
        if not message or not message.document:
            raise HTTPException(status_code=404, detail="File not found")
        
        file_name = next(
            (attr.file_name for attr in message.document.attributes 
             if isinstance(attr, types.DocumentAttributeFilename)),
            f"file_{message_id}"
        )
        
        return {
            "message_id": message_id,
            "file_name": file_name,
            "file_size": message.document.size,
            "mime_type": message.document.mime_type,
            "date": message.date.isoformat()
        }
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Failed to get file info: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=f"Failed to get file info: {str(e)}")


if __name__ == "__main__":
    uvicorn.run(
        "main:app",
        host="0.0.0.0",
        port=8001,
        reload=False,
        log_level="info"
    )
