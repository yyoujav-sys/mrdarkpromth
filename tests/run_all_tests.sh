#!/bin/bash

# Production Readiness Test Suite - Master Runner
# ================================================

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
BRIGHT_WHITE='\033[1;37m'
NC='\033[0m'

# Test configuration
API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"
VERBOSE="${VERBOSE:-false}"
SKIP_MODULES="${SKIP_MODULES:-}"

# Results
TOTAL_PASSED=0
TOTAL_FAILED=0
MODULE_RESULTS=()

echo -e "${CYAN}"
cat << "EOF"
╔════════════════════════════════════════════════════════════╗
║   Production Readiness Test Suite - Complete Day          ║
║   Estimated Duration: 6.5 hours                           ║
╚════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"
echo ""

# Pre-test checks
echo -e "${BLUE}=== Pre-Test Checks ===${NC}"
echo ""

echo -e "${YELLOW}Checking prerequisites...${NC}"

# Check Docker
if command -v docker &> /dev/null; then
  echo -e "${GREEN}✓ Docker is installed${NC}"
else
  echo -e "${RED}✗ Docker not installed${NC}"
  echo "Continuing anyway..."
fi

# Check docker-compose
if command -v docker-compose &> /dev/null; then
  echo -e "${GREEN}✓ docker-compose is installed${NC}"
else
  echo -e "${RED}✗ docker-compose not installed${NC}"
  echo "Continuing anyway..."
fi

# Check API connectivity
echo "  Testing API connectivity..."
if curl -s -f "$API_BASE_URL/health" > /dev/null 2>&1; then
  echo -e "${GREEN}✓ API is responding at $API_BASE_URL${NC}"
else
  echo -e "${RED}✗ API is not responding at $API_BASE_URL${NC}"
  echo "  Make sure to run: cd $PROJECT_ROOT && docker-compose up -d"
  echo "Continuing anyway..."
fi

# Check database
echo "  Testing database connectivity..."
if command -v psql &> /dev/null; then
  if psql -h localhost -U postgres -d mr_darkpromth -t -c "SELECT 1" > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Database is accessible${NC}"
  else
    echo -e "${YELLOW}⚠ Database may not be accessible (will skip DB tests)${NC}"
  fi
else
  echo -e "${YELLOW}⚠ psql not installed (will skip direct DB tests)${NC}"
fi

echo ""

# Module 1: Payment System
if [[ ! "$SKIP_MODULES" =~ "1" ]]; then
  echo -e "${MAGENTA}╔════════════════════════════════════════════════════════════╗${NC}"
  echo -e "${MAGENTA}║ Module 1: Payment System & Tier Upgrade                  ║${NC}"
  echo -e "${MAGENTA}╚════════════════════════════════════════════════════════════╝${NC}"
  echo ""
  
  START_TIME=$(date +%s)
  
  if bash "$SCRIPT_DIR/test_module_1_payment.sh"; then
    RESULT="PASS"
    echo -e "${GREEN}✓ Module 1 tests passed${NC}"
    TOTAL_PASSED=$((TOTAL_PASSED + 1))
  else
    RESULT="FAIL"
    echo -e "${RED}✗ Module 1 tests failed${NC}"
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
  fi
  
  END_TIME=$(date +%s)
  DURATION=$((END_TIME - START_TIME))
  
  MODULE_RESULTS+=("Module 1 (Payment): $RESULT (${DURATION}s)")
  echo ""
else
  echo -e "${YELLOW}⊘ Module 1 skipped${NC}"
  echo ""
fi

# Module 2: Terminal & Sandbox
if [[ ! "$SKIP_MODULES" =~ "2" ]]; then
  echo -e "${MAGENTA}╔════════════════════════════════════════════════════════════╗${NC}"
  echo -e "${MAGENTA}║ Module 2: Ultra Terminal & Sandbox                        ║${NC}"
  echo -e "${MAGENTA}╚════════════════════════════════════════════════════════════╝${NC}"
  echo ""
  
  if [ -z "$AUTH_TOKEN" ]; then
    echo -e "${YELLOW}⚠ AUTH_TOKEN not set. Skipping Module 2.${NC}"
    echo "  To enable: export AUTH_TOKEN=your_bearer_token"
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
    MODULE_RESULTS+=("Module 2 (Terminal): SKIP (no AUTH_TOKEN)")
  else
    START_TIME=$(date +%s)
    
    if AUTH_TOKEN="$AUTH_TOKEN" bash "$SCRIPT_DIR/test_module_2_terminal.sh"; then
      RESULT="PASS"
      echo -e "${GREEN}✓ Module 2 tests passed${NC}"
      TOTAL_PASSED=$((TOTAL_PASSED + 1))
    else
      RESULT="FAIL"
      echo -e "${RED}✗ Module 2 tests failed${NC}"
      TOTAL_FAILED=$((TOTAL_FAILED + 1))
    fi
    
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    
    MODULE_RESULTS+=("Module 2 (Terminal): $RESULT (${DURATION}s)")
  fi
  
  echo ""
else
  echo -e "${YELLOW}⊘ Module 2 skipped${NC}"
  echo ""
fi

# Module 3: Chat & Failover
if [[ ! "$SKIP_MODULES" =~ "3" ]]; then
  echo -e "${MAGENTA}╔════════════════════════════════════════════════════════════╗${NC}"
  echo -e "${MAGENTA}║ Module 3: Chat & AI Failover                             ║${NC}"
  echo -e "${MAGENTA}╚════════════════════════════════════════════════════════════╝${NC}"
  echo ""
  
  if [ -z "$AUTH_TOKEN" ]; then
    echo -e "${YELLOW}⚠ AUTH_TOKEN not set. Skipping Module 3.${NC}"
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
    MODULE_RESULTS+=("Module 3 (Chat): SKIP (no AUTH_TOKEN)")
  else
    START_TIME=$(date +%s)
    
    if AUTH_TOKEN="$AUTH_TOKEN" bash "$SCRIPT_DIR/test_module_3_chat.sh"; then
      RESULT="PASS"
      echo -e "${GREEN}✓ Module 3 tests passed${NC}"
      TOTAL_PASSED=$((TOTAL_PASSED + 1))
    else
      RESULT="FAIL"
      echo -e "${RED}✗ Module 3 tests failed${NC}"
      TOTAL_FAILED=$((TOTAL_FAILED + 1))
    fi
    
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    
    MODULE_RESULTS+=("Module 3 (Chat): $RESULT (${DURATION}s)")
  fi
  
  echo ""
else
  echo -e "${YELLOW}⊘ Module 3 skipped${NC}"
  echo ""
fi

# Module 4: Security & Load Testing
if [[ ! "$SKIP_MODULES" =~ "4" ]]; then
  echo -e "${MAGENTA}╔════════════════════════════════════════════════════════════╗${NC}"
  echo -e "${MAGENTA}║ Module 4: Security & Load Testing                        ║${NC}"
  echo -e "${MAGENTA}╚════════════════════════════════════════════════════════════╝${NC}"
  echo ""
  
  START_TIME=$(date +%s)
  
  if bash "$SCRIPT_DIR/test_module_4_security.sh"; then
    RESULT="PASS"
    echo -e "${GREEN}✓ Module 4 tests passed${NC}"
    TOTAL_PASSED=$((TOTAL_PASSED + 1))
  else
    RESULT="FAIL"
    echo -e "${RED}✗ Module 4 tests failed${NC}"
    TOTAL_FAILED=$((TOTAL_FAILED + 1))
  fi
  
  END_TIME=$(date +%s)
  DURATION=$((END_TIME - START_TIME))
  
  MODULE_RESULTS+=("Module 4 (Security): $RESULT (${DURATION}s)")
  echo ""
else
  echo -e "${YELLOW}⊘ Module 4 skipped${NC}"
  echo ""
fi

# Final Summary
echo -e "${CYAN}"
cat << "EOF"
╔════════════════════════════════════════════════════════════╗
║         PRODUCTION READINESS TEST - FINAL REPORT          ║
╚════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"
echo ""

echo -e "${BRIGHT_WHITE}Test Results by Module:${NC}"
echo ""
for result in "${MODULE_RESULTS[@]}"; do
  if [[ $result == *"PASS"* ]]; then
    echo -e "  ${GREEN}✓ $result${NC}"
  elif [[ $result == *"FAIL"* ]]; then
    echo -e "  ${RED}✗ $result${NC}"
  elif [[ $result == *"SKIP"* ]]; then
    echo -e "  ${YELLOW}⊘ $result${NC}"
  fi
done

echo ""
echo -e "${BRIGHT_WHITE}Overall Statistics:${NC}"
echo ""
echo -e "  Modules Passed:  ${GREEN}$TOTAL_PASSED${NC}"
echo -e "  Modules Failed:  ${RED}$TOTAL_FAILED${NC}"

if [ $TOTAL_FAILED -eq 0 ]; then
  echo ""
  echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
  echo -e "${GREEN}  ✓ ALL TESTS PASSED - SYSTEM READY FOR PRODUCTION${NC}"
  echo -e "${GREEN}════════════════════════════════════════════════════════════${NC}"
  EXIT_CODE=0
else
  echo ""
  echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
  echo -e "${RED}  ✗ SOME TESTS FAILED - REVIEW ISSUES BEFORE DEPLOYING${NC}"
  echo -e "${RED}════════════════════════════════════════════════════════════${NC}"
  EXIT_CODE=1
fi

echo ""
echo -e "${YELLOW}Next Steps:${NC}"
if [ $TOTAL_FAILED -eq 0 ]; then
  echo "  1. ✓ All tests passed"
  echo "  2. Review PRODUCTION_READINESS_TEST.md for details"
  echo "  3. Create backup: ./backup.sh"
  echo "  4. Deploy to production: ./deploy_production.sh"
  echo "  5. Monitor: docker-compose logs -f api"
else
  echo "  1. Review test output above for failures"
  echo "  2. Fix identified issues"
  echo "  3. Re-run tests: bash tests/run_all_tests.sh"
  echo "  4. Once all pass, proceed with deployment"
fi

echo ""

exit $EXIT_CODE
