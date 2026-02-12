#!/bin/bash
# scripts/billing_flow_test.sh

API_URL="http://localhost:8080"
TIMESTAMP=$(date +%s)
TEST_EMAIL="billing_test_${TIMESTAMP}@example.com"
TEST_PASS="Password123!"
TEST_USER="billing_tester_${TIMESTAMP}"
ULTRA_PLAN_ID="ae67a09c-e6b7-4471-aa0d-239e9f855c88"

ADMIN_EMAIL="admin@mrdarkpromth.ai"
ADMIN_PASS="admin123"

echo "--------------------------------------------------"
echo "STEP 1: Registering Test User ($TEST_EMAIL)"
REG_RES=$(curl -s -X POST "$API_URL/api/auth/register" \
     -H "Content-Type: application/json" \
     -d "{\"email\": \"$TEST_EMAIL\", \"password\": \"$TEST_PASS\", \"username\": \"$TEST_USER\"}")

USER_TOKEN=$(echo $REG_RES | jq -r '.data.token')
USER_ID=$(echo $REG_RES | jq -r '.data.user_id')

if [ "$USER_TOKEN" == "null" ] || [ -z "$USER_TOKEN" ]; then
    echo "ERROR: Registration failed"
    echo "Response: $REG_RES"
    exit 1
fi
echo "SUCCESS: Registered User ID: $USER_ID"

echo "--------------------------------------------------"
echo "STEP 2: Generating QR Code for Ultra Plan"
QR_RES=$(curl -s -X POST "$API_URL/api/billing/generate-qr" \
     -H "Authorization: Bearer $USER_TOKEN" \
     -H "Content-Type: application/json" \
     -d "{\"plan_id\": \"$ULTRA_PLAN_ID\"}")

REFERENCE=$(echo $QR_RES | jq -r '.data.reference')

if [ "$REFERENCE" == "null" ] || [ -z "$REFERENCE" ]; then
    echo "ERROR: QR Generation failed"
    echo "Response: $QR_RES"
    exit 1
fi
echo "SUCCESS: Generated Reference: $REFERENCE"

echo "--------------------------------------------------"
echo "STEP 3: Simulating Slip Upload"
SLIP_RES=$(curl -s -X POST "$API_URL/api/billing/verify-slip" \
     -H "Authorization: Bearer $USER_TOKEN" \
     -H "Content-Type: application/json" \
     -d "{\"reference\": \"$REFERENCE\", \"slip_image\": \"data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==\"}")

if echo $SLIP_RES | grep -q "error"; then
    echo "ERROR: Slip upload failed"
    echo "Response: $SLIP_RES"
    exit 1
fi
echo "SUCCESS: Slip uploaded and pending verification"

echo "--------------------------------------------------"
echo "STEP 4: Logging in as Admin"
ADMIN_RES=$(curl -s -X POST "$API_URL/api/auth/login" \
     -H "Content-Type: application/json" \
     -d "{\"email\": \"$ADMIN_EMAIL\", \"password\": \"$ADMIN_PASS\"}")

ADMIN_TOKEN=$(echo $ADMIN_RES | jq -r '.data.token')

if [ "$ADMIN_TOKEN" == "null" ] || [ -z "$ADMIN_TOKEN" ]; then
    echo "ERROR: Admin login failed"
    echo "Response: $ADMIN_RES"
    exit 1
fi
echo "SUCCESS: Admin logged in"

echo "--------------------------------------------------"
echo "STEP 5: Fetching Pending Verifications"
PENDING_RES=$(curl -s -X GET "$API_URL/api/admin/verifications/pending" \
     -H "Authorization: Bearer $ADMIN_TOKEN")

VERIFICATION_ID=$(echo $PENDING_RES | jq -r ".data.verifications[] | select(.user_id == \"$USER_ID\") | .id" | head -n 1)

if [ "$VERIFICATION_ID" == "null" ] || [ -z "$VERIFICATION_ID" ]; then
    echo "ERROR: Could not find pending verification for user $USER_ID"
    echo "Response: $PENDING_RES"
    exit 1
fi
echo "SUCCESS: Found Verification ID: $VERIFICATION_ID"

echo "--------------------------------------------------"
echo "STEP 6: Approving Slip"
APPROVE_RES=$(curl -s -X POST "$API_URL/api/admin/verifications/$VERIFICATION_ID/approve" \
     -H "Authorization: Bearer $ADMIN_TOKEN" \
     -H "Content-Type: application/json" \
     -d "{\"notes\": \"Approved via automated test\", \"is_approved\": true}")

if echo $APPROVE_RES | grep -q "error"; then
    echo "ERROR: Approval failed"
    echo "Response: $APPROVE_RES"
    exit 1
fi
echo "SUCCESS: Slip approved"

echo "--------------------------------------------------"
echo "STEP 7: Verifying User Tier Upgrade"
ME_RES=$(curl -s -X GET "$API_URL/api/auth/me" \
     -H "Authorization: Bearer $USER_TOKEN")

CURRENT_TIER=$(echo $ME_RES | jq -r '.data.tier')

if [ "$CURRENT_TIER" == "ultra" ]; then
    echo "FINAL SUCCESS: User tier is now $CURRENT_TIER"
else
    echo "FINAL FAILURE: User tier is $CURRENT_TIER, expected ultra"
    echo "Response: $ME_RES"
    exit 1
fi

echo "--------------------------------------------------"
echo "Cleanup: Admin and test user remain in DB for manual inspection if needed."
