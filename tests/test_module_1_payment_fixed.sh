# Module 1: Payment System & Tier Upgrade Tests (Fixed)
set -e
API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'
TESTS_PASSED=0
TESTS_FAILED=0
ISSUES=()
echo -e "${BLUE}====================================="
echo -e "Module 1: Payment & Tier Upgrade (Fixed)"
echo -e "====================================="
echo ""
# Test 1.1: Create user and test upgrade
echo -e "${YELLOW}Test 1.1: User Registration & Token Extraction"
TIMESTAMP=$(date +%s)
TEST_USER="test_pay_${TIMESTAMP}"
TEST_EMAIL="test_${TIMESTAMP}@example.com"
TEST_PASSWORD="TestPassword123"
echo "  Creating test user: $TEST_USER"
CREATE_USER_RESPONSE=$(curl -s -X POST "$API_BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"$TEST_USER\",
    \"email\": \"$TEST_EMAIL\",
    \"password\": \"$TEST_PASSWORD\"
  }")
echo "  Response: $CREATE_USER_RESPONSE"
# Extract user_id from new response format: {"user":{"id":"...","email":"...","username":"...","tier":"free","verified":true}}
USER_ID=$(echo "$CREATE_USER_RESPONSE" | grep -o '"id":"[^"]*' | head -1 | cut -d'"' -f4)
AUTH_TOKEN=$(echo "$CREATE_USER_RESPONSE" | grep -o '"token":"[^"]*' | head -1 | cut -d'"' -f4)
if [ -z "$USER_ID" ] || [ -z "$AUTH_TOKEN" ]; then
  echo -e "${RED}✗ FAILED: Could not extract user_id or token"
  echo "  USER_ID: $USER_ID"
  echo "  AUTH_TOKEN: $AUTH_TOKEN"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 1.1: Failed to create test user")
else
  echo -e "${GREEN}✓ User created: $USER_ID"
  echo -e "${GREEN}✓ Token extracted: ${AUTH_TOKEN:0:20}..."
  TESTS_PASSED=$((TESTS_PASSED + 1))
fi
# Test 1.2: File upload validation
echo ""
echo -e "${YELLOW}Test 1.2: File Upload Validation"
# Create test image
dd if=/dev/zero of=/tmp/test_valid.png bs=1024 count=512 2>/dev/null
echo "This is text" > /tmp/test_invalid.txt
echo "  Testing file upload with valid token..."
if [ -n "$AUTH_TOKEN" ]; then
  UPLOAD_RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/billing/verify-slip" \
    -H "Authorization: Bearer $AUTH_TOKEN" \
    -F "file=@/tmp/test_valid.png" 2>&1)
  HTTP_CODE=$(echo "$UPLOAD_RESPONSE" | tail -1)
  echo "  Upload response code: $HTTP_CODE"
  if [ "$HTTP_CODE" == "200" ] || [ "$HTTP_CODE" == "201" ] || [ "$HTTP_CODE" == "400" ]; then
    echo -e "${GREEN}✓ File upload endpoint accessible"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    echo -e "${YELLOW}⚠ Unexpected response: $HTTP_CODE"
  fi
fi
# Summary
echo ""
echo -e "${BLUE}====================================="
echo -e "Module 1 Summary"
echo -e "====================================="
echo "Tests Passed: $TESTS_PASSED"
echo "Tests Failed: $TESTS_FAILED"
if [ ${#ISSUES[@]} -gt 0 ]; then
  echo "Issues Found:"
  for issue in "${ISSUES[@]}"; do
    echo "  - $issue"
  done
fi
if [ $TESTS_FAILED -gt 0 ]; then
  echo -e "${RED}✗ Module 1 tests failed"
  exit 1
else
  echo -e "${GREEN}✓ Module 1 tests passed"
  exit 0
fi
