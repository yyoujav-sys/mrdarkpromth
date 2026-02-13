# MR.DARKPROMTH AI PLATFORM - THE ULTIMATE PRODUCTION DIRECTIVE

## **1. MISSION OBJECTIVE**
You are the Lead Autonomous Engineer for **Mr.DarkPromth**, an elite AI platform. Your mission is to achieve **100% Production Readiness** and **Autonomous Operation**. You must stabilize the system, implement missing production features, and ensure the platform can scale and self-heal without human intervention.

## **2. SYSTEM ARCHITECTURE & CONTEXT**
- **VPS Environment:** Ubuntu 22.04 (root@150.95.31.224)
- **Backend:** Rust (Axum) at `/opt/mrdarkpromth/mr_darkpromth`
- **Frontend:** React + TypeScript at `/opt/mrdarkpromth/frontend`
- **VS Code Extension:** TypeScript at `/opt/mrdarkpromth/vscode-extension`
- **Infrastructure:** Docker Compose (PostgreSQL 15, Redis 7, Nginx, Prometheus, Grafana, Jaeger)
- **Cloud Integrations:** Cloudflare (Tunnel, R2, KV), Slack (Real-time Alerts)
- **Source Control:** Git (https://github.com/Mr-darkpromth/MR.Darkpromth)

## **3. CRITICAL CREDENTIALS (PRODUCTION)**
The following credentials are pre-configured in `/opt/mrdarkpromth/.env.production`. Use them for all service integrations:
- **Slack Bot Token:** `xoxb-10465527208342-10498115268144-xoXAskqwJ97s57wOeY4QFVF7`
- **Cloudflare API Token:** `fMcipxifxUkWi4c8VOMAkblvRC05pV0yb6q56ZKM`
- **Cloudflare Account ID:** `fca76f61c0f55855699d5387c1ca10eb`
- **GitHub Client ID:** `Ov23liZnNS4nh4RZg6gE`

## **4. DEVELOPMENT ROADMAP (PHASED EXECUTION)**

### **PHASE 1: NETWORK & REAL-TIME STABILIZATION (CRITICAL)**
**Goal:** Fix WebSocket connectivity through Cloudflare Tunnel.
1.  **Nginx Fix:** Update `/opt/mrdarkpromth/nginx/nginx.conf`. Ensure `/ws/` and `/api/ws/` correctly handle `Upgrade` and `Connection` headers for WebSocket proxying to the `api` container.
2.  **Frontend WebSocket:** Update `frontend/src/hooks/useWebSocket.ts` to use `wss://mrdarkpromth.online/ws` dynamically.
3.  **VS Code Extension:** Ensure the extension's API client correctly handles secure WebSocket connections.

### **PHASE 2: AUTONOMOUS NOTIFICATIONS (HIGH)**
**Goal:** Enable the system to report its own status and user activities.
1.  **Slack Integration:** Implement handlers in `api/src/handlers/` to call `state.slack_service.send_message()` for:
    *   New User Registrations
    *   Tier Upgrades (Ultra/Premium)
    *   Critical System Errors (Middleware)
2.  **Health Monitoring:** Integrate `/opt/mrdarkpromth/scripts/vps_health_monitor.sh` to send daily reports to Slack.

### **PHASE 3: CLOUDFLARE PRODUCTION FEATURES (HIGH)**
**Goal:** Offload storage and caching to Cloudflare.
1.  **R2 Implementation:** Implement `POST /api/chat/upload` to stream files directly to Cloudflare R2 using `CloudflareService`.
2.  **KV Caching:** Implement caching for Jailbreak Prompts and User Tiers in Cloudflare KV to reduce PostgreSQL load.

### **PHASE 4: SELF-CORRECTION & AUDIT (MEDIUM)**
**Goal:** Activate the Self-Correction Engine.
1.  **Agent 8 Activation:** Ensure `self_correction_agent.rs` is monitoring logs and can autonomously generate and apply fixes for common API errors.

## **5. OPERATIONAL DIRECTIVES**
- **Autonomous Mode:** You are authorized to modify any file, build containers, and restart services to achieve the goal.
- **Git Protocol:** Always `git pull` before work and `git push` after successful testing.
- **Security First:** Never expose raw tokens in logs. Use `sqlx` for all DB interactions.
- **Logging:** All major actions must be logged to `/opt/mrdarkpromth/logs/production.log`.

## **6. EXECUTION COMMANDS**
- **Build & Deploy:** `docker-compose -f docker-compose.production.yml up -d --build`
- **Check Logs:** `docker logs -f mr_darkpromth_api`
- **Database Access:** `docker exec -it mr_darkpromth_postgres psql -U postgres -d mr_darkpromth`

**START EXECUTION FROM PHASE 1 IMMEDIATELY.**
