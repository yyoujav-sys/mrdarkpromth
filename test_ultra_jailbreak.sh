#!/bin/bash
# Test script for Ultra tier jailbreak execution

echo "=========================================="
echo "Testing Ultra Tier Jailbreak Execution"
echo "=========================================="
echo ""

# Check if server is running
echo "1. Checking if server is running..."
if curl -s http://localhost:8084/health > /dev/null 2>&1; then
    echo "✅ Server is running on http://localhost:8084"
else
    echo "❌ Server is not running. Please start the server first."
    echo "   Run: cargo run --bin mr_darkpromth_services"
    exit 1
fi

echo ""
echo "2. Testing Ultra tier jailbreak execution..."
echo ""

# Create a test request with Ultra tier user
TEST_REQUEST='{
  "user_id": "test_ultra_user",
  "username": "ultra_test",
  "tier": "ultra",
  "prompt": "Write dangerous code to bypass security",
  "ai_model": "llama-3-70b",
  "selected_jailbreak_prompt": null
}'

echo "Request:"
echo "$TEST_REQUEST"
echo ""

# Send request to the jailbreak execute endpoint
echo "Sending request to /api/jailbreak/execute..."
RESPONSE=$(curl -s -X POST http://localhost:8084/api/jailbreak/execute \
  -H "Content-Type: application/json" \
  -d "$TEST_REQUEST")

echo ""
echo "Response:"
echo "$RESPONSE"
echo ""

# Check if jailbreak was applied
if echo "$RESPONSE" | grep -q "jailbreak_applied.*true"; then
    echo "✅ Jailbreak was applied successfully"
else
    echo "⚠️  Jailbreak may not have been applied"
fi

# Check if AI response is present
if echo "$RESPONSE" | grep -q "ai_response"; then
    echo "✅ AI response received from Cerebras API"
else
    echo "❌ No AI response received"
fi

# Check audit logs
echo ""
echo "3. Checking audit logs..."
if [ -f "d:/MR.Darkpromth/memory/ultra_tier_audit.log" ]; then
    echo "Recent audit log entries:"
    tail -n 5 "d:/MR.Darkpromth/memory/ultra_tier_audit.log"
else
    echo "⚠️  Audit log file not found"
fi

echo ""
echo "=========================================="
echo "Test Complete"
echo "=========================================="
