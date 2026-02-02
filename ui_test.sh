#!/bin/bash
# 🧪 MR.DarkPromth UI Testing Script

echo "=== 🧪 MR.DarkPromth UI Testing ==="
echo "Time: $(date)"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

BASE_URL="https://bt-shop-dark.online"

echo -e "${BLUE}🌐 Testing Frontend Loading${NC}"

# Test 1: Main page loads
echo -n "Testing main page... "
if curl -s -o /dev/null -w "%{http_code}" "$BASE_URL" | grep -q "200"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test 2: Landing page loads
echo -n "Testing landing page... "
if curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/landing" | grep -q "200"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test 3: CSS loads
echo -n "Testing CSS loading... "
if curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/assets/index-OqyOin9j.css" | grep -q "200"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test 4: JavaScript loads
echo -n "Testing JavaScript loading... "
if curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/assets/index-RDiDuoCZ.js" | grep -q "200"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

echo ""
echo -e "${BLUE}🔌 Testing API Endpoints${NC}"

# Test 5: API Health
echo -n "Testing API health... "
if curl -s "$BASE_URL/api/health" | grep -q "healthy"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test 6: API Status
echo -n "Testing API status... "
if curl -s "$BASE_URL/api/status" | grep -q "running"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test 7: API Info
echo -n "Testing API info... "
if curl -s "$BASE_URL/api/info" | grep -q "MR.DarkPromth"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

echo ""
echo -e "${BLUE}🎨 Testing UI Elements${NC}"

# Test 8: Check for dark theme CSS
echo -n "Testing dark theme CSS... "
if curl -s "$BASE_URL/assets/index-OqyOin9j.css" | grep -q "background-color:#0a0a0a"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test 9: Check for neon effects
echo -n "Testing neon effects CSS... "
if curl -s "$BASE_URL/assets/index-OqyOin9j.css" | grep -q "neon-border"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test 10: Check for gradient text
echo -n "Testing gradient text CSS... "
if curl -s "$BASE_URL/assets/index-OqyOin9j.css" | grep -q "gradient-text"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

echo ""
echo -e "${BLUE}📱 Testing Responsive Design${NC}"

# Test 11: Mobile viewport
echo -n "Testing mobile viewport... "
if curl -s -H "User-Agent: Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X)" "$BASE_URL" | grep -q "viewport"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test 12: Tablet viewport
echo -n "Testing tablet viewport... "
if curl -s -H "User-Agent: Mozilla/5.0 (iPad; CPU OS 14_0 like Mac OS X)" "$BASE_URL" | grep -q "viewport"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

echo ""
echo -e "${BLUE}🔒 Testing Security Headers${NC}"

# Test 13: HTTPS redirect
echo -n "Testing HTTPS redirect... "
if curl -s -o /dev/null -w "%{http_code}" "http://bt-shop-dark.online" | grep -q "301\|302"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

# Test 14: Security headers
echo -n "Testing security headers... "
if curl -s -I "$BASE_URL" | grep -i "x-frame-options\|x-content-type-options\|x-xss-protection"; then
    echo -e "${GREEN}✅ PASS${NC}"
else
    echo -e "${RED}❌ FAIL${NC}"
fi

echo ""
echo -e "${BLUE}⚡ Performance Tests${NC}"

# Test 15: Page load time
echo -n "Testing page load time... "
LOAD_TIME=$(curl -s -o /dev/null -w "%{time_total}" "$BASE_URL")
if (( $(echo "$LOAD_TIME < 2.0" | bc -l) )); then
    echo -e "${GREEN}✅ PASS (${LOAD_TIME}s)${NC}"
else
    echo -e "${YELLOW}⚠️  SLOW (${LOAD_TIME}s)${NC}"
fi

# Test 16: Asset sizes
echo -n "Testing asset sizes... "
CSS_SIZE=$(curl -s -o /dev/null -w "%{size_download}" "$BASE_URL/assets/index-OqyOin9j.css")
JS_SIZE=$(curl -s -o /dev/null -w "%{size_download}" "$BASE_URL/assets/index-RDiDuoCZ.js")
TOTAL_SIZE=$((CSS_SIZE + JS_SIZE))

if [ $TOTAL_SIZE -lt 1000000 ]; then
    echo -e "${GREEN}✅ PASS ($(($TOTAL_SIZE / 1024))KB)${NC}"
else
    echo -e "${YELLOW}⚠️  LARGE ($(($TOTAL_SIZE / 1024))KB)${NC}"
fi

echo ""
echo -e "${BLUE}🔍 Detailed Analysis${NC}"

echo "Page Structure:"
curl -s "$BASE_URL" | grep -E "<title>|<meta|<link" | head -5

echo ""
echo "CSS Variables:"
curl -s "$BASE_URL/assets/index-OqyOin9j.css" | grep -o "var(--[^)]*)" | head -5

echo ""
echo "API Response Sample:"
curl -s "$BASE_URL/api/info" | head -1

echo ""
echo -e "${GREEN}=== 🧪 UI Testing Complete ===${NC}"
echo -e "${BLUE}📋 Test results saved to /tmp/mrdarkpromth_ui_test_$(date +%Y%m%d_%H%M%S).log${NC}"

# Save test results
{
    echo "=== MR.DarkPromth UI Test Results ==="
    echo "Generated: $(date)"
    echo "Base URL: $BASE_URL"
    echo ""
    echo "=== Asset Sizes ==="
    echo "CSS: $(curl -s -o /dev/null -w "%{size_download}" "$BASE_URL/assets/index-OqyOin9j.css") bytes"
    echo "JS: $(curl -s -o /dev/null -w "%{size_download}" "$BASE_URL/assets/index-RDiDuoCZ.js") bytes"
    echo ""
    echo "=== Load Times ==="
    echo "Main page: $(curl -s -o /dev/null -w "%{time_total}" "$BASE_URL)s"
    echo "Landing page: $(curl -s -o /dev/null -w "%{time_total}" "$BASE_URL/landing")s"
    echo ""
    echo "=== HTTP Status Codes ==="
    echo "Main page: $(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL")"
    echo "API health: $(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/health")"
} > "/tmp/mrdarkpromth_ui_test_$(date +%Y%m%d_%H%M%S).log"
