#!/bin/bash
# Sandbox Hardening Verification Script

API_URL="http://localhost:80"
EMAIL="sandbox_test_$(date +%s)@test.com"
PASSWORD="securePassword123"
USERNAME="sandbox_test_$(date +%s)"

echo "--- STEP 1: Register ---"
REGISTER_RESP=$(curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}")
USER_ID=$(echo $REGISTER_RESP | jq -r .data.user_id)

if [ "$USER_ID" == "null" ] || [ -z "$USER_ID" ]; then
    echo "Registration failed: $REGISTER_RESP"
    exit 1
fi
echo "User ID: $USER_ID"

echo -e "\n--- STEP 2: Upgrade to Ultra (Internal DB Update) ---"
docker exec mr_darkpromth_postgres psql -U postgres -d mr_darkpromth -c "UPDATE users SET tier = 'ultra' WHERE id = '$USER_ID';"

echo -e "\n--- STEP 3: Login to get fresh Ultra token ---"
LOGIN_RESP=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}")
TOKEN=$(echo $LOGIN_RESP | jq -r .data.token)

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
    echo "Login failed: $LOGIN_RESP"
    exit 1
fi

echo -e "\n--- STEP 4: Verify UID 65534 (nobody) ---"
UID_RESP=$(curl -s -X POST "$API_URL/api/terminal/ultra/execute" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"command\": \"id\", \"args\": [\"-u\"]}")
echo "UID Response: $UID_RESP"
ACTUAL_UID=$(echo $UID_RESP | jq -r .data.stdout | tr -d '\n' | tr -d ' ')

if [ "$ACTUAL_UID" == "65534" ]; then
    echo "SUCCESS: Process is running as 'nobody' (UID 65534)."
else
    echo "FAILURE: Process is running as UID $ACTUAL_UID (expected 65534)."
fi

echo -e "\n--- STEP 5: Spawn Long-Running Process in Sandbox ---"
SESSION_ID="test-session-$(date +%s)"
echo "Starting 'sleep 300' in background with session $SESSION_ID..."
curl -s -X POST "$API_URL/api/terminal/ultra/execute" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"command\": \"sleep\", \"args\": [\"300\"], \"session_id\": \"$SESSION_ID\", \"timeout_seconds\": 300}" &

SLEEP_CURL_PID=$!
sleep 3

echo -e "\n--- STEP 6: Check if process exists in API container ---"
PID_INFO=$(docker exec mr_darkpromth_api ps aux | grep "sleep 300" | grep -v grep)
if [ -n "$PID_INFO" ]; then
    echo "Process Found: $PID_INFO"
else
    echo "FAILURE: 'sleep 300' not found in api container."
fi

echo -e "\n--- STEP 7: Remove Session (Trigger Cleanup) ---"
echo "Calling DELETE /api/sandbox/sessions/$SESSION_ID"
curl -s -X DELETE "$API_URL/api/sandbox/sessions/$SESSION_ID" \
  -H "Authorization: Bearer $TOKEN"

echo -e "\n--- STEP 8: Verify Process is REAPED ---"
sleep 3
STILL_HERE=$(docker exec mr_darkpromth_api ps aux | grep "sleep 300" | grep -v grep)

if [ -z "$STILL_HERE" ]; then
    echo "SUCCESS: Process was correctly reaped!"
else
    echo "FAILURE: Process is still alive after session removal!"
    echo "$STILL_HERE"
fi

kill $SLEEP_CURL_PID 2>/dev/null
