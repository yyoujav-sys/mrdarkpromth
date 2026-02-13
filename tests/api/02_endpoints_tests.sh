#!/bin/bash

# Source utilities
SCRIPT_DIR="$(dirname "$0")"
source "$SCRIPT_DIR/../../scripts/test_utils.sh"

echo "========================================"
echo "Running General Endpoint Tests"
echo "========================================"

# Test 1: Health Check
echo "Checking Health Endpoint..."
response_code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/health")

if [ "$response_code" == "200" ]; then
    log_pass "Health check passed (Code: 200)"
else
    log_fail "Health check failed (Code: $response_code)"
fi

# Test 2: Public Billing Plans
echo "Checking Billing Plans..."
response_code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/billing/plans")

if [ "$response_code" == "200" ]; then
    log_pass "Billing plans accessible (Code: 200)"
else
    log_fail "Billing plans check failed (Code: $response_code)"
fi

# Test 3: User Profile (Authenticated)
TOKEN_FREE=$(cat "$TEMP_DIR/token_free.txt")
echo "Checking User Profile (Free User)..."

response_body=$(curl -s -H "Authorization: Bearer $TOKEN_FREE" "$BASE_URL/api/auth/me")
email=$(echo "$response_body" | jq -r '.data.email')
tier=$(echo "$response_body" | jq -r '.data.tier')

if [ "$email" == "testfree@mrdarkpromth.ai" ]; then
    log_pass "Profile email correct: $email"
else
    log_fail "Profile email incorrect. Expected: testfree@mrdarkpromth.ai, Got: $email"
fi

if [ "$tier" == "free" ]; then
    log_pass "Profile tier correct: $tier"
else
    log_fail "Profile tier incorrect. Expected: free, Got: $tier"
fi

echo "Endpoint Tests Completed Successfully."
