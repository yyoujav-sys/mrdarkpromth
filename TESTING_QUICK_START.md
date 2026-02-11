# 🚀 Production Readiness Testing - Quick Start

**Current Date**: February 11, 2026  
**Objective**: Complete all 4 test modules in 1 day  
**Status**: Ready to execute  

---

## ⚡ 5-Minute Setup

### 1. Make Scripts Executable
```bash
cd /opt/mrdarkpromth
chmod +x tests/test_module_*.sh tests/run_all_tests.sh
```

### 2. Ensure System is Running
```bash
cd /opt/mrdarkpromth

# Start all services
docker-compose up -d

# Wait for everything to start
sleep 10

# Verify health
curl http://localhost:3000/health
# Should return: {"status":"healthy"} or similar
```

### 3. For Chat & Terminal Tests: Get Auth Token
```bash
# Option A: Create test user and login
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_user_'$(date +%s)'",
    "email": "test_'$(date +%s)'@example.com",
    "password": "TestPassword123!"
  }' | jq '.token'

# Copy the token and set environment variable:
export AUTH_TOKEN="your_token_here"

# Option B: Use existing token
export AUTH_TOKEN="your_existing_token"
```

---

## 🧪 Running Tests

### Option 1: Run All Tests (Recommended)
```bash
cd /opt/mrdarkpromth

# Run complete test suite
AUTH_TOKEN=$AUTH_TOKEN bash tests/run_all_tests.sh

# This will:
# ✓ Test Module 1: Payment System
# ✓ Test Module 2: Terminal & Sandbox  
# ✓ Test Module 3: Chat & Failover
# ✓ Test Module 4: Security & Load Testing
# Estimated duration: 6-8 hours
```

### Option 2: Run Individual Modules

```bash
cd /opt/mrdarkpromth

# Module 1: Payment System (no token needed)
bash tests/test_module_1_payment.sh

# Module 2: Terminal & Sandbox (requires token)
AUTH_TOKEN=$AUTH_TOKEN bash tests/test_module_2_terminal.sh

# Module 3: Chat & Failover (requires token)
AUTH_TOKEN=$AUTH_TOKEN bash tests/test_module_3_chat.sh

# Module 4: Security & Load Testing (no token needed)
bash tests/test_module_4_security.sh
```

### Option 3: Run Specific Test (Advanced)
```bash
# To skip certain modules:
cd /opt/mrdarkpromth
SKIP_MODULES="2,3" bash tests/run_all_tests.sh
# This skips Modules 2 and 3, runs only 1 and 4
```

---

## 📊 Test Modules Overview

### Module 1: Payment System & Tier Upgrade (1.5 hours)
**What it tests**:
- ✓ New user sign-up as Free tier
- ✓ Upload payment slip image
- ✓ File upload validation (size, type, format)
- ✓ Database consistency checks
- ✓ Tier upgrade verification

**What you need**:
- Running API
- PostgreSQL database
- No special auth token

**Run it**:
```bash
bash tests/test_module_1_payment.sh
```

---

### Module 2: Ultra Terminal & Sandbox (2 hours)
**What it tests**:
- ✓ Basic terminal commands (echo, ls, pwd, etc)
- ✓ Complex script execution
- ✓ Sandbox isolation (can't escape container)
- ✓ Concurrent session performance
- ✓ RAM usage under load

**What you need**:
- Valid auth token (Ultra tier preferred)
- Running API
- Docker containers healthy

**Run it**:
```bash
export AUTH_TOKEN="your_token"
bash tests/test_module_2_terminal.sh
```

**What to monitor**:
- Open another terminal to watch resources:
  ```bash
  watch -n 1 'docker stats --no-stream'
  ```

---

### Module 3: Chat & AI Failover (1.5 hours)
**What it tests**:
- ✓ Normal chat flow (send messages, get responses)
- ✓ Chat history recording in database
- ✓ Failover from Cerebras to OpenRouter
- ✓ API key rotation working
- ✓ Proper logging of failures

**What you need**:
- Valid auth token
- Working Cerebras API keys
- Working OpenRouter fallback keys

**Run it**:
```bash
export AUTH_TOKEN="your_token"
bash tests/test_module_3_chat.sh
```

**Note**: This test temporarily modifies your .env to simulate failures. It automatically restores it.

---

### Module 4: Security & Load Testing (1.5 hours)
**What it tests**:
- ✓ Tier-based access control (Free ≠ Ultra)
- ✓ Rate limiting protection
- ✓ SSL/TLS certificate validity
- ✓ HTTP → HTTPS redirect working
- ✓ File upload size limits in Nginx
- ✓ Health check endpoint
- ✓ Database connectivity
- ✓ Structured logging

**What you need**:
- Running API
- Apache Benchmark (ab) installed (optional)
- Nginx configured

**Run it**:
```bash
bash tests/test_module_4_security.sh
```

---

## 📈 Expected Test Output

Each test produces output like:

```
═════════════════════════════════════════════════
Module 1: Payment System & Tier Upgrade
═════════════════════════════════════════════════

Test 1.1: Happy Path - Free→Ultra Upgrade
  Creating test user: test_pay_1707638400
  Response: {"user_id":"123","status":"success"}
  ✓ User created: 123
  ...

Test 1.2: Edge Case - Invalid File Upload
  1.2.1: Testing wrong MIME type (.txt)...
  ✓ Correctly rejected text file (HTTP 400)
  ...

═════════════════════════════════════════════════
Module 1 Summary
═════════════════════════════════════════════════
Tests Passed: 12
Tests Failed: 0
```

---

## ⚠️ Troubleshooting

### "API is not responding at http://localhost:3000"
```bash
# Check if docker-compose is running
docker-compose ps

# Start services if not running
cd /opt/mrdarkpromth
docker-compose up -d

# Wait 15 seconds and check health
sleep 15
curl http://localhost:3000/health
```

### "AUTH_TOKEN not set"
```bash
# Create a test user first
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "test_'$(date +%s)'",
    "email": "test_'$(date +%s)'@example.com",
    "password": "MyPassword123!"
  }'

# Extract token from response
export AUTH_TOKEN="<token_from_response>"

# Verify it's set
echo $AUTH_TOKEN
```

### "Database connection refused"
```bash
# Check if postgres is running
docker-compose logs postgres | tail -20

# Restart database
docker-compose restart postgres

# Wait for startup
sleep 10

# Test connection
psql -h localhost -U postgres -d mrdarkpromth -c "SELECT 1"
```

### "Test module execution failed (HTTP 500 errors)"
```bash
# Check API logs
docker-compose logs api | tail -50

# Restart API
docker-compose restart api

# Wait 5 seconds
sleep 5

# Rerun test
bash tests/test_module_1_payment.sh
```

### "Rate limiting or too many requests"
```bash
# If tests fail due to rate limiting, wait 60 seconds and retry
sleep 60
bash tests/test_module_4_security.sh
```

---

## 🎯 Testing Strategy for Today

### Morning (2-3 hours)
1. ✅ Run Module 1: Payment System
2. ✅ Run Module 4: Security (quick sanity check)
3. 📝 Document any failures

### Midday (2-3 hours)  
4. ✅ Run Module 2: Terminal & Sandbox
5. 📊 Monitor resource usage during tests
6. 📝 Document performance baselines

### Afternoon (2-3 hours)
7. ✅ Run Module 3: Chat & Failover
8. 🔧 Fix any issues found
9. 📋 Review all results

### If all pass:
10. ✅ Complete DISTRIBUTION_CHECKLIST.md
11. 🚀 Prepare for deployment tomorrow

---

## 📋 Test Tracking Sheet

```
Module 1: Payment System
  ☐ Test 1.1: Happy Path - PASS/FAIL
  ☐ Test 1.2: File Upload Edge Cases - PASS/FAIL
  ☐ Test 1.3: DB Consistency - PASS/FAIL

Module 2: Terminal & Sandbox
  ☐ Test 2.1: Basic Commands - PASS/FAIL
  ☐ Test 2.2: Complex Script - PASS/FAIL
  ☐ Test 2.3: Isolation - PASS/FAIL
  ☐ Test 2.4: Concurrent Performance - PASS/FAIL

Module 3: Chat & Failover
  ☐ Test 3.1: Normal Flow - PASS/FAIL
  ☐ Test 3.2: OpenRouter Failover - PASS/FAIL
  ☐ Test 3.3: Key Rotation - PASS/FAIL

Module 4: Security & Load
  ☐ Test 4.1: Tier Authorization - PASS/FAIL
  ☐ Test 4.2: Rate Limiting - PASS/FAIL
  ☐ Test 4.3: SSL/TLS - PASS/FAIL
  ☐ Test 4.4: Health Check - PASS/FAIL
  ☐ Test 4.5: Database - PASS/FAIL
  ☐ Test 4.6: Monitoring - PASS/FAIL

TOTAL: ☐ ALL PASS (ready for deployment)
```

---

## 🔧 System Commands Cheat Sheet

### Logs
```bash
# All API logs
docker-compose logs api

# Last 50 lines, follow updates
docker-compose logs -f api --tail=50

# Specific time range
docker-compose logs api --since 10m

# PostgreSQL logs
docker-compose logs postgres | tail -20

# All logs
docker-compose logs | tail -100
```

### System Status
```bash
# All services
docker-compose ps

# Resource usage
docker stats --no-stream

# Specific container
docker stats mr_darkpromth_api --no-stream

# Uptime
docker-compose logs api | grep -i "listening\|started"
```

### Database
```bash
# Connect to database
psql -h localhost -U postgres -d mrdarkpromth

# List tables
\dt

# Check table size
SELECT 
  schemaname,
  tablename,
  pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename))
FROM pg_tables
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

# Exit
\q
```

### API Testing
```bash
# Health check
curl http://localhost:3000/health

# With authentication
curl -H "Authorization: Bearer $AUTH_TOKEN" \
  http://localhost:3000/api/user/profile

# POST request
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $AUTH_TOKEN" \
  -d '{"message":"Hello"}'
```

---

## ✅ Success Criteria

**All tests pass** when:
- ✅ Module 1: 0 failures
- ✅ Module 2: 0 failures
- ✅ Module 3: 0 failures  
- ✅ Module 4: 0 failures
- ✅ No critical issues pending
- ✅ Performance meets baselines
- ✅ No data corruption
- ✅ Failover works correctly

**Then you can proceed with**:
1. Complete DISTRIBUTION_CHECKLIST.md
2. Take database backup
3. Deploy to production
4. Monitor for 24 hours
5. Announce to users

---

## 📞 Support

If you need help:
1. Check this file first
2. Check logs: `docker-compose logs api`
3. Review specific test output for error details
4. Try running individual tests in isolation

Good luck! 🚀
