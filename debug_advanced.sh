#!/bin/bash
# 🔍 MR.DarkPromth Advanced Debug Script

echo "=== 🔍 MR.DarkPromth Advanced Debug ==="
echo "Time: $(date)"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Function to print section header
print_section() {
    echo -e "${BLUE}=== $1 ===${NC}"
}

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
    fi
}

# 1. Network Deep Dive
print_section "🌐 Network Deep Dive"
echo "Network Configuration:"
docker network inspect mr_darkpromth_network | jq '.[0].Containers' 2>/dev/null || echo "Network inspection failed"
echo ""

echo "Container Network Aliases:"
for container in mr_darkpromth_api mr_darkpromth_postgres mr_darkpromth_redis; do
    echo -n "$container: "
    docker inspect $container | jq -r '.[0].NetworkSettings.Networks.mr_darkpromth_network.IPAddress' 2>/dev/null || echo "N/A"
done
echo ""

# 2. Database Deep Dive
print_section "🗄️ Database Deep Dive"
echo "Database Connections:"
docker exec mr_darkpromth_postgres psql -U postgres -d mr_darkpromth -c "
SELECT 
    pid,
    state,
    application_name,
    client_addr,
    backend_start
FROM pg_stat_activity 
WHERE state != 'idle';
" 2>/dev/null || echo "Failed to get DB connections"
echo ""

echo "Table Sizes:"
docker exec mr_darkpromth_postgres psql -U postgres -d mr_darkpromth -c "
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
FROM pg_tables 
WHERE schemaname = 'public'
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
" 2>/dev/null || echo "Failed to get table sizes"
echo ""

# 3. API Deep Dive
print_section "🔌 API Deep Dive"
echo "API Environment Variables:"
docker exec mr_darkpromth_api env | grep -E "(DATABASE|REDIS|PORT)" | sort
echo ""

echo "API Process Details:"
docker exec mr_darkpromth_api ps aux | grep -v "CMD"
echo ""

echo "API Port Usage:"
docker exec mr_darkpromth_api netstat -tlnp 2>/dev/null | grep :8080 || echo "Port 8080 not found"
echo ""

# 4. Frontend Deep Dive
print_section "🌐 Frontend Deep Dive"
echo "Nginx Configuration:"
docker exec mr_darkpromth_frontend_react cat /etc/nginx/conf.d/default.conf | head -20
echo ""

echo "Static Assets:"
docker exec mr_darkpromth_frontend_react ls -la /usr/share/nginx/html/assets/ | head -10
echo ""

# 5. SSL Deep Dive
print_section "🔐 SSL Deep Dive"
echo "Certificate Details:"
openssl x509 -in /etc/letsencrypt/live/bt-shop-dark.online/cert.pem -text -noout | grep -A 2 "Subject Alternative Name"
echo ""

echo "Certificate Chain:"
openssl s_client -connect bt-shop-dark.online:443 -servername bt-shop-dark.online 2>/dev/null | grep -E "(subject|issuer|Verify)" | head -5
echo ""

# 6. Performance Analysis
print_section "📊 Performance Analysis"
echo "Container Resource Usage (Detailed):"
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.NetIO}}\t{{.BlockIO}}" | grep mr_darkpromth
echo ""

echo "System Load:"
uptime
echo ""

echo "Disk Usage by Container:"
docker system df -v
echo ""

# 7. Error Analysis
print_section "🚨 Error Analysis"
echo "Recent API Errors:"
docker logs mr_darkpromth_api --since 1h | grep -i error | tail -5 || echo "No recent API errors"
echo ""

echo "Recent Frontend Errors:"
docker logs mr_darkpromth_frontend_react --since 1h | grep -i error | tail -5 || echo "No recent frontend errors"
echo ""

echo "Database Errors:"
docker exec mr_darkpromth_postgres psql -U postgres -d mr_darkpromth -c "
SELECT 
    log_time,
    error_severity,
    message
FROM pg_log 
WHERE log_time > now() - interval '1 hour'
ORDER BY log_time DESC
LIMIT 5;
" 2>/dev/null || echo "No recent DB errors"
echo ""

# 8. Security Check
print_section "🔒 Security Check"
echo "Open Ports:"
netstat -tlnp | grep -E ":80|:443|:8080|:3000|:3001|:5432|:6379"
echo ""

echo "Failed Login Attempts (last hour):"
docker logs mr_darkpromth_api --since 1h | grep -i "failed\|error\|unauthorized" | tail -3 || echo "No failed attempts"
echo ""

# 9. Health Check Summary
print_section "🏥 Health Check Summary"
services=("mr_darkpromth_api:8080" "mr_darkpromth_postgres:5432" "mr_darkpromth_redis:6379" "mr_darkpromth_frontend_react:80")

for service in "${services[@]}"; do
    name=$(echo $service | cut -d: -f1)
    port=$(echo $service | cut -d: -f2)
    
    if docker exec $name netstat -tlnp 2>/dev/null | grep -q ":$port"; then
        print_status 0 "$name port $port"
    else
        print_status 1 "$name port $port"
    fi
done
echo ""

# 10. Recommendations
print_section "💡 Recommendations"
echo "Based on current system state:"

# Check resource usage
HIGH_CPU=$(docker stats --no-stream --format "{{.CPUPerc}}" | grep -v "%" | awk '{if ($1+0 > 50) print $1}')
HIGH_MEM=$(docker stats --no-stream --format "{{.MemPerc}}" | grep -v "%" | awk '{if ($1+0 > 80) print $1}')

if [ ! -z "$HIGH_CPU" ]; then
    echo -e "${YELLOW}⚠️  High CPU usage detected: $HIGH_CPU${NC}"
fi

if [ ! -z "$HIGH_MEM" ]; then
    echo -e "${YELLOW}⚠️  High memory usage detected: $HIGH_MEM${NC}"
fi

# Check disk space
DISK_USAGE=$(df / | awk 'NR==2 {print $5}' | sed 's/%//')
if [ $DISK_USAGE -gt 80 ]; then
    echo -e "${YELLOW}⚠️  Disk usage high: ${DISK_USAGE}%${NC}"
fi

# Check SSL expiry
DAYS_TO_EXPIRY=$(openssl x509 -noout -days -in /etc/letsencrypt/live/bt-shop-dark.online/cert.pem 2>/dev/null)
if [ $DAYS_TO_EXPIRY -lt 30 ]; then
    echo -e "${YELLOW}⚠️  SSL certificate expires in $DAYS_TO_EXPIRY days${NC}"
fi

echo -e "${GREEN}✅ System appears to be running normally${NC}"
echo ""

echo -e "${BLUE}📋 Advanced debug complete!${NC}"
