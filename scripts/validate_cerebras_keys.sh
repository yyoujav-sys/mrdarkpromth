#!/bin/bash
# Cerebras API Key Validator

# Load keys from .env if exists
if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
fi

KEYS=${CEREBRAS_API_KEYS:-""}

if [ -z "$KEYS" ]; then
    echo "❌ No CEREBRAS_API_KEYS found in environment or .env"
    exit 1
fi

IFS=',' read -ra ADDR <<< "$KEYS"
TOTAL=${#ADDR[@]}
SUCCESS=0

echo "🔍 Validating $TOTAL Cerebras API keys..."

for key in "${ADDR[@]}"; do
    # Simple validation using curl to the models list endpoint
    response=$(curl -s -o /dev/null -w "%{http_code}" -X GET "https://api.cerebras.ai/v1/models" \
        -H "Authorization: Bearer $key")

    if [ "$response" == "200" ]; then
        echo "✅ Key ${key:0:8}... is VALID"
        ((SUCCESS++))
    else
        echo "❌ Key ${key:0:8}... is INVALID (HTTP $response)"
    fi
done

echo "-----------------------------------"
echo "📊 Results: $SUCCESS/$TOTAL keys valid."

if [ $SUCCESS -eq $TOTAL ]; then
    echo "✨ All keys are working correctly."
    exit 0
else
    echo "⚠️ Some keys failed validation."
    exit 1
fi
