# MR.DARKPROMTH AI PLATFORM — MASTER PRODUCTION DIRECTIVE

## **1. MISSION OBJECTIVE**
You are the Lead Autonomous Engineer for **Mr.DarkPromth**, an elite AI platform. Your mission is to achieve **100% Production Readiness** and **Autonomous Operation**. You must complete all remaining integration work, wire all services end-to-end, and ensure the platform is fully functional for real users.

## **2. SYSTEM ARCHITECTURE & CONTEXT**
- **VPS Environment:** Ubuntu 22.04 (root@150.95.31.224) — 2 vCPU / 1.9GB RAM / 50GB Disk
- **Backend:** Rust (Axum) at `/opt/mrdarkpromth/mr_darkpromth` → 60+ API endpoints
- **Frontend:** React + TypeScript (Vite) at `/opt/mrdarkpromth/frontend` → 20 routes
- **VS Code Extension:** TypeScript at `/opt/mrdarkpromth/vscode-extension` → `.vsix` built
- **Infrastructure:** Docker Compose (10 containers: API, Frontend, PostgreSQL 15, Redis 7, Nginx, Prometheus, Grafana, Alertmanager, Postgres-exporter, Cloudflared)
- **Cloud Integrations:** Cloudflare (Tunnel ✅, R2 ⬜, KV ⬜), Slack (Bot ✅, Event Hooks ⬜)
- **Domains:** `mrdarkpromth.online`, `api.mrdarkpromth.online`, `www.mrdarkpromth.online`
- **Database:** PostgreSQL with 26 tables, 24 migrations applied
- **Source Control:** `https://github.com/Mr-darkpromth/MR.Darkpromth` (main branch)

## **3. CRITICAL CREDENTIALS**
All credentials are in `/opt/mrdarkpromth/.env.production`:
- **Slack Bot Token:** `xoxb-10465527208342-10498115268144-xoXAskqwJ97s57wOeY4QFVF7`
- **Slack User Token:** `xoxp-10465527208342-10495875455392-10504101708373-cc7616344dad800446a3d98517fa779b`
- **Slack App ID:** `A0ADN5EGTQD`
- **Slack Alerts Channel:** `C0AE4F0E76D` (#all-mrdarkpromth)
- **Slack Metrics Channel:** `C0ADRH80Z2N` (#new-channel)
- **Slack Admin User:** `U0AEKRRDDBJ`
- **Cloudflare API Token:** `fMcipxifxUkWi4c8VOMAkblvRC05pV0yb6q56ZKM`
- **Cloudflare Account ID:** `fca76f61c0f55855699d5387c1ca10eb`
- **GitHub Client ID:** `Ov23liZnNS4nh4RZg6gE`
- **Bot scopes:** `incoming-webhook, channels:join, channels:read, chat:write, groups:read, im:write`

## **4. CURRENT STATUS (Updated 2026-02-14)**

### ✅ Completed
- All 10 Docker containers running healthy
- Nginx hardened (rate limiting, security headers, hidden file deny, SSL/TLS)
- Cloudflare Tunnel active (4 connections, 3 domains routed)
- API health endpoint returns OK, 20 Cerebras + 20 OpenRouter keys loaded
- Database: 26 tables, 13 users, all migrations applied
- Frontend: 20 routes, all components built
- VS Code Extension: `.vsix` packaged (1.0.0)
- Slack bot: Token validated, joined channels, test messages sent successfully
- VPS disk optimized: 44% (27GB free)
- Git: Clean, all changes pushed to origin/main

### ⬜ Remaining Work (This Document)
- Phase 1: Slack Event Hook Integration (wire handlers → Slack notifications)
- Phase 2: Cloudflare R2 File Upload (wire storage endpoint)
- Phase 3: Cloudflare KV Caching (wire cache layer)
- Phase 4: Self-Correction Engine Activation
- Phase 5: WebSocket E2E Verification (Cloudflare Tunnel path)
- Phase 6: Slack Bot Scope Enhancement
- Phase 7: VS Code Extension Final Validation & Publish
- Phase 8: Full E2E Production Test

---

## **5. DEVELOPMENT ROADMAP — PHASED EXECUTION**

---

### **PHASE 1: SLACK EVENT HOOKS (CRITICAL)**
**Goal:** Wire all critical backend events to send real-time Slack notifications.

**Context:** `SlackService` exists in `services/src/slack_service.rs` with methods: `send_message()`, `send_formatted_message()`, `notify_alert()`, `notify_tier_upgrade()`, `notify_metrics()`, `notify_error()`, `notify_deployment()`, `dm_admin()`. The service is loaded into `AppState` (via `app_state.rs`). However, NO handler currently calls these methods.

**Tasks:**
1. **User Registration Notification** — In `api/src/handlers/auth.rs`, after successful `register()`, call:
   ```rust
   if let Some(ref slack) = state.slack_service {
       let _ = slack.send_formatted_message(
           &state.slack_alerts_channel_id,
           &format!("🆕 New user registered: {}", user.email),
       ).await;
   }
   ```

2. **Tier Upgrade Notification** — In `api/src/handlers/billing.rs`, after successful tier upgrade/approval:
   ```rust
   slack.notify_tier_upgrade(&user.email, &new_tier).await;
   ```

3. **Error Middleware** — In `api/src/middleware/` or `api/src/main.rs`, add a global error handler that calls:
   ```rust
   slack.notify_error(&error_message, &request_path).await;
   ```

4. **Login Tracking** — In `api/src/handlers/auth.rs` after `login()`, send:
   ```rust
   slack.send_message(&channel, &format!("👤 User logged in: {}", email)).await;
   ```

5. **Admin Actions** — In `api/src/handlers/admin.rs`, after approve/reject actions:
   ```rust
   slack.dm_admin(&format!("⚡ Admin action: {} on user {}", action, target_email)).await;
   ```

**Verification:**
```bash
# Register a test user and check Slack for notification
curl -X POST https://mrdarkpromth.online/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test_slack@test.com","password":"Test123!","username":"slacktest"}'
# Check #all-mrdarkpromth channel for "🆕 New user registered" message
```

---

### **PHASE 2: CLOUDFLARE R2 FILE UPLOAD (HIGH)**
**Goal:** Wire the existing `CloudflareService.upload_to_r2()` method to an API endpoint for file uploads.

**Context:** `services/src/cloudflare_service.rs` already has `upload_to_r2()`, `list_r2_buckets()`, and `get_from_r2()` methods. They use the Cloudflare API token. However, NO API endpoint calls these methods, and `CLOUDFLARE_R2_BUCKET` is empty in `.env.production`.

**Tasks:**
1. **Create R2 Bucket** — Using Cloudflare Dashboard or API, create a bucket named `mrdarkpromth-uploads`
2. **Update `.env.production`:**
   ```env
   CLOUDFLARE_R2_BUCKET=mrdarkpromth-uploads
   ```
3. **Create Upload Handler** — Create or update `api/src/handlers/upload.rs`:
   ```rust
   pub async fn upload_file(
       State(state): State<AppState>,
       claims: Claims,  // Auth required
       mut multipart: Multipart,
   ) -> Result<Json<UploadResponse>, AppError> {
       // Extract file from multipart
       // Call state.cloudflare_service.upload_to_r2(key, data, content_type).await
       // Return public URL
   }
   ```
4. **Register Route** — In `api/src/routes.rs`:
   ```rust
   .route("/api/upload", post(upload::upload_file))
   ```
5. **Frontend Integration** — Update `frontend/src/services/api.ts` to call `/api/upload`:
   ```typescript
   export async function uploadFile(file: File): Promise<string> {
       const formData = new FormData();
       formData.append('file', file);
       const response = await apiClient.post('/api/upload', formData);
       return response.data.url;
   }
   ```

**Verification:**
```bash
# Upload a test file
curl -X POST https://mrdarkpromth.online/api/upload \
  -H "Authorization: Bearer <JWT_TOKEN>" \
  -F "file=@test.png"
# Should return {"url":"https://mrdarkpromth-uploads.r2.cloudflarestorage.com/..."}
```

---

### **PHASE 3: CLOUDFLARE KV CACHING (HIGH)**
**Goal:** Implement caching layer using Cloudflare KV to reduce PostgreSQL load.

**Context:** `cloudflare_service.rs` has `put_kv()`, `get_kv()`, `delete_kv()`, `list_kv_keys()` methods. `CLOUDFLARE_KV_NAMESPACE_ID` exists but is empty.

**Tasks:**
1. **Create KV Namespace** — Via Cloudflare Dashboard, create namespace `mrdarkpromth-cache`
2. **Update `.env.production`:**
   ```env
   CLOUDFLARE_KV_NAMESPACE_ID=<namespace_id_from_dashboard>
   ```
3. **Cache User Tiers** — In `api/src/middleware/auth.rs`, add a KV cache check before DB query:
   ```rust
   // Check KV first: key = "user_tier:{user_id}"
   let cached = cloudflare.get_kv(&format!("user_tier:{}", user_id)).await;
   if let Ok(tier) = cached {
       return Ok(tier);
   }
   // Fall through to DB query, then cache result
   cloudflare.put_kv(&format!("user_tier:{}", user_id), &tier, Some(3600)).await;
   ```
4. **Cache Jailbreak Prompts** — In `api/src/handlers/jailbreak.rs`:
   ```rust
   // Cache popular prompts in KV with 1-hour TTL
   cloudflare.put_kv(&format!("jailbreak:{}", prompt_id), &prompt_json, Some(3600)).await;
   ```
5. **Cache Invalidation** — When tier changes or prompts are updated, delete KV key

**Verification:**
```bash
# Make same API call twice, second should be faster (from KV cache)
time curl -s https://mrdarkpromth.online/api/jailbreak/prompts -H "Authorization: Bearer <TOKEN>"
time curl -s https://mrdarkpromth.online/api/jailbreak/prompts -H "Authorization: Bearer <TOKEN>"
```

---

### **PHASE 4: SELF-CORRECTION ENGINE ACTIVATION (MEDIUM)**
**Goal:** Activate Agent 8 (Self-Correction) for autonomous error detection and recovery.

**Context:** `services/src/self_correction_engine.rs` and `services/src/agent8_main.rs` exist with full `ErrorDetector`, `ErrorEvent`, `FixGenerator` logic. `services/src/self_correction_agent.rs` defines the agent. The engine is NOT currently spawned at startup.

**Tasks:**
1. **Spawn Agent 8** — In `api/src/main.rs` after server starts:
   ```rust
   // Spawn self-correction engine as background task
   tokio::spawn(async move {
       let detector = ErrorDetector::new(error_sources);
       let mut interval = tokio::time::interval(Duration::from_secs(60));
       loop {
           interval.tick().await;
           if let Some(errors) = detector.scan().await {
               for error in errors {
                   let fix = FixGenerator::generate(&error).await;
                   // Log and optionally auto-apply for non-destructive fixes
                   info!("Self-correction detected: {:?}, fix: {:?}", error, fix);
               }
           }
       }
   });
   ```

2. **Connect to Log Sources** — Configure `ErrorDetector` to watch:
   - API error logs (from `tracing` output)
   - PostgreSQL connection failures
   - Redis connection failures
   - HTTP 500 responses

3. **Slack Integration** — When errors detected, send to Slack:
   ```rust
   slack.notify_alert(&format!("🤖 Agent 8 detected: {}", error.description)).await;
   ```

4. **Safety Limits:**
   - Only auto-apply fixes for: connection pool exhaustion (increase pool), expired cache (clear cache)
   - Alert-only for: compilation errors, configuration errors
   - Never auto-apply: database schema changes, service restarts

**Verification:**
```bash
# Check API logs for Agent 8 startup
docker logs mr_darkpromth_api 2>&1 | grep -i "self.correction\|agent.8\|error.detector"
# Intentionally trigger a DB connection error and check Slack for alert
```

---

### **PHASE 5: WEBSOCKET E2E VERIFICATION (MEDIUM)**
**Goal:** Verify WebSocket connectivity works end-to-end through Cloudflare Tunnel.

**Context:** Nginx has WebSocket proxy configured at `/ws/` location. Frontend has `useWebSocket.ts` hook. API has WebSocket handlers registered. But E2E flow through Cloudflare Tunnel has NOT been verified.

**Tasks:**
1. **Test Internal WebSocket:**
   ```bash
   # Test directly to API container
   docker exec mr_darkpromth_nginx curl -i -N \
     -H "Connection: Upgrade" \
     -H "Upgrade: websocket" \
     -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
     -H "Sec-WebSocket-Version: 13" \
     http://api:8080/ws/
   ```
   Should return `101 Switching Protocols`

2. **Test Through Nginx:**
   ```bash
   curl -i -N \
     -H "Connection: Upgrade" \
     -H "Upgrade: websocket" \
     -H "Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==" \
     -H "Sec-WebSocket-Version: 13" \
     http://localhost/ws/
   ```

3. **Test Through Cloudflare Tunnel:**
   ```bash
   # From external machine
   wscat -c wss://mrdarkpromth.online/ws/
   ```

4. **Fix if needed:**
   - Cloudflare Dashboard → Network → Enable WebSockets
   - Check `cloudflared` config for WebSocket support
   - Verify Nginx `proxy_read_timeout` is sufficient (default 60s may close WS)

5. **Frontend Verification:**
   - Open browser DevTools → Network → WS tab
   - Navigate to `https://mrdarkpromth.online/app/chat`
   - Verify WebSocket connection established (green status)

**Verification:**
```bash
# Install wscat if not available
npm install -g wscat
# Test external WebSocket
wscat -c wss://mrdarkpromth.online/ws/ -H "Authorization: Bearer <JWT_TOKEN>"
```

---

### **PHASE 6: SLACK BOT SCOPE ENHANCEMENT (LOW)**
**Goal:** Add missing OAuth scopes so the bot can create channels and manage workspace.

**Context:** Bot currently has: `incoming-webhook, channels:join, channels:read, chat:write, groups:read, im:write`. Missing: `channels:manage`, `chat:write.customize`, `reactions:write`.

**Tasks:**
1. Go to [Slack App Dashboard](https://api.slack.com/apps/A0ADN5EGTQD)
2. Navigate to **OAuth & Permissions** → **Bot Token Scopes**
3. Add scopes:
   - `channels:manage` — Create and archive channels
   - `chat:write.customize` — Send messages with custom username/avatar
   - `reactions:write` — Add emoji reactions to messages
   - `users:read` — Look up user info
4. Click **Reinstall to Workspace**
5. Copy the **new Bot User OAuth Token** (it will change after reinstall)
6. Update `/opt/mrdarkpromth/.env.production`:
   ```env
   SLACK_BOT_TOKEN=xoxb-NEW-TOKEN-HERE
   ```
7. Restart API:
   ```bash
   docker compose -f docker-compose.production.yml --env-file .env.production up -d api
   ```

**Verification:**
```bash
# Test channel creation with new token
curl -s -X POST https://slack.com/api/conversations.create \
  -H "Authorization: Bearer xoxb-NEW-TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"test-channel","is_private":false}'
```

---

### **PHASE 7: VS CODE EXTENSION VALIDATION & PUBLISH (LOW)**
**Goal:** Validate VS Code extension works correctly and prepare for marketplace publishing.

**Context:** `vscode-extension/mr-darkpromth-1.0.0.vsix` is built (21.6KB). Commands: OpenChat, Authenticate, ClearChat, RefreshUserInfo, OpenUltraTerminal, ToggleUltraMode.

**Tasks:**
1. **Install Extension:**
   ```bash
   code --install-extension /opt/mrdarkpromth/vscode-extension/mr-darkpromth-1.0.0.vsix
   ```

2. **Test Each Command:**
   - `Ctrl+Shift+P` → "MR.DarkPromth: Open Chat" → Should open Toggle Chat panel
   - "MR.DarkPromth: Authenticate" → Should prompt for API key/login
   - "MR.DarkPromth: Open Ultra Terminal" → Should open terminal (Ultra tier only)

3. **Verify API Connection:**
   - Extension should connect to `https://api.mrdarkpromth.online`
   - Check network tab for API calls
   - Verify authentication flow works

4. **Update Extension Config** (if needed):
   ```json
   {
     "mr-darkpromth.apiEndpoint": "https://mrdarkpromth.online",
     "mr-darkpromth.enableUltraMode": true
   }
   ```

5. **Prepare for VS Code Marketplace:**
   - Create publisher account at https://marketplace.visualstudio.com/manage
   - Update `package.json` with publisher name
   - Add icon and README
   - Run `vsce publish`

**Verification:**
- Open VS Code with extension installed
- Execute each command and verify functionality
- Check extension output panel for errors

---

### **PHASE 8: FULL E2E PRODUCTION TEST (FINAL)**
**Goal:** Run comprehensive end-to-end test of all user flows.

**Tasks:**
1. **Landing Page Test:**
   ```bash
   curl -s -o /dev/null -w "%{http_code}" https://mrdarkpromth.online
   # Expected: 200
   ```

2. **Registration Flow:**
   ```bash
   curl -X POST https://mrdarkpromth.online/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{"email":"e2e_test@test.com","password":"Test123!","username":"e2etest"}'
   # Expected: 200 with JWT token
   # Expected: Slack notification in #all-mrdarkpromth
   ```

3. **Login Flow:**
   ```bash
   curl -X POST https://mrdarkpromth.online/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email":"e2e_test@test.com","password":"Test123!"}'
   # Expected: 200 with JWT token
   ```

4. **Chat AI Test:**
   ```bash
   curl -X POST https://mrdarkpromth.online/api/chat \
     -H "Authorization: Bearer <JWT_TOKEN>" \
     -H "Content-Type: application/json" \
     -d '{"message":"Hello from E2E test","session_id":null}'
   # Expected: AI response
   ```

5. **Admin Dashboard:**
   ```bash
   curl -X GET https://mrdarkpromth.online/api/admin/dashboard \
     -H "Authorization: Bearer <ADMIN_JWT>"
   # Expected: Dashboard data
   ```

6. **Billing Flow:**
   ```bash
   # Upload slip → Admin approves → User tier becomes Ultra
   # Verify via: GET /api/users/profile
   ```

7. **WebSocket Real-time:**
   - Open browser → Login → Navigate to Chat
   - Open DevTools → Network → WS → Verify connection
   - Send chat message → Verify real-time delivery

8. **VS Code Extension:**
   - Open VS Code → Install extension → Authenticate → Chat

9. **Slack Verification:**
   - Check #all-mrdarkpromth: Registration, error, login notifications
   - Check #new-channel: Metrics notifications
   - Verify BlockKit formatting

10. **Security Scan:**
    ```bash
    # Verify no sensitive files exposed
    curl -s -o /dev/null -w "%{http_code}" https://mrdarkpromth.online/.env  # Expected: 404
    curl -s -o /dev/null -w "%{http_code}" https://mrdarkpromth.online/.git  # Expected: 404
    ```

---

## **6. OPERATIONAL DIRECTIVES**
- **Autonomous Mode:** You are authorized to modify any file, build containers, and restart services.
- **Git Protocol:** Always `git pull` before work and `git push` after successful testing.
- **Security First:** Never expose raw tokens in logs. Use `sqlx` for all DB interactions.
- **Docker Build:** `docker compose -f docker-compose.production.yml --env-file .env.production up -d --build`
- **Logging:** All major actions must be logged via `tracing` (structured logging).
- **Slack Channel Mapping:**
  - Alerts/Errors/Registrations → `C0AE4F0E76D` (#all-mrdarkpromth)
  - Metrics/Deployments → `C0ADRH80Z2N` (#new-channel)
  - Admin DMs → `U0AEKRRDDBJ`

## **7. EXECUTION COMMANDS**
```bash
# Build & Deploy
docker compose -f docker-compose.production.yml --env-file .env.production up -d --build

# Check Logs
docker logs -f mr_darkpromth_api

# Database Access
docker exec -it mr_darkpromth_postgres psql -U postgres -d mr_darkpromth

# Nginx Reload (after config change)
docker exec mr_darkpromth_nginx nginx -t && docker exec mr_darkpromth_nginx nginx -s reload

# Git Sync
git add -A && git commit -m "description" && git push origin main

# Slack Test
curl -s -X POST https://slack.com/api/chat.postMessage \
  -H "Authorization: Bearer xoxb-10465527208342-10498115268144-xoXAskqwJ97s57wOeY4QFVF7" \
  -H "Content-Type: application/json" \
  -d '{"channel":"C0AE4F0E76D","text":"Test message"}'
```

**EXECUTE PHASES IN ORDER: PHASE 1 → PHASE 8**
