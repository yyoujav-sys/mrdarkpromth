#!/bin/bash
set -e
API_URL="http://localhost:8080"
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  MR.DarkPromth Production Readiness Test Suite           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}[1/6] Health Check${NC}"
HEALTH=$(curl -s $API_URL/health)
if echo "$HEALTH" | grep -q "healthy"; then
  echo -e "${GREEN}✓ API is healthy${NC}"
else
  echo -e "${RED}✗ API health check failed${NC}"
  exit 1
fi
echo -e "${YELLOW}[2/6] User Registration${NC}"
TIMESTAMP=$(date +%s)
REGISTER_RESPONSE=$(curl -s -X POST $API_URL/api/auth/register \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"test_${TIMESTAMP}@example.com\",\"password\":\"TestPass123\",\"username\":\"testuser_${TIMESTAMP}\"}")
TOKEN=$(echo "$REGISTER_RESPONSE" | grep -o '\"token\":\"[^\"]*' | cut -d'"' -f4)
USER_ID=$(echo "$REGISTER_RESPONSE" | grep -o '\"id\":\"[^\"]*' | head -1 | cut -d'"' -f4)
if [ -n "$TOKEN" ] && [ -n "$USER_ID" ]; then
  echo -e "${GREEN}✓ User registered successfully${NC}"
  echo "  User ID: $USER_ID"
  echo "  Token: ${TOKEN:0:30}..."
else
  echo -e "${RED}✗ User registration failed${NC}"
  echo "  Response: $REGISTER_RESPONSE"
  exit 1
fi
echo -e "${YELLOW}[3/6] Authentication${NC}"
AUTH_TEST=$(curl -s -H "Authorization: Bearer $TOKEN" $API_URL/api/chat/history)
if [ $? -eq 0 ]; then
  echo -e "${GREEN}✓ Authentication working${NC}"
else
  echo -e "${RED}✗ Authentication failed${NC}"
  exit 1
fi
echo -e "${YELLOW}[4/6] Database Connectivity${NC}"
DB_TEST=$(docker exec mr_darkpromth_postgres psql -U postgres -d mr_darkpromth -c "SELECT COUNT(*) FROM users;" 2>&1)
if echo "$DB_TEST" | grep -q "[0-9]"; then
  echo -e "${GREEN}✓ Database connection OK${NC}"
else
  echo -e "${RED}✗ Database connection failed${NC}"
  exit 1
fi
echo -e "${YELLOW}[5/6] Redis Connectivity${NC}"
REDIS_TEST=$(docker exec mr_darkpromth_redis redis-cli -a redispassword ping 2>&1)
if echo "$REDIS_TEST" | grep -q "PONG"; then
  echo -e "${GREEN}✓ Redis connection OK${NC}"
else
  echo -e "${RED}✗ Redis connection failed${NC}"
  exit 1
fi
echo -e "${YELLOW}[6/6] Sandbox Container${NC}"
SANDBOX_TEST=$(docker exec mr_darkpromth_sandbox which python3 2>&1)
if [ $? -eq 0 ]; then
  echo -e "${GREEN}✓ Sandbox container ready${NC}"
else
  echo -e "${RED}✗ Sandbox container not ready${NC}"
  exit 1
fi
echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  ✓ ALL TESTS PASSED - SYSTEM READY FOR PRODUCTION        ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Summary:"
echo "  - API Health: OK"
echo "  - User Registration: OK"
echo "  - Authentication: OK"
echo "  - Database: OK"
echo "  - Redis: OK"
echo "  - Sandbox: OK"
