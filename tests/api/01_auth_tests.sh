#!/bin/bash

# Source utilities
SCRIPT_DIR="$(dirname "$0")"
source "$SCRIPT_DIR/../../scripts/test_utils.sh"

echo "========================================"
echo "Running Authentication Tests"
echo "========================================"

# Test 1: Register New User
EMAIL="newuser_$(date +%s)@example.com"
PASSWORD="password123456"
echo "Registering user $EMAIL..."

response_code=$(curl -s -o /dev/null -w "%{http_code}" -X POST "$BASE_URL/api/auth/register" \
    -H "Content-Type: application/json" \
    -d "{\"email\":\"$EMAIL\",\"password\":\"$PASSWORD\",\"username\":\"user_$(date +%s)\"}")

if [ "$response_code" == "201" ] || [ "$response_code" == "200" ]; then
    log_pass "Registration successful (Code: $response_code)"
else
    log_fail "Registration failed (Code: $response_code)"
fi

# Test 2: Login Admin (Seeded)
login_user "admin@mrdarkpromth.ai" "admin123" "token_admin.txt"

# Test 3: Login Free User (Seeded)
login_user "testfree@mrdarkpromth.ai" "admin123" "token_free.txt"

# Test 4: Login Premium User (Seeded)
login_user "testpremium@mrdarkpromth.ai" "admin123" "token_premium.txt"

# Test 5: Login Ultra User (Seeded)
login_user "testultra@mrdarkpromth.ai" "admin123" "token_ultra.txt"

echo "Authentication Tests Completed Successfully."
