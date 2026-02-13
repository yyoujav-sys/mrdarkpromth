#!/bin/bash
# Billing Atomicity Stress Test v5

API_URL="http://localhost:80"
TIMESTAMP=$(date +%s)
EMAIL="stress_$TIMESTAMP@test.com"
PASSWORD="securePassword123"
USERNAME="stress_$TIMESTAMP"

echo "--- STEP 1: Setup User & Payment ---"
REG_RESP=$(curl -s -X POST "$API_URL/api/auth/register" -H "Content-Type: application/json" -d "{\"username\": \"$USERNAME\", \"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}")
USER_ID=$(echo $REG_RESP | jq -r .data.user_id)
TOKEN=$(echo $REG_RESP | jq -r .data.token)

QR_RESP=$(curl -s -X POST "$API_URL/api/billing/generate-qr" -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" -d "{\"plan_id\": \"ae67a09c-e6b7-4471-aa0d-239e9f855c88\"}")
REFERENCE=$(echo $QR_RESP | jq -r .data.reference)
PAYMENT_ID=$(docker exec mr_darkpromth_postgres psql -U postgres -d mr_darkpromth -t -A -c "SELECT id FROM payments WHERE reference = '$REFERENCE';" | grep -E "^[0-9a-f-]{36}$")

echo "Payment ID: [$PAYMENT_ID]"

echo "--- STEP 2: Insert Verification & Promote to Admin ---"
VERIFICATION_ID=$(docker exec mr_darkpromth_postgres psql -U postgres -d mr_darkpromth -t -A -c "INSERT INTO payment_slip_verifications (id, user_id, payment_id, slip_image_path, status) VALUES (gen_random_uuid(), '$USER_ID', '$PAYMENT_ID', 'stress.png', 'pending') RETURNING id;" | grep -E "^[0-9a-f-]{36}$")
echo "Verification ID: [$VERIFICATION_ID]"

docker exec mr_darkpromth_postgres psql -U postgres -d mr_darkpromth -c "UPDATE users SET tier = 'admin' WHERE id = '$USER_ID';"
ADMIN_TOKEN=$(curl -s -X POST "$API_URL/api/auth/login" -H "Content-Type: application/json" -d "{\"email\": \"$EMAIL\", \"password\": \"$PASSWORD\"}" | jq -r .data.token)

echo "--- STEP 3: Concurrent Approval Attack ---"
for i in {1..10}; do
  curl -s -X POST "$API_URL/api/admin/verifications/$VERIFICATION_ID/approve" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"approved\": true, \"notes\": \"Run $i\"}" -o "v5_resp_$i.json" &
done

wait
echo "Requests finished."

SUCCESS_COUNT=0
for i in {1..10}; do
  if [ -f "v5_resp_$i.json" ] && grep -q "\"data\":" "v5_resp_$i.json"; then
    SUCCESS_COUNT=$((SUCCESS_COUNT+1))
  else
    echo "Fail $i: $(cat v5_resp_$i.json 2>/dev/null || echo 'No file')"
  fi
done

echo "Final Success Count: $SUCCESS_COUNT"
if [ "$SUCCESS_COUNT" -eq 1 ]; then
  echo "RESULT: SUCCESS (Transactional integrity verified!)"
else
  echo "RESULT: FAILURE (Race condition detected: $SUCCESS_COUNT successes!)"
fi

rm v5_resp_*.json
