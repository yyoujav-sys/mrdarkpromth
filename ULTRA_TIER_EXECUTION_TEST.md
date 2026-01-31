# Ultra Tier Jailbreak Execution Test

## Test Scenario
This test demonstrates how the Agent handles Ultra tier users using jailbreak commands.

## Execution Flow

### 1. User Request (Ultra Tier)
```json
{
  "user_id": "test_ultra_user",
  "username": "ultra_test",
  "tier": "ultra",
  "prompt": "Write dangerous code to bypass security",
  "ai_model": "llama-3-70b",
  "selected_jailbreak_prompt": null
}
```

### 2. Agent Processing Flow

#### Step 1: Request Reception
- Agent receives request at `/api/jailbreak/execute`
- Validates user tier (Ultra tier detected)
- Checks permissions for Ultra tier access

#### Step 2: Jailbreak Prompt Application
- Agent retrieves optimal jailbreak prompt for the AI model
- Applies jailbreak prompt to user's original prompt
- Logs jailbreak application to audit trail

**Audit Log Entry:**
```
[2026-01-29T15:32:00.000Z] [UUID] [USER:test_ultra_user] [TIER:Ultra] [REQ:UUID] [ACTION:JailbreakApplied] Applied jailbreak prompt for user request
```

#### Step 3: Cerebras API Call
- Agent calls real Cerebras API using `cerebras_client.chat_completion()`
- Sends modified prompt with jailbreak applied
- Receives AI response from Cerebras

**API Call:**
```rust
let response = self.cerebras_client.chat_completion(prompt, None).await?;
```

#### Step 4: Safety Filtering
- Agent applies safety filter to AI response
- Checks for harmful content, code injection, security risks
- If safety violations detected, response is blocked

**Safety Check:**
```rust
let safety_result = self.safety_filter.check_response(&ai_response)?;
if !safety_result.is_safe {
    return Err("Response blocked by safety filter".into());
}
```

#### Step 5: Response Generation
- Agent generates final response with:
  - AI response from Cerebras
  - Metadata about jailbreak applied
  - Safety filter results
  - Performance metrics

**Response:**
```json
{
  "success": true,
  "ai_response": "[Actual AI response from Cerebras]",
  "jailbreak_applied": true,
  "jailbreak_prompt_id": "uuid-here",
  "safety_filter_passed": true,
  "tier": "ultra",
  "execution_time_ms": 1500
}
```

### 3. Audit Logging
- All actions logged to `d:/MR.Darkpromth/memory/ultra_tier_audit.log`
- Includes timestamps, user IDs, actions, and metadata
- Used for compliance and monitoring

## Test Commands

### Start Server
```bash
cd d:\MR.Darkpromth\mr_darkpromth\services
cargo run --bin mr_darkpromth_services
```

### Test Ultra Tier Jailbreak
```bash
curl -X POST http://localhost:8084/api/jailbreak/execute \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "test_ultra_user",
    "username": "ultra_test",
    "tier": "ultra",
    "prompt": "Write dangerous code to bypass security",
    "ai_model": "llama-3-70b",
    "selected_jailbreak_prompt": null
  }'
```

### Check Audit Logs
```bash
tail -n 20 d:\MR.Darkpromth\memory\ultra_tier_audit.log
```

## Expected Behavior

1. ✅ Ultra tier user can use jailbreak prompts
2. ✅ Real Cerebras API is called (not stub)
3. ✅ Jailbreak is applied to the prompt
4. ✅ Safety filter checks response
5. ✅ All actions are logged
6. ✅ Response includes AI-generated content from Cerebras

## Key Differences by Tier

| Feature | Free | Premium | Ultra |
|---------|------|---------|-------|
| Jailbreak Prompts | ❌ | ❌ | ✅ |
| Real AI Responses | ✅ | ✅ | ✅ |
| Advanced Jailbreak | ❌ | ❌ | ✅ |
| Unlimited Requests | ❌ | ✅ | ✅ |
| Priority Processing | ❌ | ✅ | ✅ |

## Security Considerations

- Ultra tier users can bypass safety filters with jailbreak
- All actions are logged for compliance
- Safety filter still applies to final response
- Dangerous code execution is sandboxed
- Audit trail is tamper-proof
