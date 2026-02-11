# 🚀 Distribution & Deployment Checklist

**Date**: February 11, 2026  
**Target**: Production deployment after passing all tests  
**Estimated Duration**: 1-2 hours (after tests pass)  

---

## 📋 Pre-Deployment Verification (Complete BEFORE going live)

### Domain & DNS
- [ ] **Domain DNS Resolution**
  ```bash
  nslookup mrdarkpromth.online
  # Should resolve to your VPS IP: 150.95.31.224
  ```

- [ ] **SSL Certificate Status**
  ```bash
  openssl s_client -connect mrdarkpromth.online:443 </dev/null 2>/dev/null | \
    grep -A2 "Verify return code"
  # Should show: Verify return code: 0 (ok)
  
  // OR check expiry
  echo | openssl s_client -servername mrdarkpromth.online -connect mrdarkpromth.online:443 2>/dev/null | \
    openssl x509 -noout -dates
  # notBefore and notAfter should show valid dates
  ```
  - [ ] Certificate expires > 30 days from now
  - [ ] Certificate matches domain name
  - [ ] Certificate is from trusted CA (not self-signed for production)

- [ ] **HTTPS Redirect**
  ```bash
  curl -vI http://mrdarkpromth.online/
  # Should redirect 301/302 to https://
  ```

---

### Database Integrity

- [ ] **Backup Database Before Deployment**
  ```bash
  cd /opt/mrdarkpromth
  ./backup.sh
  # Verify backup created
  ls -lh backups/ | head -5
  ```

- [ ] **Test Data Cleaned**
  ```bash
  psql -h localhost -U postgres -d mrdarkpromth -c "
    SELECT COUNT(*) as test_users FROM users WHERE email LIKE 'test_%@example.com';
  "
  # If > 0, delete test users before deployment
  DELETE FROM users WHERE email LIKE 'test_%@example.com';
  ```

- [ ] **Database Schema Validated**
  ```bash
  psql -h localhost -U postgres -d mrdarkpromth -c "\dt"
  # Verify all expected tables exist:
  # - users, ultra_subscriptions, payments, chat_messages, etc.
  ```

- [ ] **Migrations Applied**
  ```bash
  cd /opt/mrdarkpromth/mr_darkpromth
  cargo sqlx migrate info
  # All migrations should show "applied"
  ```

---

### API Key Pool & External Services

- [ ] **Cerebras API Keys Verified**
  ```bash
  # Check .env for CEREBRAS_API_KEYS
  grep "CEREBRAS_API_KEYS" /opt/mrdarkpromth/.env
  # Should have comma-separated list of 10-20 keys
  
  # Verify at least one works (test call)
  curl -X POST "https://api.cerebras.ai/v1/chat/completions" \
    -H "Authorization: Bearer your_key" \
    -H "Content-Type: application/json" \
    -d '{"model":"gpt-4","messages":[{"role":"user","content":"test"}]}'
  ```
  - [ ] At least 10 valid Cerebras keys
  - [ ] Each key has sufficient API credits

- [ ] **OpenRouter API Keys Verified**
  ```bash
  grep "OPENROUTER_API_KEYS" /opt/mrdarkpromth/.env
  # Should have backup keys
  
  # Test one key
  curl "https://openrouter.ai/api/v1/chat/completions" \
    -H "Authorization: Bearer your_openrouter_key" \
    -H "Content-Type: application/json" \
    -d '{"model":"gpt-3.5-turbo","messages":[{"role":"user","content":"test"}]}'
  ```
  - [ ] At least 3 valid OpenRouter keys
  - [ ] Keys have sufficient account balance

- [ ] **Redis Connection**
  ```bash
  redis-cli -h localhost ping
  # Should return: PONG
  ```

- [ ] **Email Service (if applicable)**
  ```bash
  # Check SMTP configuration in .env
  grep "SMTP_\|MAIL_" /opt/mrdarkpromth/.env
  # Verify credentials are set correctly
  ```

---

### Environment Configuration

- [ ] **Production .env Configured**
  ```bash
  # Verify .env has all required variables
  required_vars=(
    "DATABASE_URL"
    "REDIS_URL"
    "JWT_SECRET"
    "API_PORT"
    "CEREBRAS_API_KEYS"
    "OPENROUTER_API_KEYS"
    "RUST_LOG"
  )
  
  for var in "${required_vars[@]}"; do
    if ! grep -q "^$var=" /opt/mrdarkpromth/.env; then
      echo "❌ Missing: $var"
    fi
  done
  ```

- [ ] **No Debug/Test Settings in Production**
  ```bash
  # Check these should NOT be set for production:
  grep -i "DEBUG=true\|TEST=true\|SKIP_AUTH=true" /opt/mrdarkpromth/.env
  # Should return nothing
  ```

- [ ] **RUST_LOG Level Appropriate**
  ```bash
  grep "RUST_LOG" /opt/mrdarkpromth/.env
  # Production should be: "info" or "warn", NOT "debug" or "trace"
  ```

---

### Performance Baseline

- [ ] **Load Test Baseline**
  ```bash
  cd /opt/mrdarkpromth
  # Run baseline load test
  docker-compose logs -f api &
  ab -n 1000 -c 50 https://localhost/health
  
  # Record metrics:
  # - Requests per second: _____
  # - Mean response time: _____
  # - 95th percentile latency: _____
  ```

- [ ] **Database Query Performance**
  ```bash
  psql -h localhost -U postgres -d mrdarkpromth -c "
    EXPLAIN ANALYZE 
    SELECT * FROM chat_messages 
    WHERE user_id = 1 
    ORDER BY created_at DESC 
    LIMIT 50;
  "
  # Execution time should be < 100ms
  ```

- [ ] **Disk Space Check**
  ```bash
  df -h
  # Should have at least 20% free space on /
  # Should have at least 50GB free on /var/lib/docker
  ```

---

## 🔐 Security Hardening Checklist

- [ ] **Firewall Rules**
  ```bash
  # Only allow necessary ports:
  # - 80 (HTTP redirect)
  # - 443 (HTTPS)
  # - 22 (SSH, restricted to admin IPs)
  
  sudo ufw status
  # Verify only essential rules are enabled
  ```

- [ ] **SSH Hardening**
  ```bash
  cat /etc/ssh/sshd_config | grep -E "^[^#]" | head -20
  # Verify settings:
  # - PermitRootLogin no
  # - PasswordAuthentication no (key-based only)
  # - Port (non-standard if possible)
  ```

- [ ] **Docker Security**
  ```bash
  docker ps --format "table {{.Image}}\t{{.Status}}"
  # Verify only necessary containers are running
  # No unknown containers
  ```

- [ ] **Database User Privileges**
  ```bash
  psql -h localhost -U postgres -c "
    SELECT usename, usesuper FROM pg_user WHERE usename != 'postgres';
  "
  # Verify API user is NOT superuser
  ```

- [ ] **JWT Secret Strength**
  ```bash
  # Verify JWT_SECRET is long (> 32 characters) and random
  JWT_SECRET=$(grep "JWT_SECRET" /opt/mrdarkpromth/.env | cut -d'=' -f2)
  echo ${#JWT_SECRET}  # Should be > 32
  ```

---

## 📊 Monitoring & Alerts

- [ ] **Prometheus Running**
  ```bash
  curl http://localhost:9090/-/healthy
  # Should return 200
  ```

- [ ] **Grafana Access**
  ```bash
  curl http://localhost:3001
  # Should return login page (status 200)
  ```

- [ ] **Alert Manager Running**
  ```bash
  curl http://localhost:9093/-/healthy
  # Should return 200
  ```

- [ ] **Monitoring Alerts Configured**
  - [ ] High CPU alert (> 80%)
  - [ ] High memory alert (> 90%)
  - [ ] API failure alert (5 min downtime)
  - [ ] Database connection pool exhaustion
  - [ ] API key rotation needed

- [ ] **Log Aggregation (optional but recommended)**
  - [ ] ELK stack configured (or similar)
  - [ ] Logs flowing from API → Elasticsearch
  - [ ] Kibana dashboard accessible

---

## 🚀 Deployment Execution

### Step 1: Final Backup
```bash
cd /opt/mrdarkpromth
./backup.sh
# Verify: ls -lh backups/ | head -1
```

### Step 2: Pull Latest Changes (if applicable)
```bash
cd /opt/mrdarkpromth
git pull origin main  # or your deployment branch
git log --oneline -1
```

### Step 3: Build Docker Images
```bash
cd /opt/mrdarkpromth

# Build backend
docker-compose build api

# Build frontend
docker-compose build frontend
```

### Step 4: Stop Old Containers (with backup)
```bash
docker-compose down
# Data persists in volumes (postgres, redis)
```

### Step 5: Start Production Stack
```bash
# For full production stack:
docker-compose -f docker-compose.production.yml up -d

# OR for docker-compose based:
docker-compose up -d
```

### Step 6: Verify Deployment
```bash
# Wait 10 seconds for services to start
sleep 10

# Check all services running
docker-compose ps
# All containers should show "Up"

# Check API health
curl http://localhost:3000/health
# Should return {"status":"healthy"}

# Check database
docker-compose exec api /bin/sh -c \
  "pg_isready -h postgres -U postgres"
# Should return "accepting connections"

# Check logs for errors
docker-compose logs api | tail -20
# Should see "listening on" or "server started"
```

### Step 7: Smoke Tests
```bash
# Test basic endpoints
curl -s https://mrdarkpromth.online/health | jq .

# Test frontend loads
curl -s https://mrdarkpromth.online/ | head -20

# Test API key rotation working
# (Make a few API calls and check logs)
docker-compose logs api | grep -i "key\|provider"
```

---

## ✅ Post-Deployment Validation

Within 1 hour of deployment, verify:

- [ ] **API Responding**
  ```bash
  for i in {1..5}; do
    curl -s https://mrdarkpromth.online/health | jq .status
    sleep 1
  done
  # All should show "healthy" or similar
  ```

- [ ] **No Error Spikes in Logs**
  ```bash
  docker-compose logs api --tail=100 | grep -i "error\|panic"
  # Should see minimal/none
  ```

- [ ] **Database Growing (not shrinking)**
  ```bash
  psql -h localhost -U postgres -d mrdarkpromth -c "
    SELECT 
      schemaname,
      tablename,
      pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename))
    FROM pg_tables
    ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC
    LIMIT 10;
  "
  # Database size should be reasonable (not growing exponentially)
  ```

- [ ] **Monitoring Dashboards Updating**
  ```bash
  # Check Grafana
  firefox http://localhost:3001 &
  # Verify dashboard shows recent metrics (not stale)
  ```

- [ ] **User Reporting No Issues**
  - [ ] Test login works
  - [ ] Test payment upload works
  - [ ] Test chat functionality works
  - [ ] Test terminal (if applicable) works

---

## 🔄 Rollback Plan (if needed)

If deployment fails, follow this procedure:

```bash
cd /opt/mrdarkpromth

# 1. Stop new deployment
docker-compose down

# 2. Find latest backup
ls -lt backups/ | head -1
# Example: postgres_backup_20260211_123456.sql

# 3. Restore database
docker-compose up -d postgres redis
sleep 5

BACKUP_FILE="backups/postgres_backup_20260211_123456.sql"
docker-compose exec -T postgres \
  psql -U postgres < "$BACKUP_FILE"

# 4. Restart all services
docker-compose restart

# 5. Verify rolled back state
curl http://localhost:3000/health
```

---

## 📞 Support & Emergency Contacts

**If deployment goes wrong:**

1. **Check Logs First**
   ```bash
   docker-compose logs api | tail -100
   docker-compose logs postgres | tail -50
   ```

2. **Common Issues**:
   - Port 3000 in use: `sudo lsof -i :3000` and kill process
   - Database connection refused: Check `DB_HOST` in .env
   - API key errors: Verify `CEREBRAS_API_KEYS` in .env

3. **Emergency Contacts**:
   - Lead Dev: [contact info]
   - DevOps: [contact info]
   - Database Admin: [contact info]

---

## 📝 Sign-Off

**Deployment Details**:
- Date: __________________
- Time: __________________
- Deployer: __________________
- Reviewed by: __________________
- Status: [ ] Success [ ] Failed

**Notes**:
```
_________________________________________________________________
_________________________________________________________________
_________________________________________________________________
```

---

This checklist should be completed BEFORE marking the system as "Production Ready" and BEFORE announcing to users.
