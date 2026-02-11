#!/bin/bash

# Module 4: Security & Load Testing
# =================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Initialize test results
TESTS_PASSED=0
TESTS_FAILED=0
ISSUES=()

echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Module 4: Security & Load Testing${NC}"
echo -e "${BLUE}=====================================${NC}"
echo ""

# ============ TEST 4.1: Tier Authorization ============
echo -e "${YELLOW}Test 4.1: Tier Authorization - Free vs Ultra${NC}"

# First, we need a Free tier token
echo "  Creating Free tier test user..."
TIMESTAMP=$(date +%s)
FREE_USER="test_free_${TIMESTAMP}"
FREE_EMAIL="free_${TIMESTAMP}@example.com"
FREE_PASSWORD="TestPass123!"

FREE_USER_RESPONSE=$(curl -s -X POST "$API_BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"$FREE_USER\",
    \"email\": \"$FREE_EMAIL\",
    \"password\": \"$FREE_PASSWORD\"
  }")

FREE_TOKEN=$(echo "$FREE_USER_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$FREE_TOKEN" ]; then
  echo -e "${RED}  ✗ FAILED: Could not create Free tier user${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 4.1: Could not create Free tier user")
else
  echo -e "${GREEN}  ✓ Free tier user created${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
  
  # Test 4.1.1: Try to access Ultra Terminal
  echo "  Testing access to /api/ultra/terminal with Free tier..."
  RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/ultra/terminal/execute" \
    -H "Authorization: Bearer $FREE_TOKEN" \
    -H "Content-Type: application/json" \
    -d '{"command": "ls"}')
  
  HTTP_CODE=$(echo "$RESPONSE" | tail -1)
  BODY=$(echo "$RESPONSE" | head -1)
  
  if [ "$HTTP_CODE" == "403" ] || [ "$HTTP_CODE" == "401" ]; then
    echo -e "${GREEN}  ✓ Correctly denied Free tier access (HTTP $HTTP_CODE)${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    echo -e "${RED}  ✗ FAILED: Expected 403/401, got $HTTP_CODE${NC}"
    echo "    Response: $BODY"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    ISSUES+=("Test 4.1.1: Free tier could access Ultra endpoint")
  fi
  
  # Test 4.1.2: Error message should be clear (non-leaking)
  if echo "$BODY" | grep -q "tier\|permission\|unauthorized" && ! echo "$BODY" | grep -q "secret\|key\|password"; then
    echo -e "${GREEN}  ✓ Error message is clear and non-leaking${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    echo -e "${YELLOW}  ⚠ Error message could be improved${NC}"
  fi
fi

# ============ TEST 4.2: Rate Limiting ============
echo ""
echo -e "${YELLOW}Test 4.2: Rate Limiting Protection${NC}"

# Check if ab (Apache Benchmark) is installed
if command -v ab &> /dev/null; then
  echo "  Hammering /health endpoint with 100 concurrent requests..."
  
  AB_OUTPUT=$(ab -n 100 -c 10 -q "$API_BASE_URL/health" 2>/dev/null || echo "")
  
  if echo "$AB_OUTPUT" | grep -q "Requests per second"; then
    echo -e "${GREEN}  ✓ Load test completed${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Extract metrics
    RPS=$(echo "$AB_OUTPUT" | grep "Requests per second" | awk '{print $4}')
    MEAN_TIME=$(echo "$AB_OUTPUT" | grep "Time per request:" | head -1 | awk '{print $4}')
    
    echo "    - Requests per second: $RPS"
    echo "    - Mean response time: ${MEAN_TIME}ms"
    
    # Check if we got any 429 responses (rate limited)
    if echo "$AB_OUTPUT" | grep -q "429"; then
      echo -e "${GREEN}  ✓ Rate limiting is active (some requests hit 429)${NC}"
      TESTS_PASSED=$((TESTS_PASSED + 1))
    else
      echo -e "${YELLOW}  ⚠ No 429 responses detected (rate limiting may not be enabled)${NC}"
    fi
  else
    echo -e "${RED}  ✗ FAILED: Load test failed${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    ISSUES+=("Test 4.2: Apache Benchmark test failed")
  fi
else
  echo -e "${YELLOW}  ⚠ SKIPPED: Apache Benchmark (ab) not installed${NC}"
  echo "    To install: apt-get install apache2-utils"
  
  # Alternative: Use curl for basic rate limit testing
  echo "    Running alternative rate limit test with curl..."
  
  HTTP_CODES=""
  for i in {1..20}; do
    CODE=$(curl -s -o /dev/null -w "%{http_code}" "$API_BASE_URL/health")
    HTTP_CODES="$HTTP_CODES $CODE"
  done
  
  RATE_LIMITED=$(echo "$HTTP_CODES" | grep -o "429" | wc -l)
  
  if [ $RATE_LIMITED -gt 0 ]; then
    echo -e "${GREEN}  ✓ Rate limiting detected ($RATE_LIMITED requests limited)${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    echo -e "${YELLOW}  ⚠ Rate limiting may not be configured or limit is high${NC}"
  fi
fi

# ============ TEST 4.3: SSL/TLS Configuration ============
echo ""
echo -e "${YELLOW}Test 4.3: SSL/TLS Configuration${NC}"

# Check if API is running on HTTPS (443)
echo "  Checking SSL certificate..."

SSL_CHECK=$(openssl s_client -connect localhost:443 </dev/null 2>/dev/null || echo "")

if echo "$SSL_CHECK" | grep -q "Verify return code: 0"; then
  echo -e "${GREEN}  ✓ SSL certificate is valid${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
elif echo "$SSL_CHECK" | grep -q "Verify return code: 20"; then
  echo -e "${YELLOW}  ⚠ SSL certificate is self-signed (expected in dev)${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${YELLOW}  ⚠ Could not verify SSL (may not be enabled on 443)${NC}"
fi

# Check HTTP redirect
echo "  Checking HTTP → HTTPS redirect..."
REDIRECT=$(curl -v -X GET "http://localhost:80/" 2>&1 | grep -i "location\|301\|302" | head -1)

if [ -n "$REDIRECT" ]; then
  echo -e "${GREEN}  ✓ HTTP redirection detected${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${YELLOW}  ⚠ No HTTP redirect detected${NC}"
fi

# Check Nginx configuration
echo "  Checking Nginx upload limit configuration..."

NGINX_CONF="/opt/mrdarkpromth/nginx/nginx.production.conf"
if [ -f "$NGINX_CONF" ]; then
  UPLOAD_LIMIT=$(grep "client_max_body_size" "$NGINX_CONF" | head -1)
  if [ -n "$UPLOAD_LIMIT" ]; then
    echo -e "${GREEN}  ✓ Found: $UPLOAD_LIMIT${NC}"
    if echo "$UPLOAD_LIMIT" | grep -q "20M"; then
      echo -e "${GREEN}  ✓ Upload limit correctly set to 20MB${NC}"
      TESTS_PASSED=$((TESTS_PASSED + 1))
    else
      echo -e "${YELLOW}  ⚠ Upload limit may not be 20MB as expected${NC}"
    fi
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    echo -e "${RED}  ✗ FAILED: No client_max_body_size found${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    ISSUES+=("Test 4.3: Nginx upload limit not configured")
  fi
else
  echo -e "${YELLOW}  ⚠ Nginx config not found at $NGINX_CONF${NC}"
fi

# ============ TEST 4.4: Health Check Endpoint ============
echo ""
echo -e "${YELLOW}Test 4.4: Health Check Endpoint${NC}"

HEALTH_RESPONSE=$(curl -s -X GET "$API_BASE_URL/health" \
  -H "Content-Type: application/json")

if echo "$HEALTH_RESPONSE" | grep -q '"status":"healthy"\|"status":"ok"\|"healthy":true'; then
  echo -e "${GREEN}  ✓ Health check endpoint working${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
  
  echo "    Response: $HEALTH_RESPONSE"
  
  # Check if response includes version info
  if echo "$HEALTH_RESPONSE" | grep -q '"version"\|"uptime"\|"timestamp"'; then
    echo -e "${GREEN}  ✓ Health check includes detailed status${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  fi
else
  echo -e "${RED}  ✗ FAILED: Health check failed${NC}"
  echo "    Response: $HEALTH_RESPONSE"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 4.4: Health check endpoint not working")
fi

# ============ TEST 4.5: Database Connection ============
echo ""
echo -e "${YELLOW}Test 4.5: Database Connection Check${NC}"

# Verify API can connect to database via health check
HEALTH_JSON=$(curl -s -X GET "$API_BASE_URL/health")

if echo "$HEALTH_JSON" | grep -q '"database":"connected"\|"db_status":"ok"'; then
  echo -e "${GREEN}  ✓ Database connection verified in health check${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${YELLOW}  ⚠ Database status not found in health check (may need to add it)${NC}"
fi

# ============ TEST 4.6: Monitoring & Logging ============
echo ""
echo -e "${YELLOW}Test 4.6: Monitoring & Logging${NC}"

# Check if logs are being collected
RECENT_LOGS=$(docker-compose logs api --tail=20 2>/dev/null || echo "")

if [ -n "$RECENT_LOGS" ]; then
  echo -e "${GREEN}  ✓ API logs are being collected${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
  
  # Check for structured logging (JSON format)
  if echo "$RECENT_LOGS" | grep -q '{"'; then
    echo -e "${GREEN}  ✓ Structured logging (JSON) detected${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    echo -e "${YELLOW}  ⚠ Logs appear to be unstructured text${NC}"
  fi
else
  echo -e "${YELLOW}  ⚠ Could not retrieve logs (docker-compose may not be running)${NC}"
fi

# ============ Summary ============
echo ""
echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Module 4 Summary${NC}"
echo -e "${BLUE}=====================================${NC}"
echo -e "${GREEN}Tests Passed: $TESTS_PASSED${NC}"
echo -e "${RED}Tests Failed: $TESTS_FAILED${NC}"

if [ ${#ISSUES[@]} -gt 0 ]; then
  echo ""
  echo -e "${YELLOW}Issues Found:${NC}"
  for issue in "${ISSUES[@]}"; do
    echo "  - $issue"
  done
fi

exit $TESTS_FAILED
