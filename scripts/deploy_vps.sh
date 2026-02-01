#!/bin/bash
# VPS Production Deployment Script for MR.Darkpromth
# Run on VPS: root@150-95-31-224

set -e

echo "=== MR.Darkpromth Production Deployment ==="

# Update system
echo "[1/8] Updating system packages..."
apt update && apt upgrade -y

# Install Docker
echo "[2/8] Installing Docker..."
if ! command -v docker &> /dev/null; then
    curl -fsSL https://get.docker.com -o get-docker.sh
    sh get-docker.sh
    usermod -aG docker root
    systemctl enable docker
    systemctl start docker
fi

# Install Docker Compose
echo "[3/8] Installing Docker Compose..."
if ! command -v docker-compose &> /dev/null; then
    curl -L "https://github.com/docker/compose/releases/download/v2.23.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
    chmod +x /usr/local/bin/docker-compose
fi

# Create app directory
echo "[4/8] Setting up application directory..."
mkdir -p /opt/mrdarkpromth
cd /opt/mrdarkpromth

# Clone repository
echo "[5/8] Cloning repository..."
if [ -d ".git" ]; then
    git pull origin main
else
    git clone https://github.com/Mr-darkpromth/MR.Darkpromth.git .
fi

# Create production .env
echo "[6/8] Creating production environment..."
cat > .env << 'ENVFILE'
# Production Environment - bt-shop-dark.online
CEREBRAS_API_KEYS=${CEREBRAS_API_KEYS}
CEREBRAS_MODEL_DEFAULT=llama-3.3-70b
OPENROUTER_API_KEYS=${OPENROUTER_API_KEYS}

SERVER_HOST=0.0.0.0
SERVER_PORT=8080
CORS_ALLOWED_ORIGINS=https://bt-shop-dark.online,https://www.bt-shop-dark.online

JWT_SECRET=${JWT_SECRET}
JWT_EXPIRES_IN=24h

DATABASE_URL=postgres://postgres:postgres@postgres:5432/mr_darkpromth
REDIS_URL=redis://redis:6379

RUST_LOG=info,mr_darkpromth_api=debug

GITHUB_CLIENT_ID=${GITHUB_CLIENT_ID}
GITHUB_CLIENT_SECRET=${GITHUB_CLIENT_SECRET}
GITHUB_REDIRECT_URI=https://bt-shop-dark.online/login/github

SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=${SMTP_USER}
SMTP_PASSWORD=${SMTP_PASSWORD}
SMTP_FROM=noreply@bt-shop-dark.online
SMTP_TLS=true

SSL_CERT_PATH=/etc/nginx/certs/cert.pem
SSL_KEY_PATH=/etc/nginx/certs/key.pem

ENVIRONMENT=production
DEBUG=false
LOG_LEVEL=info
ENVFILE

echo "[7/8] Generating SSL certificates with Let's Encrypt..."
apt install -y certbot

# Stop any running nginx first
systemctl stop nginx 2>/dev/null || true

# Generate certificate
certbot certonly --standalone -d bt-shop-dark.online -d www.bt-shop-dark.online --agree-tos --non-interactive --email mrdarkpromth2@gmail.com

# Copy certificates to app directory
mkdir -p /opt/mrdarkpromth/certs
cp /etc/letsencrypt/live/bt-shop-dark.online/fullchain.pem /opt/mrdarkpromth/certs/cert.pem
cp /etc/letsencrypt/live/bt-shop-dark.online/privkey.pem /opt/mrdarkpromth/certs/key.pem

# Auto-renewal cron job
echo "0 3 * * * certbot renew --quiet && cp /etc/letsencrypt/live/bt-shop-dark.online/fullchain.pem /opt/mrdarkpromth/certs/cert.pem && cp /etc/letsencrypt/live/bt-shop-dark.online/privkey.pem /opt/mrdarkpromth/certs/key.pem && cd /opt/mrdarkpromth && docker-compose restart nginx" | crontab -

echo "[8/8] Starting services..."
docker-compose down 2>/dev/null || true
docker-compose up -d

echo ""
echo "=== Deployment Complete ==="
echo "Website: https://bt-shop-dark.online"
echo "Grafana: https://bt-shop-dark.online/grafana"
echo "Prometheus: https://bt-shop-dark.online/prometheus"
echo ""
echo "Check status: docker-compose ps"
echo "View logs: docker-compose logs -f api"
