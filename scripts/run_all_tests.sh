#!/bin/bash

# Source utilities for colors
SCRIPT_DIR="$(dirname "$0")"
source "$SCRIPT_DIR/test_utils.sh"

echo "========================================"
echo "STARTING COMPREHENSIVE TEST SUITE"
echo "========================================"

# Make scripts executable
chmod +x "$SCRIPT_DIR/../tests/api/"*.sh

FAILURES=0

run_test() {
    local script=$1
    echo
    echo "Running $script..."
    if "$script"; then
        echo -e "${GREEN}[SUCCESS]${NC} $script passed."
    else
        echo -e "${RED}[FAILURE]${NC} $script failed."
        FAILURES=$((FAILURES + 1))
    fi
}

# Run tests in order
run_test "$SCRIPT_DIR/../tests/api/01_auth_tests.sh"
run_test "$SCRIPT_DIR/../tests/api/02_endpoints_tests.sh"
run_test "$SCRIPT_DIR/../tests/api/03_features_tests.sh"

echo
echo "========================================"
if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}ALL TESTS PASSED!${NC}"
    exit 0
else
    echo -e "${RED}$FAILURES TESTS FAILED.${NC}"
    exit 1
fi
