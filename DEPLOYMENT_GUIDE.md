# MR.DarkPromth Production Deployment Guide

This guide details the steps to deploy the fully verified MR.DarkPromth system to a production environment.

## 1. Prerequisites
- **Server**: Linux (Ubuntu 22.04+ recommended) with public IP
- **Domain**: `api.mrdarkpromth.online` pointing to server IP
- **Docker**: Engine 24+ & Compose v2 installed
- **SSL Certificates**: Let's Encrypt certs in `/etc/letsencrypt/live/api.mrdarkpromth.online/`

## 2. Environment Configuration
Create/Update `.env` in the project root:

```ini
# Database
POSTGRES_USER=dark_admin
POSTGRES_PASSWORD=YOUR_STRONG_DB_PASSWORD
POSTGRES_DB=mrdarkpromth

# Redis
REDIS_PASSWORD=YOUR_STRONG_REDIS_PASSWORD

# API Keys
OPENROUTER_API_KEYS=sk-or-...,sk-or-...
CEREBRAS_API_KEYS=csk-...,csk-...

# JWT & Security
JWT_SECRET=YOUR_VERY_LONG_RANDOM_STRING
ADMIN_API_KEY=YOUR_ADMIN_KEY

# Telegram Alerts
TELEGRAM_BOT_TOKEN=123456789:ABC...
TELEGRAM_CHAT_ID=-100...

# Github OAuth
GITHUB_CLIENT_ID=...
GITHUB_CLIENT_SECRET=...
GITHUB_REDIRECT_URI=https://api.mrdarkpromth.online/auth/github/callback
```

## 3. Deployment Steps

### Step 1: Initial Build & Start
```bash
# Build and start services
docker-compose -f docker-compose.production.yml up -d --build
```

### Step 2: Database Migration
```bash
# Run migrations using the API container's sqlx-cli
docker-compose -f docker-compose.production.yml exec api sqlx migrate run
```

### Step 3: Verify Services
```bash
# Check container status
docker ps

# Check logs for startup errors
docker logs -f mr_darkpromth_api
```

## 4. Verification

### Automated Health Check
Run the included verification script:
```bash
./scripts/final_verification.sh
```

### Manual Checks
1. **API Health**: `curl https://api.mrdarkpromth.online/health` → `{"status":"healthy",...}`
2. **Metrics**: `curl https://api.mrdarkpromth.online/metrics` (should be protected or return metrics)
3. **Grafana**: Access `https://monitor.mrdarkpromth.online` (if configured) or tunnel port 3000.

## 5. Maintenance

### Monitoring
- **Prometheus**: Scrapes API at `:8080/metrics`.
- **Grafana**: Dashboards for Request Rate, Latency, and Error Rate.
- **Telegram**: Alerts on high error rates or critical failures.

### Logs
- **API Logs**: `docker logs mr_darkpromth_api`
- **Nginx Logs**: `docker logs mr_darkpromth_nginx`

### Updates
To deploy code changes:
```bash
git pull
docker-compose -f docker-compose.production.yml up -d --build api
```
