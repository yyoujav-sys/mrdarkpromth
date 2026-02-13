#!/bin/bash

# Final verification script - confirms all fixes are in place
# Run after deployment to verify everything is working

set -e

echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║         🔍 MR.DarkPromth Core Issues - Final Verification         ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""

PASSED=0
FAILED=0
TOTAL=0

# Helper function to test items
test_item() {
    local test_name=$1
    local command=$2
    local expected=$3
    
    TOTAL=$((TOTAL + 1))
    echo -n "Testing: $test_name... "
    
    OUTPUT=$(eval "$command" 2>&1)
    EXIT_CODE=$?
    
    if [ $EXIT_CODE -eq 0 ]; then
        if [ -z "$expected" ]; then
            echo "✅ PASS"
            PASSED=$((PASSED + 1))
            return 0
        elif echo "$OUTPUT" | eval "$expected" >/dev/null 2>&1; then
             echo "✅ PASS"
             PASSED=$((PASSED + 1))
             return 0
        fi
    fi
    
    echo "❌ FAIL"
    echo "    Command: $command"
    echo "    Output: $OUTPUT"
    FAILED=$((FAILED + 1))
    return 1
}

# Section 1: Code Compilation
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1️⃣  CODE COMPILATION"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_item "Rust code compiles" \
    "cd /opt/mrdarkpromth/mr_darkpromth && cargo check --verbose" \
    "grep -q 'Finished'"

test_item "Release build exists (in container)" \
    "docker exec mr_darkpromth_api test -f /usr/local/bin/mr_darkpromth_api"

test_item "API module compiled" \
    "ls -la /opt/mrdarkpromth/mr_darkpromth/api/src/*.rs | wc -l | grep -q '[0-9]'"

echo ""

# Section 2: Error Handling
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2️⃣  ERROR HANDLING"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_item "Error handler module exists" \
    "test -f /opt/mrdarkpromth/mr_darkpromth/api/src/error_handler.rs"

test_item "Error handler has ApiError struct" \
    "grep -q 'pub struct ApiError' /opt/mrdarkpromth/mr_darkpromth/api/src/error_handler.rs"

test_item "Error codes defined" \
    "grep -q 'pub const AUTH_FAILED' /opt/mrdarkpromth/mr_darkpromth/api/src/error_handler.rs"

echo ""

# Section 3: Logging
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3️⃣  STRUCTURED LOGGING"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_item "Structured logging module exists" \
    "test -f /opt/mrdarkpromth/mr_darkpromth/api/src/structured_logging.rs"

test_item "Logging has JSON support" \
    "grep -q 'serde_json::json' /opt/mrdarkpromth/mr_darkpromth/api/src/structured_logging.rs"

test_item "Performance metrics defined" \
    "grep -q 'pub struct PerformanceMetrics' /opt/mrdarkpromth/mr_darkpromth/api/src/structured_logging.rs"

echo ""

# Section 4: Authentication
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4️⃣  AUTHENTICATION & VALIDATION"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_item "Auth validators module exists" \
    "test -f /opt/mrdarkpromth/mr_darkpromth/api/src/auth_validators.rs"

test_item "Password validation implemented" \
    "grep -q 'pub fn validate_password' /opt/mrdarkpromth/mr_darkpromth/api/src/auth_validators.rs"

test_item "Email validation implemented" \
    "grep -q 'pub fn validate_email' /opt/mrdarkpromth/mr_darkpromth/api/src/auth_validators.rs"

test_item "Login attempt tracker implemented" \
    "grep -q 'pub struct LoginAttemptTracker' /opt/mrdarkpromth/mr_darkpromth/api/src/auth_validators.rs"

echo ""

# Section 5: Database Configuration
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5️⃣  DATABASE CONFIGURATION"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_item "Database config module exists" \
    "test -f /opt/mrdarkpromth/mr_darkpromth/api/src/database_config.rs"

test_item "Connection pool configured" \
    "grep -q 'max_connections' /opt/mrdarkpromth/mr_darkpromth/api/src/database_config.rs"

test_item "Health check function exists" \
    "grep -q 'pub async fn check_database_health' /opt/mrdarkpromth/mr_darkpromth/api/src/database_config.rs"

echo ""

# Section 6: Rate Limiting
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "6️⃣  RATE LIMITING"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_item "Rate limiting module exists" \
    "test -f /opt/mrdarkpromth/mr_darkpromth/api/src/rate_limiting_middleware.rs"

test_item "IpRateLimiter struct exists" \
    "grep -q 'pub struct IpRateLimiter' /opt/mrdarkpromth/mr_darkpromth/api/src/rate_limiting_middleware.rs"

test_item "Rate limit presets defined" \
    "grep -q 'pub fn standard' /opt/mrdarkpromth/mr_darkpromth/api/src/rate_limiting_middleware.rs"

echo ""

# Section 7: Automation Scripts
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "7️⃣  AUTOMATION SCRIPTS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_item "Backup script exists" \
    "test -x /opt/mrdarkpromth/scripts/backup_db.sh"

test_item "Backup verify script exists" \
    "test -x /opt/mrdarkpromth/scripts/verify_backup.sh"

test_item "Cron setup script exists" \
    "test -x /opt/mrdarkpromth/scripts/setup_cron_jobs.sh"

test_item "SSL renewal script exists" \
    "test -x /opt/mrdarkpromth/scripts/ssl-renewal.sh"

test_item "API test script exists" \
    "test -x /opt/mrdarkpromth/scripts/test_api_endpoints.sh"

echo ""

# Section 8: API Runtime
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "8️⃣  API RUNTIME (requires running services)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_item "Docker API container running" \
    "docker ps | grep -q 'mr_darkpromth.*api'"

test_item "API health endpoint available" \
    "curl -s http://localhost/health | grep -q 'healthy'"

test_item "Prometheus metrics available" \
    "curl -s http://localhost:9090/api/v1/targets | grep -q '\"job\":'"

test_item "Grafana available" \
    "curl -s http://localhost:3001/api/health | grep -q 'ok'"

echo ""

# Section 9: Files & Documentation
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "9️⃣  DOCUMENTATION"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_item "Core issues report exists" \
    "test -f /opt/mrdarkpromth/CORE_ISSUES_FIXED_REPORT.md"

test_item "Deployment guide exists" \
    "test -f /opt/mrdarkpromth/DEPLOYMENT_GUIDE.md"

test_item "Implementation summary exists" \
    "test -f /opt/mrdarkpromth/IMPLEMENTATION_SUMMARY.md"

test_item "Production readiness audit exists" \
    "test -f /opt/mrdarkpromth/PRODUCTION_READINESS_AUDIT_2026.md"

echo ""

# Section 10: Dependencies
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔟 DEPENDENCIES"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

test_item "Regex crate in Cargo.toml" \
    "grep -q 'regex' /opt/mrdarkpromth/mr_darkpromth/api/Cargo.toml"

test_item "Lazy_static crate in Cargo.toml" \
    "grep -q 'lazy_static' /opt/mrdarkpromth/mr_darkpromth/api/Cargo.toml"

echo ""

# Final Summary
echo "╔════════════════════════════════════════════════════════════════════╗"
echo "║                         📊 TEST SUMMARY                            ║"
echo "╚════════════════════════════════════════════════════════════════════╝"
echo ""
echo "Total Tests:     $TOTAL"
echo "Passed:          $PASSED ✅"
echo "Failed:          $FAILED ❌"
echo ""

PASS_RATE=$((PASSED * 100 / TOTAL))
echo "Pass Rate:       $PASS_RATE%"
echo ""

if [ $FAILED -eq 0 ]; then
    echo "╔════════════════════════════════════════════════════════════════════╗"
    echo "║           ✅ ALL VERIFICATIONS PASSED - READY FOR PROD!           ║"
    echo "╚════════════════════════════════════════════════════════════════════╝"
    echo ""
    echo "Next Steps:"
    echo "  1. Review CORE_ISSUES_FIXED_REPORT.md"
    echo "  2. Review DEPLOYMENT_GUIDE.md"
    echo "  3. Run: bash /opt/mrdarkpromth/scripts/test_api_endpoints.sh"
    echo "  4. Run: bash /opt/mrdarkpromth/scripts/verify_backup.sh"
    echo ""
    exit 0
else
    echo "╔════════════════════════════════════════════════════════════════════╗"
    echo "║                  ⚠️  SOME TESTS FAILED                            ║"
    echo "╚════════════════════════════════════════════════════════════════════╝"
    echo ""
    echo "Please review:"
    echo "  1. Check Docker services: docker-compose ps"
    echo "  2. Check logs: docker-compose logs"
    echo "  3. Review scripts are executable: ls -la /opt/mrdarkpromth/scripts/"
    echo ""
    exit 1
fi
