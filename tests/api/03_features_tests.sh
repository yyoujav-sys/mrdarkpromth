#!/bin/bash

# Source utilities
SCRIPT_DIR="$(dirname "$0")"
source "$SCRIPT_DIR/../../scripts/test_utils.sh"

echo "========================================"
echo "Running Feature Access Tests"
echo "========================================"

TOKEN_FREE=$(cat "$TEMP_DIR/token_free.txt")
TOKEN_PREMIUM=$(cat "$TEMP_DIR/token_premium.txt")
TOKEN_ULTRA=$(cat "$TEMP_DIR/token_ultra.txt")

# Test 1: Chat Access (All Tiers)
echo "Testing Chat Access (Free Tier)..."
response_code=$(curl -s -o /dev/null -w "%{http_code}" -H "Authorization: Bearer $TOKEN_FREE" "$BASE_URL/api/chat/history")
if [ "$response_code" == "200" ]; then
    log_pass "Chat accessible for Free tier"
else
    log_fail "Chat not accessible for Free tier (Code: $response_code)"
fi

# Test 2: Sandbox Access (Premium Only)
echo "Testing Sandbox Access (Free Tier - Should Fail)..."
response_code=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Authorization: Bearer $TOKEN_FREE" -H "Content-Type: application/json" -d '{"code":"print(1)","language":"python"}' "$BASE_URL/api/sandbox/execute")
if [ "$response_code" == "403" ]; then
    log_pass "Sandbox restricted for Free tier (Code: 403)"
else
    log_fail "Sandbox NOT restricted for Free tier (Code: $response_code)"
fi

echo "Testing Sandbox Access (Premium Tier - Should Succeed)..."
# Just checking access, not full execution which requires body
response_code=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Authorization: Bearer $TOKEN_PREMIUM" -H "Content-Type: application/json" -d '{}' "$BASE_URL/api/sandbox/execute")
# Expect 400 (Bad Request) or 200, but definitely NOT 403
if [ "$response_code" != "403" ]; then
    log_pass "Sandbox accessible for Premium tier (Code: $response_code)"
else
    log_fail "Sandbox restricted for Premium tier (Code: 403)"
fi

# Test 3: Terminal Access (Ultra Only)
echo "Testing Terminal Access (Premium Tier - Should Fail)..."
response_code=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Authorization: Bearer $TOKEN_PREMIUM" -H "Content-Type: application/json" -d '{"command":"ls"}' "$BASE_URL/api/terminal/execute")
if [ "$response_code" == "403" ]; then
    log_pass "Terminal restricted for Premium tier (Code: 403)"
else
    log_fail "Terminal NOT restricted for Premium tier (Code: $response_code)"
fi

echo "Testing Terminal Access (Ultra Tier - Should Succeed)..."
# Just checking access
response_code=$(curl -s -o /dev/null -w "%{http_code}" -X POST -H "Authorization: Bearer $TOKEN_ULTRA" -H "Content-Type: application/json" -d '{}' "$BASE_URL/api/terminal/execute")
if [ "$response_code" != "403" ]; then
    log_pass "Terminal accessible for Ultra tier (Code: $response_code)"
else
    log_fail "Terminal restricted for Ultra tier (Code: 403)"
fi

echo "Feature Tests Completed Successfully."
