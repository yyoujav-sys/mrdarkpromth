#!/bin/bash
# Failover Verification Script

echo "=== Failover Testing for MR.DarkPromth ==="

# 1. Check if OpenRouter keys are loaded
echo "Checking API logs for OpenRouter keys..."
if [ -f "backend_failover.log" ]; then
    grep -i "OpenRouter" backend_failover.log
else
    docker logs mr_darkpromth_api | grep -i "OpenRouter"
fi

# 2. Try a chat request (requires a user account, so we'll just check the endpoint existence for now)
echo "Verifying /api/chat endpoint..."
curl -s -X POST http://localhost:8080/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "test"}' | grep -q "Unauthorized" && echo "✅ /api/chat is reachable (Unauthorized as expected)"

echo "=== Test Prep Complete ==="
