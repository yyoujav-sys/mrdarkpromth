#!/bin/bash
# Test Script: Malicious .exe upload attempt (targeted via Docker Exec)
set -euo pipefail

# TARGET is the internal service name in the docker network
TARGET="http://api:8080"
DOCKER_CMD="docker exec mr_darkpromth_nginx"

TIMESTAMP=$(date +%s)
TEST_USER="malicious_${TIMESTAMP}"
TEST_EMAIL="${TEST_USER}@evil.com"
TEST_PASS="EvilPass123!"

echo "--- STEP 1: Register ---"
REG=$($DOCKER_CMD curl -s -X POST "$TARGET/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$TEST_USER\",\"email\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASS\"}")
echo "$REG"

echo "--- STEP 2: Login ---"
LOGIN=$($DOCKER_CMD curl -s -X POST "$TARGET/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASS\"}")
TOKEN=$(echo "$LOGIN" | python3 -c "import sys,json; print(json.load(sys.stdin)['data']['token'])" 2>/dev/null || echo "")

if [ -z "$TOKEN" ]; then echo "Login failed"; exit 1; fi

echo "--- STEP 3: Generate QR/Reference ---"
PLAN_ID="ae67a09c-e6b7-4471-aa0d-239e9f855c88" # Ultra Plan
QR_RESP=$($DOCKER_CMD curl -s -X POST "$TARGET/api/billing/generate-qr" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"plan_id\":\"$PLAN_ID\"}")
REF=$(echo "$QR_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['data']['reference'])" 2>/dev/null || echo "")

if [ -z "$REF" ]; then echo "QR Gen failed"; echo "$QR_RESP"; exit 1; fi

echo "--- STEP 4: Upload .EXE (Disguised as base64) ---"
EXE_BASE64="data:application/x-msdownload;base64,TVqQAAMAAAAEAAAA//8AALgAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgAAAAA4fug4AtAnNIbgBTM0hVGhpcyBwcm9ncmFtIGNhbm5vdCBiZSBydW4gaW4gRE9TIG1vZGUuDQ0KJAAAAAAAAABQRQAATAEDAAAAAAAAAAAAAAAAUEAAAwAAAA..."

UPLOAD_RESP=$($DOCKER_CMD curl -s -X POST "$TARGET/api/billing/verify-slip" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"reference\":\"$REF\",\"slip_image\":\"$EXE_BASE64\"}")

echo "Response: $UPLOAD_RESP"

if echo "$UPLOAD_RESP" | grep -q "400" || echo "$UPLOAD_RESP" | grep -q "INVALID_FILE_TYPE"; then
  echo "SUCCESS: Server rejected .exe file"
else
  VID=$(echo "$UPLOAD_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['data']['id'])" 2>/dev/null || echo "")
  if [ -n "$VID" ]; then
    echo "CRITICAL FAILURE: Server accepted .exe file (Verification ID: $VID)"
  else
    echo "ERROR: Unexpected response format"
  fi
fi
