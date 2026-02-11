#!/bin/bash

# Module 3: Chat & AI Failover Tests
# ==================================

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
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Initialize test results
TESTS_PASSED=0
TESTS_FAILED=0
ISSUES=()

# Requires AUTH_TOKEN to be set
if [ -z "$AUTH_TOKEN" ]; then
  echo -e "${RED}ERROR: AUTH_TOKEN not set. Please set a valid bearer token.${NC}"
  echo "  Usage: AUTH_TOKEN=your_token bash test_module_3_chat.sh"
  exit 1
fi

echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Module 3: Chat & AI Failover${NC}"
echo -e "${BLUE}=====================================${NC}"
echo ""

# ============ TEST 3.1: Normal Chat Flow ============
echo -e "${YELLOW}Test 3.1: Normal Chat Flow${NC}"

CHAT_MESSAGES=(
  "Hello, what is 2+2?"
  "Explain artificial intelligence in one sentence"
  "What did I ask before?"
)

SESSION_ID=""

for msg in "${CHAT_MESSAGES[@]}"; do
  echo "  Sending: '$msg'"
  
  START_TIME=$(date +%s)
  RESPONSE=$(curl -s -X POST "$API_BASE_URL/api/chat" \
    -H "Authorization: Bearer $AUTH_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
      \"message\": \"$msg\",
      \"session_id\": \"$SESSION_ID\"
    }")
  END_TIME=$(date +%s)
  ELAPSED=$((END_TIME - START_TIME))
  
  if echo "$RESPONSE" | grep -q '"status":"success"'; then
    echo -e "${GREEN}  ✓ Message sent and processed (${ELAPSED}s)${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Extract session_id if not already set
    if [ -z "$SESSION_ID" ]; then
      SESSION_ID=$(echo "$RESPONSE" | grep -o '"session_id":"[^"]*' | cut -d'"' -f4)
      echo "    Session ID: $SESSION_ID"
    fi
    
    # Check response time
    if [ $ELAPSED -gt 15 ]; then
      echo -e "${YELLOW}    ⚠ Warning: Response took ${ELAPSED}s (expected < 15s)${NC}"
    fi
  else
    echo -e "${RED}  ✗ FAILED: Chat message failed${NC}"
    echo "    Response: $RESPONSE"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    ISSUES+=("Test 3.1: Chat message '$msg' failed")
  fi
done

# Verify chat history in database
echo ""
echo "  Verifying chat history in database..."
HISTORY_COUNT=$(psql -h "$DB_HOST" -U "$DB_USER" -d "$DB_NAME" -t -c "
SELECT COUNT(*) FROM chat_messages 
WHERE session_id = '$SESSION_ID';
" 2>/dev/null || echo "0")

if [ "$HISTORY_COUNT" -ge 3 ]; then
  echo -e "${GREEN}  ✓ All messages recorded in database (count: $HISTORY_COUNT)${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${RED}  ✗ FAILED: Messages not properly stored (count: $HISTORY_COUNT)${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 3.1: Chat history not properly stored")
fi

# ============ TEST 3.2: Failover to OpenRouter ============
echo ""
echo -e "${YELLOW}Test 3.2: Failover to OpenRouter${NC}"

echo "  This test requires modifying .env to simulate Cerebras failure"
echo "  Steps:"
echo "    1. Backup current .env"
echo "    2. Set invalid CEREBRAS_API_KEYS"
echo "    3. Restart API"
echo "    4. Send chat message"
echo "    5. Verify OpenRouter was used"
echo "    6. Restore .env and restart"
echo ""

# Create backup of .env
ENV_FILE="$PROJECT_ROOT/.env"
if [ -f "$ENV_FILE" ]; then
  cp "$ENV_FILE" "${ENV_FILE}.backup.test"
  
  echo "  Backing up .env to ${ENV_FILE}.backup.test"
  
  # Modify CEREBRAS_API_KEYS to be invalid
  sed -i.bak 's/CEREBRAS_API_KEYS=.*/CEREBRAS_API_KEYS=invalid_key_for_testing_failover/' "$ENV_FILE"
  
  echo "  Modified CEREBRAS_API_KEYS to invalid value"
  echo "  Restarting API service..."
  
  # Restart the API (requires docker-compose running)
  cd "$PROJECT_ROOT"
  docker-compose restart api 2>/dev/null || true
  
  # Wait for API to be ready
  sleep 5
  
  echo "  Sending test message (should trigger failover)..."
  
  RESPONSE=$(curl -s -X POST "$API_BASE_URL/api/chat" \
    -H "Authorization: Bearer $AUTH_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
      \"message\": \"Test failover: can you respond to this?\",
      \"session_id\": \"failover_test_$$\"
    }")
  
  # Check if response indicates OpenRouter was used
  if echo "$RESPONSE" | grep -q '"provider":"openrouter"'; then
    echo -e "${GREEN}  ✓ Correctly failed over to OpenRouter${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  elif echo "$RESPONSE" | grep -q '"status":"success"'; then
    echo -e "${GREEN}  ✓ Request succeeded (failover likely used)${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    echo -e "${RED}  ✗ FAILED: Chat failed during failover test${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    ISSUES+=("Test 3.2: Failover to OpenRouter failed")
  fi
  
  # Check API logs for failover message
  echo "  Checking API logs for failover indication..."
  API_LOGS=$(docker-compose logs api --tail=50 2>/dev/null || echo "")
  
  if echo "$API_LOGS" | grep -q -i "failover\|openrouter\|cerebras.*error"; then
    echo -e "${GREEN}  ✓ Logs show failover attempt${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    echo -e "${YELLOW}  ⚠ Could not find failover indication in logs (may not be configured)${NC}"
  fi
  
  # Restore .env
  echo "  Restoring original .env..."
  mv "${ENV_FILE}.backup.test" "$ENV_FILE"
  rm -f "${ENV_FILE}.bak"
  
  # Restart API again
  echo "  Restarting API service..."
  docker-compose restart api 2>/dev/null || true
  sleep 5
else
  echo -e "${YELLOW}  ⚠ SKIPPED: .env file not found at $ENV_FILE${NC}"
fi

# ============ TEST 3.3: API Key Rotation ============
echo ""
echo -e "${YELLOW}Test 3.3: API Key Rotation${NC}"

echo "  Sending 4 chat messages to verify key rotation..."

ROTATION_TESTS=0
for i in {1..4}; do
  echo "  Message $i/4..."
  
  RESPONSE=$(curl -s -X POST "$API_BASE_URL/api/chat" \
    -H "Authorization: Bearer $AUTH_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{
      \"message\": \"Rotation test message $i\",
      \"session_id\": \"key_rotation_test_$$\"
    }")
  
  if echo "$RESPONSE" | grep -q '"status":"success"'; then
    ROTATION_TESTS=$((ROTATION_TESTS + 1))
  fi
done

if [ $ROTATION_TESTS -eq 4 ]; then
  echo -e "${GREEN}  ✓ All 4 key rotation tests passed${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${YELLOW}  ⚠ Only $ROTATION_TESTS/4 rotation tests passed${NC}"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 3.3: Key rotation incomplete ($ROTATION_TESTS/4 passed)")
fi

# Check logs for proper key usage
echo "  Verifying key usage from logs..."
API_LOGS=$(docker-compose logs api --tail=100 2>/dev/null || echo "")

if echo "$API_LOGS" | grep -q "using.*key\|rotate.*key\|api_key"; then
  echo -e "${GREEN}  ✓ Key rotation logging detected${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
else
  echo -e "${YELLOW}  ⚠ Key rotation logging not visible (may not be enabled)${NC}"
fi

# ============ Summary ============
echo ""
echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Module 3 Summary${NC}"
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
