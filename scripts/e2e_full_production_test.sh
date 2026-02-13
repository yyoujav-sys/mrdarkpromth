#!/bin/bash
# ═══════════════════════════════════════════════════════════
#  E2E Test: Full Production Integration
#  Covers: Auth, R2 Upload, KV Cache, WebSocket
#  Targets: Nginx (Port 80) -> API (Port 8080)
# ═══════════════════════════════════════════════════════════
set -euo pipefail

BASE_URL="http://localhost"
HOST_HEADER="Host: mrdarkpromth.online"
PROTO_HEADER="X-Forwarded-Proto: https"

PASS=0
FAIL=0

ok()   { PASS=$((PASS+1)); echo "  ✅ $1"; }
fail() { FAIL=$((FAIL+1)); echo "  ❌ $1"; }

TIMESTAMP=$(date +%s)
TEST_USER="prod_test_${TIMESTAMP}"
TEST_EMAIL="${TEST_USER}@test.com"
TEST_PASS="TestPass123!"

echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║   🚀 Full E2E Production Test (Auth, R2, KV, WS)             ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo ""

# Helper for curl with required headers
curl_api() {
  curl -s -H "$HOST_HEADER" -H "$PROTO_HEADER" "$@"
}

# =========================================================
# STEP 1: Auth (Register & Login)
# =========================================================
echo "━━━ STEP 1: Register & Login ━━━"
REG_RESP=$(curl_api -X POST "$BASE_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$TEST_USER\",\"email\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASS\"}")

if echo "$REG_RESP" | python3 -c "import sys,json; d=json.load(sys.stdin); assert 'data' in d" 2>/dev/null; then
  ok "User registered: $TEST_EMAIL"
else
  fail "Registration failed"; echo "  $REG_RESP"; exit 1
fi

LOGIN_RESP=$(curl_api -X POST "$BASE_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$TEST_EMAIL\",\"password\":\"$TEST_PASS\"}")

TOKEN=$(echo "$LOGIN_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['data']['token'])" 2>/dev/null || echo "")

if [ -n "$TOKEN" ]; then
  ok "User logged in. Token acquired."
else
  fail "Login failed"; echo "  $LOGIN_RESP"; exit 1
fi

# =========================================================
# STEP 2: R2 File Upload (Phase 2)
# =========================================================
echo ""
echo "━━━ STEP 2: R2 File Upload ━━━"
echo "This is a test file content for R2 upload verification." > test_upload.txt

UPLOAD_RESP=$(curl_api -X POST "$BASE_URL/api/upload" \
  -H "Authorization: Bearer $TOKEN" \
  -F "file=@test_upload.txt")

FILE_URL=$(echo "$UPLOAD_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['data']['url'])" 2>/dev/null || echo "")

if [ -n "$FILE_URL" ]; then
  ok "File uploaded successfully."
  ok "File URL: $FILE_URL"
  # Clean up local file
  rm test_upload.txt
else
  fail "File upload failed"
  echo "  $UPLOAD_RESP"
  rm test_upload.txt
fi

# =========================================================
# STEP 3: KV Caching (Phase 3)
# =========================================================
echo ""
echo "━━━ STEP 3: KV Caching (Popular Prompts) ━━━"

# First request - should populate cache (or return fresh if not cached yet)
echo "  Fetch 1 (Populate Cache)..."
RESP_1=$(curl_api "$BASE_URL/api/jailbreak/prompts/popular" -H "Authorization: Bearer $TOKEN")
CACHED_1=$(echo "$RESP_1" | python3 -c "import sys,json; d=json.load(sys.stdin); print('yes' if d.get('data',{}).get('cached') else 'no')" 2>/dev/null || echo "no")

if [ "$CACHED_1" = "yes" ]; then
  echo "  ℹ️  Response 1 was already cached. That's fine."
else
  echo "  ℹ️  Response 1 was fresh (not cached)."
fi

# Wait a moment for async cache write
sleep 2

# Second request - should be cached
echo "  Fetch 2 (Expect Cached)..."
RESP_2=$(curl_api "$BASE_URL/api/jailbreak/prompts/popular" -H "Authorization: Bearer $TOKEN")
CACHED_2=$(echo "$RESP_2" | python3 -c "import sys,json; d=json.load(sys.stdin); print('yes' if d.get('data',{}).get('cached') else 'no')" 2>/dev/null || echo "no")

if [ "$CACHED_2" = "yes" ]; then
  ok "KV Caching Verified! Response 2 returned 'cached': true"
else
  fail "KV Caching Failed. Response 2 not cached."
  echo "  NOTE: This might be due to async latency or build not applying KV logic."
fi

# =========================================================
# STEP 4: WebSocket Connectivity (Phase 5)
# =========================================================
echo ""
echo "━━━ STEP 4: WebSocket Connectivity ━━━"
# Nginx /ws/ endpoint

WS_RESP_CODE=$(curl -s -o /dev/null -w "%{http_code}" \
  -H "$HOST_HEADER" \
  -H "$PROTO_HEADER" \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
  -H "Sec-WebSocket-Version: 13" \
  -H "Sec-WebSocket-Protocol: access_token, $TOKEN" \
  "$BASE_URL/api/ws/events")

echo "  WS Handshake HTTP Code: $WS_RESP_CODE"

if [ "$WS_RESP_CODE" = "101" ]; then
  ok "WebSocket Handshake Successful (HTTP 101)"
elif [ "$WS_RESP_CODE" = "426" ]; then
  fail "WebSocket Returned 426 (Upgrade Required)"
elif [ "$WS_RESP_CODE" = "400" ]; then
  fail "WebSocket Returned 400 (Bad Request - Nginx or Axum rejected upgrade)"
else
  fail "WebSocket connection failed with code $WS_RESP_CODE. Expected 101."
fi


# =========================================================
# FINAL SUMMARY
# =========================================================
echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║                     📊 TEST RESULTS                         ║"
echo "╠═══════════════════════════════════════════════════════════════╣"
printf "  Passed: %d ✅\n" "$PASS"
printf "  Failed: %d ❌\n" "$FAIL"
echo ""

if [ "$FAIL" -eq 0 ]; then
  echo "║  🏆 ALL SYSTEMS GO! PRODUCTION INTEGRATION VERIFIED!        ║"
  echo "╚═══════════════════════════════════════════════════════════════╝"
  exit 0
else
  echo "║  ⚠️  ERRORS DETECTED                                        ║"
  echo "╚═══════════════════════════════════════════════════════════════╝"
  exit 1
fi
