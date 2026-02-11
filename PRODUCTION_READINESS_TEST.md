# 🚀 Production Readiness Test Plan (1-Day Complete)

**Date**: February 11, 2026  
**Goal**: Verify all critical systems work end-to-end before distribution  
**Duration**: 1 day  
**Test Coverage**: 4 modules, 15+ test cases  

---

## 📋 Executive Summary

| Module | Status | Criticality | Estimated Time |
|--------|--------|-------------|-----------------|
| 1. Payment & Tier Upgrade | ⏳ Pending | Critical | 1.5 hours |
| 2. Ultra Terminal & Sandbox | ⏳ Pending | Critical | 2 hours |
| 3. Chat & AI Failover | ⏳ Pending | High | 1.5 hours |
| 4. Security & Load Testing | ⏳ Pending | High | 1.5 hours |
| **Total** | | | **6.5 hours** |

---

## 🧪 Module 1: Payment System & Tier Upgrade (Critical Path)

### Test Case 1.1: Happy Path - Free→Ultra Upgrade
**Scenario**: New user joins as Free tier, uploads fake slip, admin approves, tier upgrades instantly

**Steps**:
1. Create test user account (Free tier)
   - Username: `test_payment_user_${timestamp}`
   - Email: `test_${timestamp}@example.com`
2. User uploads payment slip (fake image)
   - File: Generate 512KB PNG image
   - Expected: File accepted by API
3. Admin panel approval
   - Navigate to Admin → Payment Approvals
   - Approve the slip
   - Expected: Instant redirect and confirmation
4. Verify database state
   - Check `ultra_subscriptions` table: payment_status = 'approved'
   - Check `user_tiers` table: tier = 'ultra', activated_at = now()
   - Check `payment_audit_log` for approval timestamp

**Verification Queries**:
```sql
SELECT * FROM ultra_subscriptions WHERE user_id = $1 AND status = 'approved';
SELECT * FROM user_tiers WHERE user_id = $1 AND tier = 'ultra';
SELECT * FROM payment_audit_log WHERE user_id = $1 ORDER BY created_at DESC LIMIT 5;
```

**Pass Criteria**:
- ✅ Slip uploads successfully
- ✅ Tier changes within 2 seconds of approval
- ✅ All database records consistent
- ✅ No orphaned records

---

### Test Case 1.2: Edge Case - Invalid File Upload
**Scenario**: Attempt to upload invalid file types or oversized files

**Tests**:
1. **Wrong MIME type**: Upload .txt file (should reject)
   - Expected: 400 Bad Request + "Invalid file type"
2. **File too large**: Upload 25MB file (limit: 20MB)
   - Expected: 413 Payload Too Large + "File exceeds 20MB limit"
3. **Corrupted image**: Upload invalid PNG header
   - Expected: 400 Bad Request + "Invalid image"
4. **Zero-byte file**: Upload empty file
   - Expected: 400 Bad Request + "File is empty"

**Pass Criteria**:
- ✅ All 4 rejection scenarios return correct HTTP status
- ✅ Error messages are clear and actionable
- ✅ No partial uploads in filesystem

---

### Test Case 1.3: Database Consistency
**Scenario**: Verify payment data integrity across tables

**Checks**:
1. No orphaned payments (payment_id references non-existent user)
2. No duplicate approvals (same payment approved twice)
3. Subscription dates make sense (activated_at > created_at)
4. Audit log completeness (every approval logged)

**SQL Validation**:
```sql
-- Check orphaned payments
SELECT p.* FROM payments p 
WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = p.user_id);

-- Check duplicate approvals
SELECT payment_id, COUNT(*) as count FROM payment_audit_log 
WHERE action = 'approve' GROUP BY payment_id HAVING COUNT(*) > 1;

-- Check date consistency
SELECT * FROM ultra_subscriptions 
WHERE activated_at < created_at;
```

---

## 🖥️ Module 2: Ultra Terminal & Sandbox Tests

### Test Case 2.1: Basic Terminal Commands
**Scenario**: Ultra user executes safe commands in sandbox

**Commands to test**:
```bash
echo "Hello World"
ls -la
pwd
whoami
cat /etc/hostname
date
uname -a
```

**Expected**:
- ✅ All commands execute in < 2 seconds
- ✅ Output matches expected values
- ✅ No access to host filesystem (pwd shows `/sandbox/user_${id}`)
- ✅ Session remains active for 30 minutes

---

### Test Case 2.2: Complex Script Execution
**Scenario**: Run multi-line bash script

**Test script**:
```bash
#!/bin/bash
mkdir test_dir
cd test_dir
for i in {1..5}; do
    echo "File $i" > file_$i.txt
done
ls -la
tar -czf archive.tar.gz *.txt
du -sh archive.tar.gz
```

**Expected**:
- ✅ Script runs without errors
- ✅ All files created
- ✅ Archive created successfully
- ✅ Execution time < 5 seconds

---

### Test Case 2.3: Sandbox Isolation
**Scenario**: Verify sandboxed commands cannot escape container

**Attacks to test**:
1. `cat /etc/passwd` → Should show empty or restricted version
2. `../../../etc/hostname` → Should fail or show sandbox value
3. Access parent directory: `cd ../` → Should fail
4. Read host volumes: `mount` → Should show only sandbox mounts

**Pass Criteria**:
- ✅ Cannot read sensitive host files
- ✅ Cannot access parent directories
- ✅ Cannot modify system files
- ✅ Cannot start privileged processes

---

### Test Case 2.4: Concurrent Session Performance
**Scenario**: Open 5 simultaneous Terminal sessions

**Metrics to monitor**:
```bash
# In one terminal, run:
watch -n 1 'docker stats mr_darkpromth_api --no-stream'

# In another, spawn 5 sessions:
for i in {1..5}; do
    curl -X POST http://localhost:3000/api/ultra/terminal \
      -H "Authorization: Bearer $TOKEN" \
      -d "command=sleep 30" &
done
```

**Performance Targets**:
- ✅ RAM increase < 500MB per session
- ✅ CPU usage stays < 80%
- ✅ Response time < 500ms per command
- ✅ No session crashes

---

## 💬 Module 3: Chat & AI Failover Tests

### Test Case 3.1: Normal Chat Flow
**Scenario**: Send several chat messages and verify history

**Test messages**:
1. Simple: "Hello, who are you?"
2. Complex prompt: "Explain quantum computing in simple terms"
3. Multi-turn: Follow-up to previous answer

**Expected**:
- ✅ Responses from Cerebras API within 10 seconds
- ✅ Responses coherent and relevant
- ✅ History saved in database
- ✅ User can retrieve full conversation

**Verification**:
```sql
SELECT * FROM chat_messages WHERE user_id = $1 ORDER BY created_at DESC LIMIT 10;
SELECT * FROM chat_sessions WHERE user_id = $1 ORDER BY created_at DESC LIMIT 3;
```

---

### Test Case 3.2: Failover to OpenRouter
**Scenario**: Simulate Cerebras API failure, verify fallback works

**Setup**:
1. Edit `.env`: Set `CEREBRAS_API_KEYS` to invalid key (e.g., "invalid_key_12345")
2. Keep `OPENROUTER_API_KEYS` valid
3. Send chat message
4. Monitor logs during failover

**Expected**:
- ✅ First attempt fails with invalid Cerebras key
- ✅ System logs failover warning
- ✅ Second attempt uses OpenRouter
- ✅ User receives response from OpenRouter
- ✅ All within 15 seconds total

**Log patterns to check**:
```
[ERROR] Cerebras API failed: Invalid API key
[WARN] Attempting failover to OpenRouter...
[INFO] Using OpenRouter API for chat
```

---

### Test Case 3.3: API Key Rotation
**Scenario**: Verify system handles multiple API keys correctly

**Setup**:
```env
CEREBRAS_API_KEYS=key1,key2,key3,key4
OPENROUTER_API_KEYS=okey1,okey2
```

**Test**:
1. Send 4 chat messages (should rotate through all Cerebras keys)
2. Check logs for key rotation pattern
3. Verify no key is used twice in a row

**Expected**:
- ✅ Keys rotate sequentially
- ✅ No key exhaustion
- ✅ Proper logging of key usage

---

## 🔒 Module 4: Security & Load Testing

### Test Case 4.1: Tier Authorization
**Scenario**: Free tier user cannot access Ultra features

**Restricted endpoints** (should return 403):
```
POST /api/ultra/terminal
POST /api/ultra/sandbox/execute
GET /api/ultra/features/advanced
```

**Test setup**:
1. Create Free tier user
2. Get auth token
3. Try accessing each endpoint

**Expected**:
- ✅ All return 403 Forbidden
- ✅ Response: `{"error": "Insufficient tier", "required": "ultra"}`
- ✅ No sensitive leakage in error messages

---

### Test Case 4.2: Rate Limiting
**Scenario**: Verify rate limits protect API

**Test**:
```bash
# Hammer /health endpoint with 100 requests
ab -n 100 -c 10 http://localhost:3000/health

# Check response codes - should have 429 Too Many Requests
```

**Expected**:
- ✅ Most requests succeed (200)
- ✅ Some requests hit 429 after limit
- ✅ Rate limit resets after timeout (usually 60 seconds)

---

### Test Case 4.3: Load Test - API Endpoints
**Scenario**: Simulate production load

**Test 1: Health Check (baseline)**
```bash
ab -n 200 -c 20 http://localhost:3000/health
```

**Targets**:
- Response time: < 100ms (p99)
- Success rate: 100%

**Test 2: Chat Endpoint**
```bash
# See load-tests/load-test-api.js
npm run load-test
```

**Targets**:
- Response time: < 3s (p99)
- Success rate: > 99.5%
- Throughput: > 20 req/s

---

### Test Case 4.4: Nginx Configuration Validation
**Scenario**: Verify Nginx is correctly configured

**Checks**:
1. SSL certificate is valid
   ```bash
   openssl s_client -connect localhost:443 </dev/null 2>/dev/null | grep "Verify return code"
   ```
2. HTTP → HTTPS redirect works
   ```bash
   curl -I http://localhost/ 2>&1 | grep -i location
   ```
3. File upload limit matches code (20MB)
   ```bash
   grep "client_max_body_size" /opt/mrdarkpromth/nginx/nginx.production.conf
   ```

**Expected**:
- ✅ SSL: "Verify return code: 0 (ok)"
- ✅ HTTP redirects to HTTPS
- ✅ client_max_body_size = 20M

---

## 🛠️ Test Execution Guide

### Pre-Test Checklist
- [ ] Database backed up
- [ ] .env file configured correctly
- [ ] Docker containers running: `docker-compose ps`
- [ ] No existing test users in database
- [ ] API logs cleared for easier monitoring

### Running Tests

```bash
# Terminal 1: Monitor logs
docker-compose logs -f api | grep -E "ERROR|WARN|failover"

# Terminal 2: Monitor metrics
watch -n 1 'docker stats --no-stream'

# Terminal 3: Run tests
cd /opt/mrdarkpromth/tests
bash run_all_tests.sh
```

---

## 📊 Test Results Template

### Summary
```
Total Tests: __
Passed: __
Failed: __
Success Rate: __%

Critical Issues: __
High Issues: __
Low Issues: __
```

### Module 1 Results
- [ ] Test 1.1 (Happy Path): PASS / FAIL
- [ ] Test 1.2 (Invalid Upload): PASS / FAIL
- [ ] Test 1.3 (DB Consistency): PASS / FAIL

### Module 2 Results
- [ ] Test 2.1 (Basic Commands): PASS / FAIL
- [ ] Test 2.2 (Complex Script): PASS / FAIL
- [ ] Test 2.3 (Sandbox Isolation): PASS / FAIL
- [ ] Test 2.4 (Concurrent Sessions): PASS / FAIL

### Module 3 Results
- [ ] Test 3.1 (Chat Flow): PASS / FAIL
- [ ] Test 3.2 (Failover): PASS / FAIL
- [ ] Test 3.3 (Key Rotation): PASS / FAIL

### Module 4 Results
- [ ] Test 4.1 (Tier Authorization): PASS / FAIL
- [ ] Test 4.2 (Rate Limiting): PASS / FAIL
- [ ] Test 4.3 (Load Test): PASS / FAIL
- [ ] Test 4.4 (Nginx Config): PASS / FAIL

---

## 🚨 Identified Issues & Fixes

Document any issues found during testing:

| Issue | Severity | Reproduction Steps | Fix Applied | Status |
|-------|----------|-------------------|-------------|--------|
| Example | High | Step 1, 2, 3 | Modified file.rs line 123 | ✅ Fixed |

---

## ✅ Sign-off Checklist (before distribution)

- [ ] All 15+ tests passed
- [ ] No critical issues pending
- [ ] Database in clean state (test data removed)
- [ ] API key pool verified (40+ valid keys)
- [ ] SSL certificate valid (> 30 days remaining)
- [ ] Domain DNS resolution verified
- [ ] Monitoring alerts properly configured
- [ ] Backup system tested and working
- [ ] Load test shows acceptable performance
- [ ] Security tests all pass
- [ ] Failover tested and working
- [ ] Documentation updated
- [ ] Team sign-off obtained

---

## 📞 Support Contact

If tests fail, check:
1. **Logs**: `docker-compose logs -f api`
2. **Database**: `psql -U postgres -d mrdarkpromth -c "\dt"`
3. **Network**: `curl -v http://localhost:3000/health`
4. **Configuration**: Check `/opt/mrdarkpromth/.env`

For persistent issues, escalate with full logs and test output.
