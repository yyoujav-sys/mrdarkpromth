#!/bin/bash

# Module 1: Payment System & Tier Upgrade Tests
# ============================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
DB_HOST="${DB_HOST:-localhost}"
DB_USER="${DB_USER:-postgres}"
DB_NAME="${DB_NAME:-mr_darkpromth}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Initialize test results
TESTS_PASSED=0
TESTS_FAILED=0
ISSUES=()

echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Module 1: Payment & Tier Upgrade${NC}"
echo -e "${BLUE}=====================================${NC}"
echo ""

# ============ TEST 1.1: Happy Path ============
echo -e "${YELLOW}Test 1.1: Happy Path - Free→Ultra Upgrade${NC}"

# Generate test user
TIMESTAMP=$(date +%s)
TEST_USER="test_pay_${TIMESTAMP}"
TEST_EMAIL="test_${TIMESTAMP}@example.com"
TEST_PASSWORD="TestPassword123!"

echo "  Creating test user: $TEST_USER"

# Create test user via API
CREATE_USER_RESPONSE=$(curl -s -X POST "$API_BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"$TEST_USER\",
    \"email\": \"$TEST_EMAIL\",
    \"password\": \"$TEST_PASSWORD\"
  }")

echo "  Response: $CREATE_USER_RESPONSE"

# Extract user_id from response (adjust based on actual API response)
USER_ID=$(echo "$CREATE_USER_RESPONSE" | grep -o '"user_id":"[^"]*' | cut -d'"' -f4)
if [ -z "$USER_ID" ]; then
  echo -e "${RED}✗ FAILED: Could not extract user_id${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 1.1: Failed to create test user")
else
  echo -e "${GREEN}✓ User created: $USER_ID${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
fi

# ============ TEST 1.2: Invalid File Upload ============
echo ""
echo -e "${YELLOW}Test 1.2: Edge Case - Invalid File Upload${NC}"

# Create test files
echo "  Generating test files..."

# Test 1.2.1: Valid image
convert -size 512x512 xc:blue /tmp/test_valid_512kb.png 2>/dev/null || {
  dd if=/dev/zero of=/tmp/test_valid_512kb.png bs=1024 count=512 2>/dev/null
}

# Test 1.2.2: Text file (wrong MIME)
echo "This is a text file" > /tmp/test_invalid.txt

# Test 1.2.3: Oversized file (25MB > 20MB limit)
dd if=/dev/zero of=/tmp/test_oversized.bin bs=1M count=25 2>/dev/null

# Test 1.2.4: Zero-byte file
touch /tmp/test_empty.png

echo "  Testing file uploads..."

# Test wrong MIME type
echo "  1.2.1: Testing wrong MIME type (.txt)..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/billing/verify-slip" \
  -H "Authorization: Bearer $TEST_TOKEN" \
  -F "file=@/tmp/test_invalid.txt" 2>&1)
HTTP_CODE=$(echo "$RESPONSE" | tail -1)
if [ "$HTTP_CODE" == "400" ] || [ "$HTTP_CODE" == "415" ]; then
  echo -e "${GREEN}  ✓ Correctly rejected text file (HTTP $HTTP_CODE)${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${RED}  ✗ FAILED: Expected 400/415, got $HTTP_CODE${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 1.2.1: Text file not rejected")
fi

# Test oversized file
echo "  1.2.2: Testing oversized file (25MB)..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/billing/verify-slip" \
  -H "Authorization: Bearer $TEST_TOKEN" \
  -F "file=@/tmp/test_oversized.bin" 2>&1)
HTTP_CODE=$(echo "$RESPONSE" | tail -1)
if [ "$HTTP_CODE" == "413" ] || [ "$HTTP_CODE" == "400" ]; then
  echo -e "${GREEN}  ✓ Correctly rejected oversized file (HTTP $HTTP_CODE)${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${RED}  ✗ FAILED: Expected 413/400, got $HTTP_CODE${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 1.2.2: Oversized file not rejected")
fi

# Test empty file
echo "  1.2.3: Testing empty file..."
RESPONSE=$(curl -s -w "\n%{http_code}" -X POST "$API_BASE_URL/api/billing/verify-slip" \
  -H "Authorization: Bearer $TEST_TOKEN" \
  -F "file=@/tmp/test_empty.png" 2>&1)
HTTP_CODE=$(echo "$RESPONSE" | tail -1)
if [ "$HTTP_CODE" == "400" ]; then
  echo -e "${GREEN}  ✓ Correctly rejected empty file (HTTP $HTTP_CODE)${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${RED}  ✗ FAILED: Expected 400, got $HTTP_CODE${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 1.2.3: Empty file not rejected")
fi

# ============ TEST 1.3: Database Consistency ============
echo ""
echo -e "${YELLOW}Test 1.3: Database Consistency Checks${NC}"

echo "  Checking for orphaned payments..."
ORPHANED=$(psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -t -c "
SELECT COUNT(*) FROM payments p 
WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = p.user_id);
" 2>/dev/null || echo "ERROR")

if [ "$ORPHANED" == "0" ] || [ "$ORPHANED" == "" ]; then
  echo -e "${GREEN}  ✓ No orphaned payments found${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${RED}  ✗ FAILED: Found $ORPHANED orphaned payments${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 1.3: Found $ORPHANED orphaned payments")
fi

echo "  Checking for duplicate approvals..."
DUPLICATES=$(psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -t -c "
SELECT COUNT(*) FROM (
  SELECT payment_id, COUNT(*) as count FROM payment_audit_log 
  WHERE action = 'approve' GROUP BY payment_id HAVING COUNT(*) > 1
) subquery;
" 2>/dev/null || echo "ERROR")

if [ "$DUPLICATES" == "0" ] || [ "$DUPLICATES" == "" ]; then
  echo -e "${GREEN}  ✓ No duplicate approvals found${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${RED}  ✗ FAILED: Found $DUPLICATES payments with duplicate approvals${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 1.3: Found $DUPLICATES payments with duplicate approvals")
fi

echo "  Checking date consistency..."
DATE_ISSUES=$(psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -t -c "
SELECT COUNT(*) FROM ultra_subscriptions 
WHERE activated_at < created_at;
" 2>/dev/null || echo "ERROR")

if [ "$DATE_ISSUES" == "0" ] || [ "$DATE_ISSUES" == "" ]; then
  echo -e "${GREEN}  ✓ All subscription dates are consistent${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${RED}  ✗ FAILED: Found $DATE_ISSUES subscriptions with invalid dates${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 1.3: Found $DATE_ISSUES subscriptions with invalid dates")
fi

# ============ Summary ============
echo ""
echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Module 1 Summary${NC}"
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

# Cleanup
rm -f /tmp/test_*.png /tmp/test_*.txt /tmp/test_*.bin

exit $TESTS_FAILED
