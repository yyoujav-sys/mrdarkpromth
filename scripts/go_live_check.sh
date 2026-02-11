#!/bin/bash
# MR.DarkPromth Go-Live Verification Script
# Run this before go-live to verify all systems are operational
set -e

PASS=0
FAIL=0
WARN=0

check() {
    local name=$1
    local result=$2
    local expected=$3
    if [ "$result" = "$expected" ]; then
        echo "  ✅ $name"
        PASS=$((PASS + 1))
    else
        echo "  ❌ $name (got: $result, expected: $expected)"
        FAIL=$((FAIL + 1))
    fi
}

warn_check() {
    local name=$1
    local result=$2
    echo "  ⚠️  $name: $result"
    WARN=$((WARN + 1))
}

echo "============================================"
echo "  MR.DarkPromth Go-Live Verification"
echo "  $(date)"
echo "============================================"
echo

# 1. Service Health
echo "📡 Service Health"
check "API healthy" "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/health)" "200"
check "Prometheus healthy" "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:9090/-/healthy)" "200"
check "Alertmanager healthy" "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:9093/-/healthy)" "200"
check "Grafana healthy" "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:3000/api/health)" "200"

# 2. Database
echo
echo "🗄️  Database"
DB_USERS=$(curl -s http://localhost:8080/metrics | grep "^mrdarkpromth_users_total" | awk '{print $2}')
check "Users in DB" "$([ "$DB_USERS" -gt 0 ] 2>/dev/null && echo "yes" || echo "no")" "yes"
echo "  ℹ️  Total users: $DB_USERS"

# 3. Metrics
echo
echo "📊 Prometheus Metrics"
TARGETS=$(curl -s http://localhost:9090/api/v1/targets | python3 -c "import sys,json; d=json.load(sys.stdin); active=[t for t in d['data']['activeTargets'] if t['health']=='up']; print(len(active))" 2>/dev/null)
check "Prometheus targets up" "$([ "$TARGETS" -ge 2 ] && echo "yes" || echo "no")" "yes"
echo "  ℹ️  Active targets: $TARGETS"

RULES=$(curl -s http://localhost:9090/api/v1/rules | python3 -c "import sys,json; d=json.load(sys.stdin); print(sum(len(g['rules']) for g in d['data']['groups']))" 2>/dev/null)
check "Alert rules loaded" "$([ "$RULES" -ge 10 ] && echo "yes" || echo "no")" "yes"
echo "  ℹ️  Total rules: $RULES"

# 4. Rate Limiting
echo
echo "🚫 Rate Limiting"
check "Rate limit metrics exposed" "$(curl -s http://localhost:8080/metrics | grep -c '^mrdarkpromth_rate_limit_rejected_total ')" "1"

# 5. Endpoints
echo
echo "🔌 API Endpoints"
check "GET /health" "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/health)" "200"
check "GET /metrics" "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/metrics)" "200"
check "GET /api/billing/plans" "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/api/billing/plans)" "200"
check "GET /api/auth/me (no auth)" "$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/api/auth/me)" "401"

# 6. Backups
echo
echo "💾 Backups"
BACKUP_COUNT=$(ls /backups/databases/*.sql.gz 2>/dev/null | wc -l)
check "Backups exist" "$([ "$BACKUP_COUNT" -gt 0 ] && echo "yes" || echo "no")" "yes"
echo "  ℹ️  Backup files: $BACKUP_COUNT"

# 7. Docker Containers
echo
echo "🐳 Docker Containers"
RUNNING=$(docker ps --filter "name=mr_darkpromth" --format "{{.Names}}: {{.Status}}" | grep -c "Up")
echo "  ℹ️  Running containers: $RUNNING"

# Summary
echo
echo "============================================"
echo "  RESULTS: $PASS passed, $FAIL failed, $WARN warnings"
echo "============================================"

if [ "$FAIL" -gt 0 ]; then
    echo "  🔴 NOT READY FOR GO-LIVE"
    exit 1
else
    echo "  🟢 READY FOR GO-LIVE"
    exit 0
fi
