#!/bin/bash
# Production Monitoring Script for MR.DarkPromth
# Run via cron: */5 * * * * /opt/mrdarkpromth/scripts/production_monitor.sh

set -e

LOG_FILE="/var/log/mrdarkpromth/monitor.log"
ALERT_EMAIL="${ALERT_EMAIL:-admin@mrdarkpromth.ai}"
API_URL="http://localhost:8080"
SLACK_WEBHOOK="${SLACK_WEBHOOK:-}"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

alert() {
    log "ALERT: $1"
    if [ -n "$SLACK_WEBHOOK" ]; then
        curl -s -X POST "$SLACK_WEBHOOK" -H 'Content-type: application/json' \
            -d "{\"text\":\"🚨 MR.DarkPromth Alert: $1\"}" > /dev/null
    fi
}

# Create log directory
mkdir -p /var/log/mrdarkpromth

log "=== Starting Production Health Check ==="

# 1. Check Docker containers
CONTAINERS=("mr_darkpromth_api" "mr_darkpromth_postgres" "mr_darkpromth_redis" "mr_darkpromth_nginx" "mr_darkpromth_frontend")
for container in "${CONTAINERS[@]}"; do
    if ! docker ps --format "{{.Names}}" | grep -q "^${container}$"; then
        alert "Container $container is NOT running!"
        docker compose -f /opt/mrdarkpromth/docker-compose.yml up -d "$container" 2>/dev/null || true
    fi
done

# 2. Check API health endpoint
HTTP_CODE=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/health" --max-time 10 || echo "000")
if [ "$HTTP_CODE" != "200" ]; then
    alert "API health check failed! HTTP $HTTP_CODE"
else
    log "✅ API health: OK"
fi

# 3. Check database connectivity
if ! docker exec mr_darkpromth_postgres pg_isready -U postgres > /dev/null 2>&1; then
    alert "PostgreSQL is not responding!"
else
    log "✅ PostgreSQL: OK"
fi

# 4. Check Redis connectivity
if ! docker exec mr_darkpromth_redis redis-cli ping > /dev/null 2>&1; then
    alert "Redis is not responding!"
else
    log "✅ Redis: OK"
fi

# 5. Check disk space (alert if >85% used)
DISK_USAGE=$(df /opt | tail -1 | awk '{print $5}' | tr -d '%')
if [ "$DISK_USAGE" -gt 85 ]; then
    alert "Disk usage critical: ${DISK_USAGE}%"
else
    log "✅ Disk usage: ${DISK_USAGE}%"
fi

# 6. Check memory usage
MEM_USAGE=$(free | grep Mem | awk '{printf("%.0f", $3/$2 * 100)}')
if [ "$MEM_USAGE" -gt 90 ]; then
    alert "Memory usage critical: ${MEM_USAGE}%"
else
    log "✅ Memory usage: ${MEM_USAGE}%"
fi

# 7. Check SSL certificate expiry (if present)
if [ -f /opt/mrdarkpromth/ssl/cert.pem ]; then
    CERT_EXPIRY=$(openssl x509 -enddate -noout -in /opt/mrdarkpromth/ssl/cert.pem 2>/dev/null | cut -d= -f2)
    EXPIRY_EPOCH=$(date -d "$CERT_EXPIRY" +%s 2>/dev/null || echo "0")
    CURRENT_EPOCH=$(date +%s)
    DAYS_LEFT=$(( (EXPIRY_EPOCH - CURRENT_EPOCH) / 86400 ))
    if [ "$DAYS_LEFT" -lt 14 ]; then
        alert "SSL certificate expires in $DAYS_LEFT days!"
    else
        log "✅ SSL certificate: $DAYS_LEFT days remaining"
    fi
fi

log "=== Health Check Complete ==="
