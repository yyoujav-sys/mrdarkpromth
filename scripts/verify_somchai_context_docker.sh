#!/bin/bash
# Somchai Context Retention Test (via Docker Exec)

EMAIL="somchai_$(date +%s)@test.com"
PASSWORD="somchaiPassword123"
USERNAME="somchai_$(date +%s)"

echo "--- STEP 1: Register ---"
REGISTER_JSON=$(docker exec -i mr_darkpromth_api curl -s -X POST "http://localhost:8080/api/auth/register" \
  -H "Content-Type: application/json" \
  -d "{\"username\": \"$USERNAME\", \"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}")

TOKEN=$(echo $REGISTER_JSON | jq -r .data.token)

if [ "$TOKEN" == "null" ] || [ -z "$TOKEN" ]; then
  echo "Registration failed: $REGISTER_JSON"
  exit 1
fi

echo -e "\n--- STEP 2: Turn 1 - Introduce Somchai ---"
CHAT1_JSON=$(docker exec -i mr_darkpromth_api curl -s -X POST "http://localhost:8080/api/chat" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"message\": \"สวัสดีครับ ผมชื่อสมชาย\"}")

CONV_ID=$(echo $CHAT1_JSON | jq -r .data.conversation_id)

echo -e "\n--- STEP 3: Turn 2 - Verify Name Recognition ---"
CHAT2_JSON=$(docker exec -i mr_darkpromth_api curl -s -X POST "http://localhost:8080/api/chat" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{\"message\": \"จำได้ไหมว่าผมชื่ออะไร\", \"conversation_id\": \"$CONV_ID\"}")

RESPONSE_TEXT=$(echo $CHAT2_JSON | jq -r .data.response)
echo "AI Response: $RESPONSE_TEXT"

if [[ "$RESPONSE_TEXT" == *"สมชาย"* ]]; then
  echo -e "\nSUCCESS: AI remembered Somchai!"
else
  echo -e "\nFAILURE: AI forgot Somchai."
  echo "Raw Response: $CHAT2_JSON"
fi
