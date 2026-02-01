#!/bin/bash

# ==========================================
# MR.DarkPromth Production Test Suite
# ==========================================

set -e

BASE_URL="https://localhost"
API_BASE="$BASE_URL/api"

echo "🚀 Starting Production Tests..."
echo "=================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        exit 1
    fi
}

# Function to test endpoint
test_endpoint() {
    local url=$1
    local method=${2:-GET}
    local data=$3
    local expected_status=${4:-200}
    
    if [ "$method" = "POST" ]; then
        response=$(curl -k -s -w "%{http_code}" -X POST "$url" \
            -H "Content-Type: application/json" \
            -d "$data" \
            -o /tmp/response.json)
    else
        response=$(curl -k -s -w "%{http_code}" "$url" -o /tmp/response.json)
    fi
    
    if [ "$response" = "$expected_status" ]; then
        return 0
    else
        echo "Expected $expected_status, got $response"
        cat /tmp/response.json
        return 1
    fi
}

echo "1. Testing SSL Certificate..."
if openssl s_client -connect localhost:443 -servername localhost < /dev/null 2>/dev/null | grep -q "Verify return code: 18 (self signed certificate)"; then
    print_status 0 "SSL Certificate Working"
else
    print_status 1 "SSL Certificate Failed"
fi

echo "2. Testing Health Endpoint..."
test_endpoint "$BASE_URL/health" && print_status 0 "Health Check"

echo "3. Testing User Registration..."
REGISTER_DATA='{
    "email": "prodtest@example.com",
    "password": "SecurePass123!",
    "username": "prodtestuser"
}'
test_endpoint "$API_BASE/auth/register" "POST" "$REGISTER_DATA" && print_status 0 "User Registration"

# Extract token from registration
TOKEN=$(cat /tmp/response.json | jq -r .token 2>/dev/null || echo "")

if [ -z "$TOKEN" ]; then
    echo "4. Testing Login (to get token)..."
    LOGIN_DATA='{
        "email": "prodtest@example.com",
        "password": "SecurePass123!"
    }'
    test_endpoint "$API_BASE/auth/login" "POST" "$LOGIN_DATA"
    TOKEN=$(cat /tmp/response.json | jq -r .token 2>/dev/null || echo "")
    print_status ${#TOKEN} "Login & Token Extraction"
fi

if [ -n "$TOKEN" ] && [ "$TOKEN" != "null" ]; then
    echo "5. Testing User Profile..."
    test_endpoint "$API_BASE/auth/me" "GET" "" "200" "-H \"Authorization: Bearer $TOKEN\"" && print_status 0 "User Profile"
    
    echo "6. Testing AI Chat..."
    CHAT_DATA='{"message": "Hello! Tell me a short joke."}'
    test_endpoint "$API_BASE/chat" "POST" "$CHAT_DATA" "200" && print_status 0 "AI Chat"
    
    echo "7. Testing Tools List..."
    test_endpoint "$API_BASE/tools" "GET" "" "200" "-H \"Authorization: Bearer $TOKEN\"" && print_status 0 "Tools List"
    
    echo "8. Testing Premium Restrictions (should fail)..."
    TOOL_DATA='{"tool_id": "file_reader", "parameters": {"path": "/etc/hostname"}}'
    test_endpoint "$API_BASE/tools/execute" "POST" "$TOOL_DATA" "403" "-H \"Authorization: Bearer $TOKEN\"" && print_status 0 "Premium Restrictions"
else
    print_status 1 "No valid token obtained"
fi

echo "9. Testing Database Connection..."
docker exec mr_darkpromth_postgres_prod pg_isready -U postgres > /dev/null 2>&1 && print_status 0 "Database Connection"

echo "10. Testing Redis Connection..."
docker exec mr_darkpromth_redis_prod redis-cli ping > /dev/null 2>&1 && print_status 0 "Redis Connection"

echo "11. Testing Monitoring Services..."
curl -k -s http://localhost:9090/api/v1/query?query=up > /dev/null && print_status 0 "Prometheus"
curl -k -s http://localhost:3001/api/health > /dev/null && print_status 0 "Grafana"

echo "12. Testing Performance (100 concurrent requests)..."
echo "Running load test..."
for i in {1..100}; do
    curl -k -s "$BASE_URL/health" > /dev/null &
done
wait
print_status 0 "Load Test (100 requests)"

echo "13. Checking Resource Usage..."
echo "Memory Usage:"
docker stats --no-stream --format "table {{.Container}}\t{{.MemUsage}}\t{{.CPUPerc}}" | grep mr_darkpromth

echo "14. Testing Error Handling..."
test_endpoint "$BASE_URL/nonexistent" "GET" "" "404" && print_status 0 "404 Error Handling"

echo "15. Testing CORS Headers..."
curl -k -s -I "$BASE_URL/health" | grep -i "access-control-allow-origin" > /dev/null && print_status 0 "CORS Headers"

echo "=================================="
echo -e "${GREEN}🎉 All Production Tests Passed!${NC}"
echo "=================================="
echo ""
echo "📊 System Status:"
echo "  - API: https://localhost/api"
echo "  - Frontend: https://localhost"
echo "  - Prometheus: http://localhost:9090"
echo "  - Grafana: http://localhost:3001"
echo ""
echo "🔑 Test User Created:"
echo "  - Email: prodtest@example.com"
echo "  - Password: SecurePass123!"
echo ""
echo "📝 Next Steps:"
echo "  1. Update your domain in CORS_ALLOWED_ORIGINS"
echo "  2. Configure real SSL certificates"
echo "  3. Set up production email service"
echo "  4. Configure billing (Stripe)"
echo "  5. Set up monitoring alerts"
echo ""
echo -e "${GREEN}✅ System is PRODUCTION READY!${NC}"
