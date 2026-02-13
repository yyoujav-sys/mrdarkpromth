#!/bin/bash

# Configuration
BASE_URL="http://localhost"
TEMP_DIR="/tmp/mrdarkpromth_tests"
mkdir -p "$TEMP_DIR"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper Functions

log_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

log_pass() {
    echo -e "${GREEN}[PASS]${NC} $1"
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    exit 1
}

# Function to extract JSON field using python (to avoid jq dependency if missing, but we prefer jq)
get_json_field() {
    echo "$1" | grep -o "\"$2\":[^,}]*" | cut -d: -f2 | tr -d '" '
}

# Function to assert HTTP status code
assert_status() {
    local response_file=$1
    local expected_status=$2
    local message=$3
    
    # Extract status code from the last line of the response file (curl -w "%{http_code}")
    # We expect the caller to append the status code to the file
    local actual_status=$(tail -n1 "$response_file")
    
    if [ "$actual_status" == "$expected_status" ]; then
        log_pass "$message (Status: $actual_status)"
    else
        log_fail "$message (Expected: $expected_status, Got: $actual_status)"
        cat "$response_file"
    fi
}

# Login helper
# Usage: login_user "email" "password" "token_file_name"
login_user() {
    local email=$1
    local password=$2
    local token_file="$TEMP_DIR/$3"
    
    log_info "Logging in as $email..."
    
    response=$(curl -s -X POST "$BASE_URL/api/auth/login" \
        -H "Content-Type: application/json" \
        -d "{\"email\":\"$email\",\"password\":\"$password\"}")
        
    if [[ $response == *"token"* ]]; then
        # Use jq if available, otherwise simple grep/cut
        if command -v jq &> /dev/null; then
            echo "$response" | jq -r '.data.token' > "$token_file"
        else
            echo "$response" | grep -o '"token":"[^"]*"' | cut -d'"' -f4 > "$token_file"
        fi
        log_pass "Login successful for $email"
    else
        log_fail "Login failed for $email. Response: $response"
    fi
}
