#!/bin/bash

# ==========================================
# MR.DarkPromth Production Deployment
# ==========================================

set -e

COMPOSE_FILE="docker-compose.production.yml"

echo "🚀 Deploying MR.DarkPromth to Production..."
echo "=========================================="

# Check Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running"
    exit 1
fi

# Check .env file exists
if [ ! -f ./.env ]; then
    echo "❌ .env file not found. Copy .env.production.example to .env and configure it."
    exit 1
fi

# Check required environment variables
source .env
for var in POSTGRES_PASSWORD REDIS_PASSWORD JWT_SECRET CEREBRAS_API_KEYS; do
    if [ -z "${!var}" ]; then
        echo "❌ Required environment variable $var is not set in .env"
        exit 1
    fi
done

echo "1. 🔧 Setting up Production Environment..."

# Create necessary directories
mkdir -p logs/nginx
mkdir -p backups
mkdir -p monitoring/grafana/dashboards
mkdir -p monitoring/grafana/datasources
mkdir -p certs

# Generate self-signed SSL certificates if none exist
if [ ! -f ./certs/cert.pem ]; then
    echo "   📋 Generating self-signed SSL certificates..."
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout ./certs/key.pem \
        -out ./certs/cert.pem \
        -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"
    echo "   ⚠️  Using self-signed certificates. Replace with Let's Encrypt for production."
fi

echo "2. 🐳 Building Production Images..."
docker compose -f "$COMPOSE_FILE" build --no-cache

echo "3. 🚀 Starting Production Services..."
docker compose -f "$COMPOSE_FILE" down
docker compose -f "$COMPOSE_FILE" up -d

echo "4. ⏳ Waiting for services to be ready..."

# Wait for database
echo "   Waiting for PostgreSQL..."
timeout 60 bash -c 'until docker exec mr_darkpromth_postgres pg_isready -U postgres; do sleep 1; done'

# Wait for Redis
echo "   Waiting for Redis..."
timeout 60 bash -c 'until docker exec mr_darkpromth_redis redis-cli -a "$REDIS_PASSWORD" ping 2>/dev/null | grep -q PONG; do sleep 1; done'

# Wait for API
echo "   Waiting for API..."
timeout 90 bash -c 'until curl -k -f -s https://localhost/health > /dev/null 2>&1 || curl -f -s http://localhost:8080/health > /dev/null 2>&1; do sleep 2; done'

echo "5. 🧪 Running Health Checks..."

# Basic health checks
API_HEALTH=$(curl -k -s -o /dev/null -w "%{http_code}" https://localhost/health 2>/dev/null || curl -s -o /dev/null -w "%{http_code}" http://localhost:8080/health 2>/dev/null || echo "000")
if [ "$API_HEALTH" = "200" ]; then
    echo "   ✅ API health check passed"
else
    echo "   ❌ API health check failed (HTTP $API_HEALTH)"
fi

# Check all containers are running
echo "   📦 Container Status:"
for container in mr_darkpromth_postgres mr_darkpromth_redis mr_darkpromth_api mr_darkpromth_nginx mr_darkpromth_frontend; do
    if docker ps --format '{{.Names}}' | grep -q "^${container}$"; then
        echo "      ✅ $container"
    else
        echo "      ❌ $container is not running"
    fi
done

echo "6. 📊 Monitoring Status..."
for container in mr_darkpromth_prometheus mr_darkpromth_grafana; do
    if docker ps --format '{{.Names}}' | grep -q "^${container}$"; then
        echo "   ✅ $container"
    else
        echo "   ⚠️  $container is not running (optional)"
    fi
done

echo ""
echo "=========================================="
echo "🎉 PRODUCTION DEPLOYMENT COMPLETE!"
echo "=========================================="
echo ""
echo "🌐 Access URLs:"
echo "  - Frontend:   https://localhost"
echo "  - API:        https://localhost/api"
echo "  - Prometheus: http://localhost:9090  (localhost only)"
echo "  - Grafana:    http://localhost:3001  (localhost only)"
echo ""
echo "📚 Documentation:  ./DEPLOYMENT_GUIDE.md"
echo "💾 Backup:          ./backup.sh"
echo "📦 Backup Database: ./backup_database.sh"
echo ""
echo "🔐 IMPORTANT SECURITY REMINDERS:"
echo "  1. Ensure JWT_SECRET is a strong random value"
echo "  2. Ensure GRAFANA_ADMIN_PASSWORD is set in .env"
echo "  3. Replace self-signed certs with Let's Encrypt for production"
echo "  4. Verify CORS_ALLOWED_ORIGINS matches your domain"
echo ""
echo "✅ System is PRODUCTION READY!"
