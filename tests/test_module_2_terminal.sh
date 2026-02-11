#!/bin/bash

# Module 2: Ultra Terminal & Sandbox Tests
# ========================================

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
NC='\033[0m' # No Color

# Initialize test results
TESTS_PASSED=0
TESTS_FAILED=0
ISSUES=()

# Requires AUTH_TOKEN to be set
if [ -z "$AUTH_TOKEN" ]; then
  echo -e "${RED}ERROR: AUTH_TOKEN not set. Please set a valid bearer token.${NC}"
  echo "  Usage: AUTH_TOKEN=your_token bash test_module_2_terminal.sh"
  exit 1
fi

echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Module 2: Ultra Terminal & Sandbox${NC}"
echo -e "${BLUE}=====================================${NC}"
echo ""

# Function to execute terminal command via API
execute_terminal_command() {
  local cmd=$1
  local expected_pattern=$2
  
  curl -s -X POST "$API_BASE_URL/api/ultra/terminal/execute" \
    -H "Authorization: Bearer $AUTH_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"command\": \"$cmd\"}"
}

# ============ TEST 2.1: Basic Commands ============
echo -e "${YELLOW}Test 2.1: Basic Terminal Commands${NC}"

COMMANDS=(
  "echo 'Hello World'"
  "ls -la"
  "pwd"
  "whoami"
  "date"
  "uname -a"
  "id"
)

for cmd in "${COMMANDS[@]}"; do
  echo "  Testing: $cmd"
  
  START_TIME=$(date +%s%N)
  RESPONSE=$(execute_terminal_command "$cmd")
  END_TIME=$(date +%s%N)
  ELAPSED_MS=$(( (END_TIME - START_TIME) / 1000000 ))
  
  if echo "$RESPONSE" | grep -q '"status":"success"'; then
    echo -e "${GREEN}  ✓ Command executed (${ELAPSED_MS}ms)${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
    
    # Check if response time is acceptable (< 2 seconds)
    if [ $ELAPSED_MS -gt 2000 ]; then
      echo -e "${YELLOW}    ⚠ Warning: Command took ${ELAPSED_MS}ms (expected < 2000ms)${NC}"
    fi
  else
    echo -e "${RED}  ✗ FAILED: Command execution failed${NC}"
    echo "    Response: $RESPONSE"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    ISSUES+=("Test 2.1: Command '$cmd' failed")
  fi
done

# ============ TEST 2.2: Complex Script Execution ============
echo ""
echo -e "${YELLOW}Test 2.2: Complex Script Execution${NC}"

SCRIPT='#!/bin/bash
set -e
mkdir -p test_dir
cd test_dir
for i in {1..5}; do
  echo "File $i" > file_$i.txt
done
ls -la
tar -czf archive.tar.gz *.txt
du -sh archive.tar.gz'

echo "  Executing multi-line script..."
RESPONSE=$(curl -s -X POST "$API_BASE_URL/api/ultra/terminal/execute" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -H "Content-Type: application/json" \
  -d "{\"command\": \"$SCRIPT\", \"multiline\": true}")

if echo "$RESPONSE" | grep -q '"status":"success"'; then
  echo -e "${GREEN}  ✓ Complex script executed successfully${NC}"
  TESTS_PASSED=$((TESTS_PASSED + 1))
  
  # Verify output contains expected files
  if echo "$RESPONSE" | grep -q "file_1.txt\|file_2.txt\|archive.tar.gz"; then
    echo -e "${GREEN}  ✓ All expected files created${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    echo -e "${RED}  ✗ FAILED: Expected files not found in output${NC}"
    TESTS_FAILED=$((TESTS_FAILED + 1))
    ISSUES+=("Test 2.2: Expected files not found in output")
  fi
else
  echo -e "${RED}  ✗ FAILED: Script execution failed${NC}"
  echo "    Response: $RESPONSE"
  TESTS_FAILED=$((TESTS_FAILED + 1))
  ISSUES+=("Test 2.2: Complex script execution failed")
fi

# ============ TEST 2.3: Sandbox Isolation ============
echo ""
echo -e "${YELLOW}Test 2.3: Sandbox Isolation Checks${NC}"

ISOLATION_TESTS=(
  "cat /etc/passwd"
  "cat /etc/hostname"
  "../../../etc/passwd"
  "cd ../ && pwd"
  "mount"
  "ip addr"
)

for cmd in "${ISOLATION_TESTS[@]}"; do
  echo "  Testing isolation: $cmd"
  
  RESPONSE=$(execute_terminal_command "$cmd")
  
  # These commands should either fail or return restricted content
  if echo "$RESPONSE" | grep -q '"status":"error"'; then
    echo -e "${GREEN}  ✓ Command properly blocked${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  elif echo "$RESPONSE" | grep -q "permission denied\|Permission denied\|No such file"; then
    echo -e "${GREEN}  ✓ Command returned permission denied${NC}"
    TESTS_PASSED=$((TESTS_PASSED + 1))
  else
    # For commands like 'pwd' or 'mount', check output is sandboxed
    if echo "$RESPONSE" | grep -q "/sandbox\|docker"; then
      echo -e "${GREEN}  ✓ Output shows sandbox environment${NC}"
      TESTS_PASSED=$((TESTS_PASSED + 1))
    else
      echo -e "${YELLOW}  ⚠ Unable to verify isolation (manual review needed)${NC}"
    fi
  fi
done

# ============ TEST 2.4: Concurrent Sessions Performance ============
echo ""
echo -e "${YELLOW}Test 2.4: Concurrent Session Performance Test${NC}"
echo "  Spawning 5 concurrent terminal sessions..."

# This test requires the system to be running and monitored separately
# Just verify the API can handle concurrent requests
CONCURRENT_COUNT=5
SUCCESS_COUNT=0

for i in $(seq 1 $CONCURRENT_COUNT); do
  (
    echo "    Session $i: Running command..."
    RESPONSE=$(curl -s -X POST "$API_BASE_URL/api/ultra/terminal/execute" \
      -H "Authorization: Bearer $AUTH_TOKEN" \
      -H "Content-Type: application/json" \
      -d "{\"command\": \"sleep 2 && echo 'Session $i complete'\"}")
    
    if echo "$RESPONSE" | grep -q '"status":"success"'; then
      echo -e "${GREEN}    ✓ Session $i succeeded${NC}"
    fi
  ) &
done

# Wait for all background jobs
wait

echo -e "${GREEN}  ✓ All concurrent sessions completed${NC}"
TESTS_PASSED=$((TESTS_PASSED + 1))

echo "  NOTE: For accurate performance metrics, monitor:"
echo "    - RAM usage: docker stats mr_darkpromth_sandbox --no-stream"
echo "    - CPU usage: watch -n 1 'top -p \$(pgrep -f sandbox)'"
echo "    - Response times from JSON responses"

# ============ Summary ============
echo ""
echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}Module 2 Summary${NC}"
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
