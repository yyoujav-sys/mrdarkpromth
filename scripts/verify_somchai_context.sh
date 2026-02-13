#!/bin/bash
# Somchai Context Retention Test

API_URL="http://localhost:80"
EMAIL="somchai_$(date +%s)@test.com"
PASSWORD="somchaiPassword123"
USERNAME="somchai_$(date +%s)"

echo "--- STEP 1: Register ---"
REGISTER_RESP=$(curl -s -X POST "$API_URL/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}")
echo $REGISTER_RESP

TOKEN=$(echo $REGISTER_RESP | jq -r .data.token)

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
  echo "Registration failed"
  exit 1
fi

echo -e "\n--- STEP 2: Turn 1 - Introduce Somchai ---"
CHAT1_RESP=$(curl -s -X POST "$API_URL/api/chat" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"message\": \"สวัสดีครับ ผมชื่อสมชาย\"}")
echo $CHAT1_RESP

CONV_ID=$(echo $CHAT1_RESP | jq -r .data.conversation_id)

echo -e "\n--- STEP 3: Turn 2 - Verify Name Recognition ---"
CHAT2_RESP=$(curl -s -X POST "$API_URL/api/chat" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"message\": \"จำได้ไหมว่าผมชื่ออะไร\", \"conversation_id\": \"$CONV_ID\"}")
echo $CHAT2_RESP

RESPONSE_TEXT=$(echo $CHAT2_RESP | jq -r .data.response)

if [[ "$RESPONSE_TEXT" == *"สมชาย"* ]]; then
  echo -e "\nSUCCESS: AI remembered Somchai!"
else
  echo -e "\nFAILURE: AI forgot Somchai."
fi
