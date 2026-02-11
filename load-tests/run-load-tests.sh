#!/bin/bash
# Load Test Runner Script
# Runs k6 load tests with various configurations

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESULTS_DIR="${SCRIPT_DIR}/results"
BASE_URL="${LOAD_TEST_BASE_URL:-http://localhost:8080}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== MR.DarkPromth Load Testing ===${NC}"
echo "Base URL: $BASE_URL"
echo "Results: $RESULTS_DIR"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Check if k6 is installed
if ! command -v k6 &> /dev/null; then
    echo -e "${RED}k6 is not installed. Installing...${NC}"
    
    # Try to install k6
    if command -v apt-get &> /dev/null; then
        sudo gpg --no-default-keyring --keyring /usr/share/keyrings/k6-archive-keyring.gpg --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
        echo "deb [signed-by=/usr/share/keyrings/k6-archive-keyring.gpg] https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
        sudo apt-get update
        sudo apt-get install -y k6
    elif command -v brew &> /dev/null; then
        brew install k6
    else
        echo -e "${RED}Please install k6 manually: https://k6.io/docs/getting-started/installation/${NC}"
        exit 1
    fi
fi

run_test() {
    local test_name=$1
    local test_file=$2
    local extra_args=${3:-}
    
    echo -e "\n${YELLOW}Running: $test_name${NC}"
    echo "Config: $extra_args"
    
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local output_file="${RESULTS_DIR}/${test_name}_${timestamp}.json"
    
    k6 run \
        --out json="$output_file" \
        --env LOAD_TEST_BASE_URL="$BASE_URL" \
        $extra_args \
        "$test_file"
    
    echo -e "${GREEN}Results saved to: $output_file${NC}"
}

# Main menu
case "${1:-menu}" in
    quick)
        echo -e "${BLUE}Running quick smoke test...${NC}"
        run_test "quick" "${SCRIPT_DIR}/load-test-api.js" "--vus 5 --duration 30s"
        ;;
    
    smoke)
        echo -e "${BLUE}Running smoke test...${NC}"
        run_test "smoke" "${SCRIPT_DIR}/load-test-api.js" "--vus 1 --duration 1m"
        ;;
    
    load)
        echo -e "${BLUE}Running load test (20-50 VUs, 10 min)...${NC}"
        run_test "load" "${SCRIPT_DIR}/load-test-api.js" ""
        ;;
    
    stress)
        echo -e "${BLUE}Running stress test (100-200 VUs, 16 min)...${NC}"
        run_test "stress" "${SCRIPT_DIR}/load-test-api.js" ""
        ;;
    
    spike)
        echo -e "${BLUE}Running spike test (300 VUs spike)...${NC}"
        run_test "spike" "${SCRIPT_DIR}/load-test-api.js" ""
        ;;
    
    sandbox)
        echo -e "${BLUE}Running sandbox execution test...${NC}"
        run_test "sandbox" "${SCRIPT_DIR}/sandbox-execution.js" "--vus 10 --duration 2m"
        ;;
    
    full)
        echo -e "${BLUE}Running full test suite...${NC}"
        run_test "full" "${SCRIPT_DIR}/load-test-api.js" ""
        ;;
    
    menu|*)
        echo "Usage: $0 {quick|smoke|load|stress|spike|sandbox|full}"
        echo ""
        echo "Test Types:"
        echo "  quick   - Quick 30s test with 5 VUs"
        echo "  smoke   - 1 minute smoke test"
        echo "  load    - 10 minute load test (20-50 VUs)"
        echo "  stress  - 16 minute stress test (100-200 VUs)"
        echo "  spike   - Spike test (sudden 300 VU surge)"
        echo "  sandbox - Sandbox execution load test"
        echo "  full    - Full test suite with all scenarios"
        echo ""
        echo "Environment Variables:"
        echo "  LOAD_TEST_BASE_URL  - Target URL (default: http://localhost:8080)"
        echo "  SANDBOX_AUTH_TOKEN  - Auth token for sandbox tests"
        ;;
esac

echo -e "\n${BLUE}=== Load Testing Complete ===${NC}"
