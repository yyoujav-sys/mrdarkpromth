#!/bin/bash
# 🔍 MR.DarkPromth System Debug Script

echo "=== 🔍 MR.DarkPromth System Debug ==="
echo "Time: $(date)"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
    fi
}

echo -e "${BLUE}📦 Containers Status:${NC}"
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep mr_darkpromth
echo ""

echo -e "${BLUE}🌐 Network Test:${NC}"
# Test API to PostgreSQL
if docker exec mr_darkpromth_api ping -c 1 mr_darkpromth_postgres >/dev/null 2>&1; then
    print_status 0 "API → PostgreSQL"
else
    print_status 1 "API → PostgreSQL"
fi

# Test API to Redis
if docker exec mr_darkpromth_api ping -c 1 mr_darkpromth_redis >/dev/null 2>&1; then
    print_status 0 "API → Redis"
else
    print_status 1 "API → Redis"
fi
echo ""

echo -e "${BLUE}🔌 API Test:${NC}"
if curl -s http://localhost:8080/health | grep -q "healthy"; then
    print_status 0 "API Health"
else
    print_status 1 "API Health"
fi

# Test API endpoints
if curl -s http://localhost:8080/api/status | grep -q "running"; then
    print_status 0 "API Status"
else
    print_status 1 "API Status"
fi
echo ""

echo -e "${BLUE}🌐 Frontend Test:${NC}"
if curl -s -o /dev/null -w "%{http_code}" https://bt-shop-dark.online | grep -q "200"; then
    print_status 0 "Frontend HTTP"
else
    print_status 1 "Frontend HTTP"
fi

if curl -s -o /dev/null -w "%{http_code}" https://bt-shop-dark.online/landing | grep -q "200"; then
    print_status 0 "Landing Page"
else
    print_status 1 "Landing Page"
fi
echo ""

echo -e "${BLUE}🔐 SSL Status:${NC}"
if certbot certificates 2>/dev/null | grep -q "bt-shop-dark.online"; then
    print_status 0 "SSL Certificate"
else
    print_status 1 "SSL Certificate"
fi

# Check SSL expiration
if openssl x509 -noout -dates -in /etc/letsencrypt/live/bt-shop-dark.online/cert.pem 2>/dev/null | grep -q "notAfter"; then
    EXPIRY=$(openssl x509 -noout -dates -in /etc/letsencrypt/live/bt-shop-dark.online/cert.pem 2>/dev/null | grep notAfter | cut -d= -f2)
    echo -e "${GREEN}✅ SSL expires: $EXPIRY${NC}"
else
    print_status 1 "SSL Expiration Date"
fi
echo ""

echo -e "${BLUE}📊 Resource Usage:${NC}"
docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}" | grep mr_darkpromth
echo ""

echo -e "${BLUE}🗄️ Database Status:${NC}"
if docker exec mr_darkpromth_postgres pg_isready -U postgres >/dev/null 2>&1; then
    print_status 0 "PostgreSQL Ready"
    
    # Check connection count
    CONN_COUNT=$(docker exec mr_darkpromth_postgres psql -U postgres -t -c "SELECT count(*) FROM pg_stat_activity WHERE state = 'active';" 2>/dev/null | tr -d ' ')
    echo -e "${GREEN}✅ Active DB Connections: $CONN_COUNT${NC}"
else
    print_status 1 "PostgreSQL Ready"
fi
echo ""

echo -e "${BLUE}🔴 Redis Status:${NC}"
if docker exec mr_darkpromth_redis redis-cli ping 2>/dev/null | grep -q "PONG"; then
    print_status 0 "Redis Connection"
    
    # Check Redis memory
    REDIS_MEMORY=$(docker exec mr_darkpromth_redis redis-cli info memory 2>/dev/null | grep used_memory_human | cut -d: -f2 | tr -d '\r')
    echo -e "${GREEN}✅ Redis Memory: $REDIS_MEMORY${NC}"
else
    print_status 1 "Redis Connection"
fi
echo ""

echo -e "${BLUE}📋 Recent Errors (Last 10 lines):${NC}"
echo "API Logs:"
docker logs mr_darkpromth_api --tail 10 2>&1 | grep -i error || echo "No errors in API logs"
echo ""
echo "Frontend Logs:"
docker logs mr_darkpromth_frontend_react --tail 10 2>&1 | grep -i error || echo "No errors in frontend logs"
echo ""

echo -e "${BLUE}🔍 Quick Health Check:${NC}"
HEALTH_URLS=("http://localhost:8080/health" "http://localhost:8080/api/status" "https://bt-shop-dark.online")
for url in "${HEALTH_URLS[@]}"; do
    if curl -s "$url" >/dev/null; then
        print_status 0 "$url"
    else
        print_status 1 "$url"
    fi
done
echo ""

echo -e "${YELLOW}💾 Saving debug report to /tmp/mrdarkpromth_debug_$(date +%Y%m%d_%H%M%S).log${NC}"
{
    echo "=== MR.DarkPromth Debug Report ==="
    echo "Generated: $(date)"
    echo ""
    echo "=== Container Status ==="
    docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep mr_darkpromth
    echo ""
    echo "=== System Resources ==="
    df -h
    echo ""
    free -h
    echo ""
    echo "=== Docker System Info ==="
    docker system df
} > "/tmp/mrdarkpromth_debug_$(date +%Y%m%d_%H%M%S).log"

echo -e "${GREEN}=== 🔍 Debug Complete ===${NC}"
echo -e "${BLUE}📋 Full report saved to /tmp/mrdarkpromth_debug_$(date +%Y%m%d_%H%M%S).log${NC}"
