#!/bin/bash

# Comprehensive API endpoint testing script
# Tests all critical API endpoints for functionality and error handling
# Usage: bash test_api_endpoints.sh [BASE_URL]

BASE_URL="${1:-http://localhost:8080}"
TIMESTAMP=$(date +%s)
TEST_RESULTS_FILE="/tmp/api_test_results_${TIMESTAMP}.txt"
TEST_USER_EMAIL="test_${TIMESTAMP}@example.com"
TEST_USER_PASS="SecureTestPass123!@#"
TEST_JWT_TOKEN=""

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Metrics
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to print test results
print_result() {
    local test_name=$1
    local passed=$2
    local expected=$3
    local actual=$4
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$passed" = true ]; then
        echo -e "${GREEN}✅ PASS${NC}: $test_name"
        echo "PASS: $test_name - Expected: $expected, Got: $actual" >> "$TEST_RESULTS_FILE"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}❌ FAIL${NC}: $test_name"
        echo "FAIL: $test_name - Expected: $expected, Got: $actual" >> "$TEST_RESULTS_FILE"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
}

# Test: Health endpoint
test_health_endpoint() {
    echo ""
    echo -e "${BLUE}=== Testing Health Endpoint ===${NC}"
    
    local response=$(curl -s -w "\n%{http_code}" "$BASE_URL/health")
    local body=$(echo "$response" | head -n -1)
    local status=$(echo "$response" | tail -n 1)
    
    [ "$status" = "200" ] && print_result "Health endpoint returns 200" true "200" "$status" || print_result "Health endpoint returns 200" false "200" "$status"
    
    echo "$body" | grep -q "healthy" && print_result "Health response contains 'healthy'" true "contains" "found" || print_result "Health response contains 'healthy'" false "contains" "not found"
}

# Test: Root endpoint
test_root_endpoint() {
    echo ""
    echo -e "${BLUE}=== Testing Root Endpoint ===${NC}"
    
    local response=$(curl -s -w "\n%{http_code}" "$BASE_URL/")
    local status=$(echo "$response" | tail -n 1)
    
    [ "$status" = "200" ] && print_result "Root endpoint returns 200" true "200" "$status" || print_result "Root endpoint returns 200" false "200" "$status"
}

# Test: Register endpoint
test_register_endpoint() {
    echo ""
    echo -e "${BLUE}=== Testing Registration Endpoint ===${NC}"
    
    local response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/auth/register" \
        -H "Content-Type: application/json" \
        -d "{
            \"username\": \"testuser${TIMESTAMP}\",
            \"email\": \"${TEST_USER_EMAIL}\",
            \"password\": \"${TEST_USER_PASS}\"
        }")
    
    local status=$(echo "$response" | tail -n 1)
    local body=$(echo "$response" | head -n -1)
    
    if [ "$status" = "200" ] || [ "$status" = "201" ]; then
        print_result "Register endpoint succeeds" true "200/201" "$status"
        echo "$body" | grep -q "user_id" && print_result "Register response contains user_id" true "contains" "found" || print_result "Register response contains user_id" false "contains" "not found"
    else
        print_result "Register endpoint succeeds" false "200/201" "$status"
    fi
}

# Test: Login endpoint
test_login_endpoint() {
    echo ""
    echo -e "${BLUE}=== Testing Login Endpoint ===${NC}"
    
    # First create a test user
    curl -s -X POST "$BASE_URL/api/auth/register" \
        -H "Content-Type: application/json" \
        -d "{
            \"username\": \"logintest${TIMESTAMP}\",
            \"email\": \"login${TIMESTAMP}@example.com\",
            \"password\": \"${TEST_USER_PASS}\"
        }" > /dev/null
    
    # Small delay to ensure user is created
    sleep 1
    
    # Then try login
    local response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{
            \"email\": \"login${TIMESTAMP}@example.com\",
            \"password\": \"${TEST_USER_PASS}\"
        }")
    
    local status=$(echo "$response" | tail -n 1)
    local body=$(echo "$response" | head -n -1)
    
    if [ "$status" = "200" ] || [ "$status" = "201" ]; then
        print_result "Login endpoint succeeds" true "200/201" "$status"
        TEST_JWT_TOKEN=$(echo "$body" | grep -o '"token":"[^"]*' | cut -d':' -f2 | tr -d '"')
        [ -n "$TEST_JWT_TOKEN" ] && print_result "Login returns JWT token" true "token" "found" || print_result "Login returns JWT token" false "token" "not found"
    else
        print_result "Login endpoint succeeds" false "200/201" "$status"
    fi
}

# Test: Invalid credentials
test_invalid_credentials() {
    echo ""
    echo -e "${BLUE}=== Testing Invalid Credentials ===${NC}"
    
    local response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{
            \"email\": \"nonexistent@example.com\",
            \"password\": \"WrongPassword123!\"
        }")
    
    local status=$(echo "$response" | tail -n 1)
    
    [ "$status" = "401" ] && print_result "Invalid credentials return 401" true "401" "$status" || print_result "Invalid credentials return 401" false "401" "$status"
}

# Test: Missing required fields
test_missing_fields() {
    echo ""
    echo -e "${BLUE}=== Testing Missing Required Fields ===${NC}"
    
    local response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/auth/register" \
        -H "Content-Type: application/json" \
        -d "{\"username\": \"testuser\"}")
    
    local status=$(echo "$response" | tail -n 1)
    
    [ "$status" = "400" ] || [ "$status" = "422" ] && print_result "Missing fields return 400/422" true "400/422" "$status" || print_result "Missing fields return 400/422" false "400/422" "$status"
}

# Test: List plans endpoint
test_list_plans() {
    echo ""
    echo -e "${BLUE}=== Testing List Plans Endpoint ===${NC}"
    
    local response=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/billing/plans")
    local status=$(echo "$response" | tail -n 1)
    
    [ "$status" = "200" ] && print_result "List plans returns 200" true "200" "$status" || print_result "List plans returns 200" false "200" "$status"
}

# Test: Metrics endpoint
test_metrics_endpoint() {
    echo ""
    echo -e "${BLUE}=== Testing Metrics Endpoint ===${NC}"
    
    local response=$(curl -s -w "\n%{http_code}" "$BASE_URL/metrics")
    local status=$(echo "$response" | tail -n 1)
    
    [ "$status" = "200" ] && print_result "Metrics endpoint returns 200" true "200" "$status" || print_result "Metrics endpoint returns 200" false "200" "$status"
}

# Test: Rate limiting
test_rate_limiting() {
    echo ""
    echo -e "${BLUE}=== Testing Rate Limiting ===${NC}"
    
    # Make many requests rapidly
    local failed_count=0
    for i in {1..150}; do
        status=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/health")
        if [ "$status" = "429" ]; then
            failed_count=$((failed_count + 1))
        fi
    done
    
    if [ "$failed_count" -gt 5 ]; then
        print_result "Rate limiting is active" true ">5 429s" "$failed_count"
    else
        print_result "Rate limiting is active" false ">5 429s" "$failed_count"
    fi
}

# Test: Error response format
test_error_response_format() {
    echo ""
    echo -e "${BLUE}=== Testing Error Response Format ===${NC}"
    
    local response=$(curl -s -X POST "$BASE_URL/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"email\": \"test@test.com\", \"password\": \"wrong\"}")
    
    echo "$response" | grep -q "error_code" && print_result "Error response has error_code" true "has" "found" || print_result "Error response has error_code" false "has" "not found"
    echo "$response" | grep -q "message" && print_result "Error response has message" true "has" "found" || print_result "Error response has message" false "has" "not found"
    echo "$response" | grep -q "request_id" && print_result "Error response has request_id" true "has" "found" || print_result "Error response has request_id" false "has" "not found"
    echo "$response" | grep -q "timestamp" && print_result "Error response has timestamp" true "has" "found" || print_result "Error response has timestamp" false "has" "not found"
}

# Main execution
main() {
    echo "======================================"
    echo "API Endpoint Testing Suite"
    echo "======================================"
    echo "Base URL: $BASE_URL"
    echo "Test Results Log: $TEST_RESULTS_FILE"
    echo ""
    
    # Initialize results file
    echo "API Test Results - $(date)" > "$TEST_RESULTS_FILE"
    echo "Base URL: $BASE_URL" >> "$TEST_RESULTS_FILE"
    echo "======================================" >> "$TEST_RESULTS_FILE"
    
    # Run tests
    test_health_endpoint
    test_root_endpoint
    test_register_endpoint
    test_login_endpoint
    test_invalid_credentials
    test_missing_fields
    test_list_plans
    test_metrics_endpoint
    test_error_response_format
    # test_rate_limiting  # Commented out as it may impact other tests
    
    # Print summary
    echo ""
    echo "======================================"
    echo "Test Summary"
    echo "======================================"
    echo -e "Total Tests:  $TOTAL_TESTS"
    echo -e "${GREEN}Passed:       $PASSED_TESTS${NC}"
    echo -e "${RED}Failed:       $FAILED_TESTS${NC}"
    
    if [ "$FAILED_TESTS" -eq 0 ]; then
        echo -e "${GREEN}✅ All tests passed!${NC}"
        return 0
    else
        echo -e "${RED}❌ Some tests failed${NC}"
        return 1
    fi
}

# Execute main
main
exit $?
