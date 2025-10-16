# Deployment Guide

Deploy FilesLink to cloud platforms for free or low-cost hosting.

## Quick Comparison

| Platform | Free Tier | Setup Time | Keep-Alive Needed | Best For |
|----------|-----------|------------|-------------------|----------|
| **Railway** | 500 hrs/mo | 5 min | No | Recommended ⭐ |
| **Render** | Yes (sleeps) | 5 min | Yes | Free tier + keep-alive |
| **Fly.io** | 3 VMs free | 10 min | No | Global edge |
| **VPS** | Varies | 15 min | No | Full control |

---

## Render (Free with Keep-Alive) (Recommended) ⭐

Render offers a generous free tier, but instances sleep after 15 minutes of inactivity. **Use Cloudflare Workers to prevent sleep!**

### Why Render + Keep-Alive?

- ✅ Completely free (without persistent disk)
- ✅ No sleep with keep-alive solution
- ✅ Automatic HTTPS
- ✅ Git-based deployment
- ⚠️ Ephemeral storage (maybe settings,links lost on restart)

### Deploy to Render

1. **Fork the Repository**
   ```bash
   # Fork https://github.com/bytetrix/fileslink
   ```

2. **Create Web Service**
   - Go to [render.com](https://render.com)
   - Click "New +" → "Web Service"
   - Connect your GitHub repository
   - Select your forked FilesLink repo

3. **Configure Build**
   
   Render auto-detects Docker. Use these settings:
   
   - **Name**: `fileslink`
   - **Region**: Choose closest to you
   - **Branch**: `main`
   - **Runtime**: `Docker`
   - **Instance Type**: `Free`

4. **Add Environment Variables**
   
   ```bash
   BOT_TOKEN=your_telegram_bot_token
   STORAGE_CHANNEL_ID=-1001234567890
   APP_FILE_DOMAIN=https://yourapp.onrender.com/files
   TELEGRAM_API_URL=https://api.telegram.org
   TELEGRAM_LOCAL=false
   SERVER_PORT=8080
   RUST_LOG=warn
   ```

5. **Deploy**
   
   Click "Create Web Service". First build takes ~5-10 minutes.

### 🔥 Prevent Sleep with Cloudflare Workers

**Problem**: Render free tier sleeps after 15 minutes of inactivity, causing 30-50 second cold starts.

**Solution**: Use [**Cloudflare Render Ping**](https://github.com/ByteTrix/cloudflare-render-ping) to keep your instance alive 24/7!

#### Quick Setup

1. **Deploy Keep-Alive Worker**
   
   Click here to deploy instantly:
   
   [![Deploy to Cloudflare Workers](https://deploy.workers.cloudflare.com/button)](https://deploy.workers.cloudflare.com/?url=https://github.com/ByteTrix/cloudflare-render-ping)
   
   Or follow the [setup guide](https://github.com/ByteTrix/cloudflare-render-ping#readme).

2. **Configure Your Render URL**
   
   Add your Render app URL to the worker:
   ```javascript
   const RENDER_SERVICE_URL = "https://yourapp.onrender.com";
   ```

3. **Done!**
   
   The worker pings your app every 5 minutes, preventing sleep. Completely free!

#### How It Works

- Cloudflare Worker runs on their edge network (100% free, 100k requests/day)
- Pings your Render service every 5 minutes via HTTP health check
- Keeps instance warm 24/7
- No cold starts for your users!

**Repository**: [ByteTrix/cloudflare-render-ping](https://github.com/ByteTrix/cloudflare-render-ping)

### 🌐 Custom Domain Setup on Render

Want to use your own domain (e.g., `files.yourdomain.com`) instead of `yourapp.onrender.com`? Here's how:

#### Step 1: Add Custom Domain in Render

1. **Open Your Service**
   - Go to your Render dashboard
   - Select your FilesLink service

2. **Navigate to Settings**
   - Click on the "Settings" tab
   - Scroll down to "Custom Domains" section

3. **Add Your Domain**
   - Click "Add Custom Domain"
   - Enter your domain: `files.yourdomain.com` (or `yourdomain.com`)
   - Click "Save"

4. **Get DNS Records**
   
   Render will show you the DNS records to add. You'll see something like:
   
   **For subdomain (files.yourdomain.com):**
   ```
   Type: CNAME
   Name: files (or files.yourdomain.com)
   Value: yourapp.onrender.com
   ```
   
   **For root domain (yourdomain.com):**
   ```
   Type: A
   Name: @ (or yourdomain.com)
   Value: 216.24.57.1
   ```
   
   Keep this page open - you'll need these values!

#### Step 2: Configure DNS at Your Domain Provider

The steps vary by provider, but here are instructions for popular ones:

##### **Cloudflare DNS** (Recommended)

1. **Login to Cloudflare**
   - Go to [dash.cloudflare.com](https://dash.cloudflare.com)
   - Select your domain

2. **Add DNS Record**
   - Click "DNS" → "Records"
   - Click "Add record"
   - Fill in:
     - **Type**: `CNAME`
     - **Name**: `files` (for files.yourdomain.com)
     - **Target**: `yourapp.onrender.com`
     - **Proxy status**: ⚠️ DNS only (gray cloud, not orange)
     - **TTL**: Auto
   - Click "Save"

3. **Important: Disable Cloudflare Proxy**
   - The cloud icon must be **gray** (DNS only)
   - Orange cloud (proxied) will NOT work with Render!

##### **Namecheap**

1. **Login to Namecheap**
   - Go to [namecheap.com](https://namecheap.com)
   - Go to Domain List → Manage

2. **Add CNAME Record**
   - Go to "Advanced DNS" tab
   - Click "Add New Record"
   - Fill in:
     - **Type**: `CNAME Record`
     - **Host**: `files`
     - **Value**: `yourapp.onrender.com`
     - **TTL**: Automatic
   - Click the checkmark to save

##### **GoDaddy**

1. **Login to GoDaddy**
   - Go to [godaddy.com](https://godaddy.com)
   - My Products → DNS

2. **Add CNAME Record**
   - Click "Add" under DNS Records
   - Fill in:
     - **Type**: `CNAME`
     - **Name**: `files`
     - **Value**: `yourapp.onrender.com`
     - **TTL**: 600 seconds
   - Click "Save"

##### **Google Domains (now Squarespace)**

1. **Login to Your Registrar**
   - Go to your domain management

2. **Add CNAME Record**
   - Go to DNS settings
   - Click "Manage custom records"
   - Add:
     - **Host name**: `files`
     - **Type**: `CNAME`
     - **Data**: `yourapp.onrender.com`
   - Click "Save"

#### Step 3: Update Environment Variables

Once DNS is configured, update your `APP_FILE_DOMAIN`:

1. **Go to Render Dashboard**
   - Select your service
   - Click "Environment"

2. **Update APP_FILE_DOMAIN**
   
   Change from:
   ```bash
   APP_FILE_DOMAIN=https://yourapp.onrender.com/files
   ```
   
   To your custom domain:
   ```bash
   APP_FILE_DOMAIN=https://files.yourdomain.com/files
   ```

3. **Save Changes**
   
   Render will automatically redeploy with the new domain.

#### Step 4: Wait for DNS Propagation

- DNS changes can take 5 minutes to 48 hours to propagate
- Usually takes 15-30 minutes
- Check status: [whatsmydns.net](https://whatsmydns.net)

#### Step 5: Verify HTTPS

1. **Check Certificate**
   
   Render automatically provisions SSL certificate via Let's Encrypt.
   
   Visit: `https://files.yourdomain.com`
   
   You should see a valid SSL certificate (green lock icon).

2. **Test Your Bot**
   
   Send a file to your Telegram bot and verify the download link uses your custom domain:
   ```
   https://files.yourdomain.com/files/abc12345
   ```

#### Troubleshooting Custom Domain

**Domain not working after 1 hour?**

1. **Verify DNS Records**
   ```powershell
   # Windows PowerShell
   Resolve-DnsName files.yourdomain.com
   
   # Should show: yourapp.onrender.com
   ```
   
   ```bash
   # Linux/Mac
   dig files.yourdomain.com
   
   # Should show CNAME pointing to yourapp.onrender.com
   ```

2. **Check Cloudflare Proxy**
   
   If using Cloudflare, ensure proxy is **disabled** (gray cloud):
   - Go to DNS settings
   - Click the orange cloud to turn it gray
   - Wait 5 minutes

3. **Verify in Render**
   
   In Render dashboard → Settings → Custom Domains:
   - Status should show "Verified" with green checkmark
   - SSL status should show "Active"

4. **Check APP_FILE_DOMAIN**
   
   Ensure environment variable is correct:
   - Must use HTTPS
   - Must end with `/files`
   - Example: `https://files.yourdomain.com/files`

5. **Redeploy if Needed**
   
   If domain is verified but not working:
   - Go to "Manual Deploy"
   - Click "Deploy latest commit"

**SSL Certificate Issues?**

- Render automatically provisions SSL via Let's Encrypt
- Can take up to 15 minutes after DNS verification
- If stuck, try removing and re-adding the custom domain

**Getting "Service Unavailable"?**

- Check if your service is running (not sleeping)
- Deploy the [Cloudflare keep-alive worker](https://github.com/ByteTrix/cloudflare-render-ping)
- Update the worker with your custom domain

#### Update Keep-Alive Worker

If you're using the Cloudflare keep-alive solution, update it with your custom domain:

```javascript
// In your Cloudflare Worker
const RENDER_SERVICE_URL = "https://files.yourdomain.com"; // Update this
```

This ensures the worker pings your custom domain to keep the service alive.

---


## Railway

Railway is the easiest and most reliable option for FilesLink.

### Why Railway?

- ✅ No sleep on free tier (500 hours/month)
- ✅ Automatic HTTPS
- ✅ One-click deployment from GitHub
- ✅ Environment variables in UI
- ✅ Free PostgreSQL if needed

### Deploy to Railway

1. **Fork the Repository**
   ```bash
   # Fork https://github.com/bytetrix/fileslink to your account
   ```

2. **Create New Project**
   - Go to [railway.app](https://railway.app)
   - Click "New Project"
   - Select "Deploy from GitHub repo"
   - Choose your forked repository

3. **Configure Environment Variables**
   
   In Railway dashboard, add these variables:
   
   ```bash
   BOT_TOKEN=your_telegram_bot_token
   STORAGE_CHANNEL_ID=-1001234567890
   APP_FILE_DOMAIN=https://yourapp.up.railway.app/files
   TELEGRAM_API_URL=https://api.telegram.org
   TELEGRAM_LOCAL=false
   SERVER_PORT=8080
   RUST_LOG=warn
   ```

4. **Get Your Domain**
   
   Railway automatically assigns a domain like:
   ```
   https://yourapp.up.railway.app
   ```
   
   Copy this and update `APP_FILE_DOMAIN` to:
   ```
   https://yourapp.up.railway.app/files
   ```

5. **Deploy**
   
   Railway automatically builds and deploys! Monitor logs in the dashboard.

### Custom Domain (Optional)

1. Go to project settings
2. Click "Generate Domain" or "Custom Domain"
3. Update `APP_FILE_DOMAIN` with your new domain

---

## Fly.io (Global Edge)

Best for low-latency global deployment.

### Why Fly.io?

- ✅ 3 VMs free (256MB RAM each)
- ✅ Global edge deployment
- ✅ Automatic HTTPS
- ✅ No sleep issues

### Deploy to Fly.io

1. **Install Fly CLI**
   
   ```bash
   # Windows (PowerShell)
   iwr https://fly.io/install.ps1 -useb | iex
   
   # macOS/Linux
   curl -L https://fly.io/install.sh | sh
   ```

2. **Login**
   
   ```bash
   flyctl auth login
   ```

3. **Create fly.toml**
   
   Create `fly.toml` in your project root:
   
   ```toml
   app = "your-app-name"
   primary_region = "iad"  # Change to your region
   
   [build]
   
   [http_service]
     internal_port = 8080
     force_https = true
     auto_stop_machines = false
     auto_start_machines = true
     min_machines_running = 1
   
   [[vm]]
     memory = '256mb'
     cpu_kind = 'shared'
     cpus = 1
   ```

4. **Create App**
   
   ```bash
   flyctl apps create your-app-name
   ```

5. **Set Environment Variables**
   
   ```bash
   flyctl secrets set BOT_TOKEN=your_token
   flyctl secrets set STORAGE_CHANNEL_ID=-1001234567890
   flyctl secrets set APP_FILE_DOMAIN=https://your-app-name.fly.dev/files
   flyctl secrets set TELEGRAM_API_URL=https://api.telegram.org
   flyctl secrets set TELEGRAM_LOCAL=false
   flyctl secrets set SERVER_PORT=8080
   flyctl secrets set RUST_LOG=warn
   ```

6. **Deploy**
   
   ```bash
   flyctl deploy
   ```

### Monitor

```bash
# View logs
flyctl logs

# Check status
flyctl status

# Scale regions
flyctl regions add lax syd  # Add Los Angeles, Sydney
```

---

## VPS / Self-Hosted

For complete control and unlimited resources.

### Requirements

- Ubuntu 20.04+ or Debian 11+
- Docker and Docker Compose installed
- Domain name (optional)
- SSH access

### Setup

1. **SSH into Server**
   
   ```bash
   ssh user@your-server-ip
   ```

2. **Install Docker**
   
   ```bash
   # Update system
   sudo apt update && sudo apt upgrade -y
   
   # Install Docker
   curl -fsSL https://get.docker.com -o get-docker.sh
   sudo sh get-docker.sh
   
   # Install Docker Compose
   sudo apt install docker-compose -y
   
   # Add user to docker group
   sudo usermod -aG docker $USER
   newgrp docker
   ```

3. **Clone Repository**
   
   ```bash
   git clone https://github.com/bytetrix/fileslink.git
   cd fileslink
   ```

4. **Configure Environment**
   
   ```bash
   cp .env.example .env
   nano .env
   ```
   
   Update these values:
   ```bash
   BOT_TOKEN=your_token
   STORAGE_CHANNEL_ID=-1001234567890
   APP_FILE_DOMAIN=https://yourdomain.com/files
   TELEGRAM_API_URL=https://api.telegram.org
   TELEGRAM_LOCAL=false
   SERVER_PORT=8080
   RUST_LOG=warn
   ```

5. **Start Services**
   
   ```bash
   docker compose up -d
   ```

6. **Setup Reverse Proxy (Optional)**
   
   Install nginx for HTTPS:
   
   ```bash
   sudo apt install nginx certbot python3-certbot-nginx -y
   ```
   
   Create nginx config `/etc/nginx/sites-available/fileslink`:
   
   ```nginx
   server {
       listen 80;
       server_name yourdomain.com;
       
       location / {
           proxy_pass http://localhost:8080;
           proxy_http_version 1.1;
           proxy_set_header Upgrade $http_upgrade;
           proxy_set_header Connection 'upgrade';
           proxy_set_header Host $host;
           proxy_cache_bypass $http_upgrade;
           proxy_set_header X-Real-IP $remote_addr;
           proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
           proxy_set_header X-Forwarded-Proto $scheme;
       }
   }
   ```
   
   Enable and get SSL certificate:
   
   ```bash
   sudo ln -s /etc/nginx/sites-available/fileslink /etc/nginx/sites-enabled/
   sudo nginx -t
   sudo systemctl restart nginx
   sudo certbot --nginx -d yourdomain.com
   ```

7. **Auto-Start on Boot**
   
   Create systemd service `/etc/systemd/system/fileslink.service`:
   
   ```ini
   [Unit]
   Description=FilesLink Bot
   After=docker.service
   Requires=docker.service
   
   [Service]
   Type=oneshot
   RemainAfterExit=yes
   WorkingDirectory=/home/user/fileslink
   ExecStart=/usr/bin/docker compose up -d
   ExecStop=/usr/bin/docker compose down
   
   [Install]
   WantedBy=multi-user.target
   ```
   
   Enable service:
   ```bash
   sudo systemctl daemon-reload
   sudo systemctl enable fileslink
   sudo systemctl start fileslink
   ```

### Maintenance

```bash
# View logs
docker compose logs -f

# Restart services
docker compose restart

# Update to latest version
git pull
docker compose pull
docker compose up -d

# Backup file mappings
docker compose exec fileslink-app cat /app/file_mappings.json > backup.json
```

---

## Environment Variables by Platform

### Railway
```bash
BOT_TOKEN=your_token
STORAGE_CHANNEL_ID=-1001234567890
APP_FILE_DOMAIN=https://yourapp.up.railway.app/files
TELEGRAM_API_URL=https://api.telegram.org
TELEGRAM_LOCAL=false
SERVER_PORT=8080
RUST_LOG=warn
```

### Render (+ Cloudflare Keep-Alive)
```bash
BOT_TOKEN=your_token
STORAGE_CHANNEL_ID=-1001234567890
APP_FILE_DOMAIN=https://yourapp.onrender.com/files
TELEGRAM_API_URL=https://api.telegram.org
TELEGRAM_LOCAL=false
SERVER_PORT=8080
RUST_LOG=warn
```

### Fly.io
```bash
BOT_TOKEN=your_token
STORAGE_CHANNEL_ID=-1001234567890
APP_FILE_DOMAIN=https://your-app-name.fly.dev/files
TELEGRAM_API_URL=https://api.telegram.org
TELEGRAM_LOCAL=false
SERVER_PORT=8080
RUST_LOG=warn
```

### VPS (with domain)
```bash
BOT_TOKEN=your_token
STORAGE_CHANNEL_ID=-1001234567890
APP_FILE_DOMAIN=https://yourdomain.com/files
TELEGRAM_API_URL=https://api.telegram.org
TELEGRAM_LOCAL=false
SERVER_PORT=8080
RUST_LOG=warn
```

---

## Troubleshooting

### Deployment Fails

**Check logs:**
```bash
# Railway: View in dashboard
# Render: View in dashboard
# Fly.io:
flyctl logs
# VPS:
docker compose logs
```

**Common issues:**
- Missing environment variables
- Wrong `APP_FILE_DOMAIN` format (must end with `/files`)
- Invalid channel ID format (must start with `-100`)

### Bot Not Responding

1. Verify `BOT_TOKEN` is correct
2. Check bot is admin in storage channel
3. Verify `STORAGE_CHANNEL_ID` is correct (use @userinfobot)
4. Check application logs

### Download Links Don't Work

1. Verify `APP_FILE_DOMAIN` matches your actual domain
2. Must end with `/files`
3. Use HTTPS in production
4. Check application is accessible at the domain

### Render Instance Sleeping

**Solution**: Deploy [Cloudflare Render Ping](https://github.com/ByteTrix/cloudflare-render-ping)

This keeps your Render instance awake 24/7 for free!

---

## Cost Comparison

| Platform | Monthly Cost | Storage | Bandwidth | Notes |
|----------|-------------|---------|-----------|-------|
| **Railway** | Free (500hrs) | Unlimited | Unlimited | Best for hobby projects |
| **Render** | Free | Unlimited | 100GB | Sleeps without keep-alive |
| **Fly.io** | Free (3 VMs) | 3GB | 160GB | Good for global deployment |
| **VPS** | $5-20/mo | Varies | Varies | Full control |

**Recommendation**: Start with **Railway** for simplicity, or **Render + Cloudflare Keep-Alive** for completely free hosting.

---

## Next Steps

- [Configuration Guide](CONFIGURATION.md) - Set up environment variables
- [Telegram Setup](TELEGRAM_STORAGE_SETUP.md) - Configure your storage channel
- [FAQ](FAQ.md) - Common issues and solutions
- [CLI Usage](CLI.md) - Manage permissions from command line

---

**Questions?** Open an issue on [GitHub](https://github.com/bytetrix/fileslink/issues).
