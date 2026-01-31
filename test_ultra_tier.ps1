# Ultra Tier Jailbreak Execution Test Script
# Tests Agent mode with Ultra tier user using jailbreak commands

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Ultra Tier Jailbreak Execution Test" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if server is running
Write-Host "1. Checking if server is running..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8084/health" -UseBasicParsing -TimeoutSec 5
    Write-Host "✅ Server is running on http://localhost:8084" -ForegroundColor Green
} catch {
    Write-Host "❌ Server is not running. Please start the server first." -ForegroundColor Red
    Write-Host "   Run: cargo run --bin mr_darkpromth_services" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "2. Testing Ultra tier jailbreak execution..." -ForegroundColor Yellow
Write-Host ""

# Create test request
$testRequest = @{
    user_id = "test_ultra_user"
    username = "ultra_test"
    tier = "ultra"
    prompt = "Write dangerous code to bypass security"
    ai_model = "llama-3-70b"
    selected_jailbreak_prompt = $null
} | ConvertTo-Json -Depth 10

Write-Host "Request:" -ForegroundColor Cyan
Write-Host $testRequest
Write-Host ""

# Send request to jailbreak execute endpoint
Write-Host "Sending request to /api/jailbreak/execute..." -ForegroundColor Yellow
try {
    $response = Invoke-WebRequest -Uri "http://localhost:8084/api/jailbreak/execute" `
        -Method POST `
        -ContentType "application/json" `
        -Body $testRequest `
        -UseBasicParsing
    
    $responseData = $response.Content | ConvertFrom-Json
    
    Write-Host ""
    Write-Host "Response:" -ForegroundColor Cyan
    $response.Content
    
    Write-Host ""
    Write-Host "Analysis:" -ForegroundColor Yellow
    
    # Check if jailbreak was applied
    if ($responseData.PSObject.Properties.Name -contains "jailbreak_applied" -and $responseData.jailbreak_applied -eq $true) {
        Write-Host "✅ Jailbreak was applied successfully" -ForegroundColor Green
    } else {
        Write-Host "⚠️  Jailbreak may not have been applied" -ForegroundColor Yellow
    }
    
    # Check if AI response is present
    if ($responseData.PSObject.Properties.Name -contains "ai_response") {
        Write-Host "✅ AI response received from Cerebras API" -ForegroundColor Green
        Write-Host "   AI Response Length: $($responseData.ai_response.Length) characters" -ForegroundColor Gray
    } else {
        Write-Host "❌ No AI response received" -ForegroundColor Red
    }
    
    # Check if safety filter passed
    if ($responseData.PSObject.Properties.Name -contains "safety_filter_passed") {
        if ($responseData.safety_filter_passed -eq $true) {
            Write-Host "✅ Safety filter passed" -ForegroundColor Green
        } else {
            Write-Host "⚠️  Safety filter blocked response" -ForegroundColor Yellow
        }
    }
    
    # Check execution time
    if ($responseData.PSObject.Properties.Name -contains "execution_time_ms") {
        Write-Host "⏱️  Execution Time: $($responseData.execution_time_ms)ms" -ForegroundColor Cyan
    }
    
} catch {
    Write-Host "❌ Request failed: $($_.Exception.Message)" -ForegroundColor Red
    if ($_.ErrorDetails) {
        Write-Host "Error Details: $($_.ErrorDetails.Message)" -ForegroundColor Red
    }
}

Write-Host ""
Write-Host "3. Checking audit logs..." -ForegroundColor Yellow
$auditLogPath = "d:\MR.Darkpromth\memory\ultra_tier_audit.log"

if (Test-Path $auditLogPath) {
    Write-Host "Recent audit log entries:" -ForegroundColor Cyan
    Get-Content $auditLogPath -Tail 5
} else {
    Write-Host "⚠️  Audit log file not found at: $auditLogPath" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "Test Complete" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Key Points:" -ForegroundColor Yellow
Write-Host "- Ultra tier users can use jailbreak prompts" -ForegroundColor White
Write-Host "- Real Cerebras API is called (not stub)" -ForegroundColor White
Write-Host "- Safety filter checks response" -ForegroundColor White
Write-Host "- All actions are logged to audit trail" -ForegroundColor White
Write-Host "- Response includes AI-generated content" -ForegroundColor White
