# FastTelethon Service Dockerfile
FROM python:3.11-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    gcc \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy requirements and install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY main.py .
COPY FastTelethon.py .
COPY docker-entrypoint-enhanced.sh /docker-entrypoint.sh

# Make entrypoint executable
RUN chmod +x /docker-entrypoint.sh

# Create sessions directory
RUN mkdir -p /app/sessions && chmod 700 /app/sessions

# Expose port
EXPOSE 8001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
  CMD python -c "import requests; requests.get('http://localhost:8001/health')" || exit 1

# Use enhanced entrypoint with session backup
ENTRYPOINT ["/docker-entrypoint.sh"]
