# Deploy to Render.com (Free Tier)

## � Quick Setup

**Note:** Render free tier does NOT support persistent disks or docker-compose. We use environment variables to persist sessions.

---

## Step 1: Get Telegram Session (ONE TIME)

### Option A: Via Web UI (Easiest - No Local Setup)

1. Deploy to Render first (Step 2)
2. Visit `https://your-fasttelethon-service.onrender.com/auth`
3. Enter verification code sent to your phone
4. Once authorized, visit: `https://your-fasttelethon-service.onrender.com/session/backup`
5. Copy the `session_base64` value
6. Add it to Render environment variables (Step 3)

### Option B: Local Authorization (Alternative)

```bash
# Clone repo locally
git clone <your-repo>
cd FilesLink

# Setup environment
cp .env.example .env
# Edit .env: Add TELEGRAM_API_ID, TELEGRAM_API_HASH, TELEGRAM_PHONE

# Install Python dependencies
cd python_service
pip install -r requirements.txt

# Authorize and get session
python authorize.py

# Encode session to base64
# Windows PowerShell:
[Convert]::ToBase64String([IO.File]::ReadAllBytes("fasttelethon_session.session"))

# Linux/Mac:
base64 -w 0 fasttelethon_session.session

# Copy the output - you'll need it in Step 3
```

---

## Step 2: Deploy to Render

1. **Push to GitHub:**
   ```bash
   git push origin main
   ```

2. **Create Blueprint:**
   - Go to [Render Dashboard](https://dashboard.render.com)
   - Click **"New +"** → **"Blueprint"**
   - Connect your GitHub repo
   - Select `render.yaml`
   - Click **"Apply"**

3. **Set Environment Variables:**

   For **`fileslink`** service:
   - `BOT_TOKEN` = Your Telegram bot token
   - `STORAGE_CHANNEL_ID` = Your storage channel ID (e.g., `-1001234567890`)
   - `BASE_URL` = `https://your-service-name.onrender.com`

   For **`fasttelethon`** service:
   - `TELEGRAM_API_ID` = From my.telegram.org
   - `TELEGRAM_API_HASH` = From my.telegram.org
   - `TELEGRAM_PHONE` = Your phone with country code (e.g., `+1234567890`)
   - `TELEGRAM_CHANNEL_ID` = Same as `STORAGE_CHANNEL_ID`
   - `TELETHON_SESSION_BASE64` = *(Leave empty for now if using Option A)*

---

## Step 3: Save Session (After First Deploy)

If you used **Option A** (Web UI):

1. Visit `https://your-fasttelethon-service.onrender.com/session/backup`
2. Copy the `session_base64` value
3. Go to Render Dashboard → `fasttelethon` service → Environment
4. Add/Update: `TELETHON_SESSION_BASE64` = `<paste the value>`
5. Save → Service will redeploy automatically

**Now your session persists across all future deploys!** ✅

---

## How It Works

- On startup, FastTelethon checks for `TELETHON_SESSION_BASE64` env var
- If found, decodes and restores session automatically
- No re-authorization needed on redeploys
- Works on free tier (no persistent disk required)

---

## Important Notes

- **Free Tier Limits:** 750 hours/month (1 service = 24/7), services sleep after 15min inactivity
- **Service URLs:** Render provides `https://<service-name>.onrender.com`
- **Internal Communication:** Services use `http://fasttelethon:8001` (not localhost)
- **First Deploy:** May take 5-10 minutes to build

---

## Troubleshooting

**Service won't authorize:**
- Check logs: Render Dashboard → Service → Logs
- Verify phone number format: `+1234567890` (with country code)
- Ensure API_ID and API_HASH are correct

**"Not authorized" after redeploy:**
- Make sure `TELETHON_SESSION_BASE64` env var is set
- Re-authorize via `/auth` and get new session from `/session/backup`

**Services can't communicate:**
- Check `FASTTELETHON_URL` in fileslink service = `http://fasttelethon:8001`
- Use service names, not URLs, for internal communication
