# CLAUDE.md — MR.DarkPromth

> Ultra Tier AI Platform — Rust + React full-stack with tiered access (Free / Premium / Ultra), AI code generation, sandboxed execution, and jailbreak prompt management.

## Domain

- **Production domain:** `mrdarkpromth.online`
- **GitHub redirect URI** (`.env`): `https://mrdarkpromth.online/auth/github/callback`

> **✅ Domain migration complete:** All config files, scripts, nginx, Docker Compose, frontend, and backend now reference `mrdarkpromth.online`.

---

## Quick Commands

### Backend (Rust workspace — root: `/opt/mrdarkpromth`)

```bash
cargo build --release          # Build all crates
cargo run --bin mr_darkpromth_api  # Run API server (needs .env)
cargo test                     # Run all tests
cargo check                    # Type-check without building
cargo clippy -- -D warnings    # Lint
cargo fmt --all                # Format
```

### Frontend (`/opt/mrdarkpromth/frontend`)

```bash
pnpm install && pnpm dev       # Dev server on :5173
pnpm build                     # Production build (tsc -b && vite build)
pnpm lint                      # ESLint
pnpm e2e                       # Playwright E2E tests
```

### Docker

```bash
# Dev stack
docker-compose build && docker-compose up -d

# Production stack (includes monitoring, certbot, exporters)
docker-compose -f docker-compose.production.yml up -d

# Logs
docker-compose logs -f api

# Migrations
docker-compose exec api /bin/sh -c "cargo sqlx migrate run"
```

---

## Rust Workspace Structure

Root workspace at `/opt/mrdarkpromth/Cargo.toml`, resolver v2.

| Crate | Path | Binary/Lib | Purpose |
|---|---|---|---|
| `mr_darkpromth_api` | `mr_darkpromth/api/` | **Binary** (`mr_darkpromth_api`) | Axum 0.8 HTTP server, routing, middleware, handlers |
| `mr_darkpromth_core` | `mr_darkpromth/core/` | Library | Auth, encryption, security, tier logic, agents, session management |
| `mr_darkpromth_services` | `mr_darkpromth/services/` | Library | Billing, user management, AI integration, sandboxing, email, tools |
| `mr_darkpromth_db` | `mr_darkpromth/db/` | Library | SQLx database layer |
| `cerebras_client` | `mr_darkpromth/services/cerebras_client/` | Library | Cerebras AI HTTP client |

**Dependency chain:** `api` → `services` + `core` + `db` → `core` → (standalone)

### Key Dependency Versions (from actual Cargo.toml files)

| Dependency | Version | Notes |
|---|---|---|
| `axum` | 0.8 | With `json`, `macros`, `ws` features |
| `sqlx` | 0.8 | `runtime-tokio-rustls`, `postgres`, `uuid`, `chrono`, `bigdecimal`, `macros` |
| `tokio` | 1.0 | Full features |
| `tower-http` | 0.5 | `cors`, `trace` |
| `tower_governor` | 0.4 | Rate limiting (api only) |
| `redis` | 0.25 | `tokio-comp` |
| `reqwest` | 0.12 | `json`, `rustls-tls`, `stream` |
| `jsonwebtoken` | 9.3 | JWT |
| `argon2` | 0.5 | Password hashing |
| `serde` | 1.0 | With `derive` |
| `bigdecimal` | 0.4 | With `serde` (services only) |
| `lettre` | 0.11 | Email (services only) |
| `qrcode` | 0.12 | QR generation (services only) |

### API Handler Modules (`mr_darkpromth/api/src/handlers/`)

| File | Scope |
|---|---|
| `auth.rs` | Login, register, refresh, logout, email verification, password reset |
| `github.rs` | GitHub OAuth (URL, callback, link/unlink, profile) |
| `chat_billing.rs` | Chat, billing plans, QR, slip verification, subscription, payment history |
| `tools_jailbreak.rs` | Tool listing/execution, sandbox, jailbreak prompts CRUD/search |
| `ultra_terminal.rs` | Terminal execute, Ultra terminal sessions |
| `ultra_terminal_ws.rs` | WebSocket-based Ultra terminal |
| `admin.rs` | User management, metrics, docs, telemetry, dashboard |
| `learning.rs` | Learning insights, metrics, patterns |

### API Routes (from `axum_router.rs`)

**Public (no auth):**
- `GET /health`, `GET /`, `GET /metrics`
- `GET /api/auth/github/url`, `POST /api/auth/github/callback`
- `GET /api/users/:id`, `GET /api/billing/plans`, `GET /api/billing/plans/:id`

**Auth (rate-limited, no JWT):**
- `POST /api/auth/register`, `POST /api/auth/login`, `POST /api/auth/refresh`, `POST /api/auth/logout`
- `POST /api/auth/verify-email`, `POST /api/auth/resend-verification`
- `POST /api/auth/request-password-reset`, `POST /api/auth/reset-password`

**Protected (JWT required):**
- User: `/api/auth/me`, `/api/users/me/*`
- Chat: `POST /api/chat`, `GET /api/chat/history`
- Billing: `/api/billing/generate-qr`, `/api/billing/verify-slip`, `/api/billing/subscription`, `/api/billing/history`
- Jailbreak: `/api/jailbreak/prompts/*`
- Tools: `/api/tools`, `/api/tools/execute`, `/api/sandbox/execute`, `/api/sandbox/sessions/*`
- Terminal (Ultra): `/api/terminal/execute`, `/api/terminal/ultra/*`
- Admin (Premium+): `/api/admin/*`
- Learning: `/api/learning/*`

### Core Modules (`mr_darkpromth/core/src/` — 28 files)

Key: `auth.rs`, `encryption.rs`, `jailbreak_safety.rs`, `output_filter.rs`, `rate_limiter.rs`, `session_manager.rs`, `terminal.rs`, `tier.rs`, `github_oauth.rs`, `api_key_manager.rs`, `security_audit.rs`, `security_monitor.rs`

### Services (`mr_darkpromth/services/src/` — 68 files)

Key: `billing_service.rs`, `user_service.rs`, `cerebras_integration.rs`, `openrouter_client.rs`, `email_service.rs`, `sandboxed_execution.rs`, `tool_system.rs`, `ultra_tier_logic.rs`, `tier_management.rs`, `monitoring.rs`, `jailbreak_service.rs`

---

## Frontend

**Stack:** React 19 + Vite 7 + TypeScript 5.9 + Tailwind CSS 4 + Zustand + React Router 7

### Config

| File | Value |
|---|---|
| `frontend/.env` | `VITE_API_BASE_URL=https://mrdarkpromth.online` |
| `frontend/.env.production` | `VITE_API_BASE_URL=https://mrdarkpromth.online` |
| `frontend/src/lib/api.ts` | Axios `baseURL` fallback: `https://mrdarkpromth.online` |
| `vite.config.ts` | `@` alias → `./src` |

### Routing (`App.tsx`)

| Path | Component | Access |
|---|---|---|
| `/` | `LandingPage` | Public |
| `/login` | `Login` | Public |
| `/register` | `Register` | Public |
| `/login/github` | `GitHubLogin` | Public |
| `/auth/verify-email` | `EmailVerification` | Public |
| `/auth/password-reset` | `PasswordReset` | Public |
| `/app` | `HackerDashboard` (index) | Authenticated |
| `/app/chat` | `Chat` | Authenticated |
| `/app/sandbox` | `Sandbox` | Authenticated |
| `/app/tools` | `Tools` | Authenticated |
| `/app/jailbreak` | `Jailbreak` | Authenticated |
| `/app/terminal` | `Terminal` | **Ultra only** |
| `/app/admin` | `Admin` | **Premium+** |
| `/app/admin/dashboard` | `AdminDashboard` | **Premium+** |
| `/app/profile` | `Profile` | Authenticated |
| `/app/profile/settings` | `ProfileSettings` | Authenticated |
| `/app/billing` | `Billing` | Authenticated |

### Key Frontend Files

- **API client:** `src/lib/api.ts` (Axios singleton class with interceptors)
- **WebSocket:** `src/lib/websocket.ts`
- **Auth store:** `src/store/authStore.ts` (Zustand + persist)
- **Chat store:** `src/store/chatStore.ts`
- **i18n:** `src/lib/translations.ts` (Thai + English)
- **Language context:** `src/contexts/LanguageContext.tsx`
- **UI components:** `src/components/ui/` (5 components)

---

## Infrastructure

### Docker Compose Services

**Dev stack** (`docker-compose.yml`):

| Service | Image | Port |
|---|---|---|
| `postgres` | postgres:15-alpine | Internal only |
| `redis` | redis:7-alpine | Internal only |
| `api` | `mr_darkpromth_api:latest` | 8080 |
| `frontend` | Custom Dockerfile | Volume only |
| `nginx` | nginx:alpine | 80, 443 |

**Production stack** (`docker-compose.production.yml`) — adds:

| Service | Image | Port |
|---|---|---|
| `certbot` | certbot/certbot | — |
| `prometheus` | prom/prometheus | 127.0.0.1:9090 |
| `grafana` | grafana/grafana | 127.0.0.1:3001 |
| `postgres-exporter` | prometheuscommunity/postgres-exporter | — |
| `redis-exporter` | oliver006/redis_exporter | — |
| `nginx-exporter` | nginx/nginx-prometheus-exporter | — |
| `alertmanager` | prom/alertmanager | 127.0.0.1:9093 |
| `jaeger` | jaegertracing/all-in-one | 127.0.0.1:16686 |

**Note:** Production binds Postgres/Redis to `127.0.0.1` only. Monitoring services are also localhost-only.

### Nginx (`nginx/nginx.production.conf`)

- HTTP → HTTPS redirect
- Let's Encrypt ACME challenges at `/.well-known/acme-challenge/`
- Rate limiting zones: `api_limit` (10r/s), `auth_limit` (5r/s), `general_limit` (30r/s)
- Upstream: `mr_darkpromth_api:8080` with keepalive
- Security headers: HSTS, CSP, X-Frame-Options, X-Content-Type-Options
- WebSocket proxy at `/ws/`
- SPA fallback: `try_files $uri $uri/ /index.html`
- SSL: TLS 1.2/1.3, certs from `/etc/letsencrypt/`

### Database Migrations (`migrations/` — 14 files)

```
001 - Users and ultra tier tables
002 - Users table
003 - Audit tables
004 - Error and fix tables
005 - Correction history
006 - User tier enum upgrade
007 - Billing tables
008 - Email token tables
009 - Chat history tables
010 - Jailbreak prompt library
011 - Production optimization
012 - Seed test users
013 - Fix admin password hash
014 - Add language column (dated: 20260209)
```

### Config Files (`config/`)

- `default.toml`, `development.toml`, `production.toml` — Server, DB, Redis, logging, security, cerebras, agents, ultra_tier settings
- `logrotate.conf`, `logstash.conf` — Log management

### Monitoring (`monitoring/`)

- `prometheus.yml` — Scrapes API (:8080/metrics), nginx-exporter (:9113), self (:9090)
- `prometheus-alerts.yml` — Alert rules
- `alertmanager.yml` — Routes with severity-based routing (critical/warning)
- `grafana/` — Datasources, dashboards provisioning

---

## Environment Variables

### Root `.env` (actual current values reference)

| Variable | Source | Description |
|---|---|---|
| `CEREBRAS_API_KEYS` | `.env` | 20 comma-separated Cerebras keys |
| `OPENROUTER_API_KEYS` | `.env` | 20 comma-separated OpenRouter fallback keys |
| `JWT_SECRET` | `.env` | 64-char secret |
| `DATABASE_URL` | `.env` | `postgres://postgres:{PASS}@postgres:5432/mr_darkpromth` |
| `REDIS_URL` | `.env` | `redis://redis:6379` |
| `POSTGRES_PASSWORD` | `.env` | Database password |
| `GITHUB_CLIENT_ID` | `.env` | GitHub OAuth client ID |
| `GITHUB_CLIENT_SECRET` | `.env` | GitHub OAuth client secret |
| `GITHUB_REDIRECT_URI` | `.env` | `https://mrdarkpromth.online/auth/github/callback` |
| `CEREBRAS_MAX_RETRIES` | `.env` | `15` |
| `CEREBRAS_KEY_COOLDOWN_SECS` | `.env` | `30` |
| `RUST_LOG` | `.env` | `info,mr_darkpromth_api=info,http=info,warn` |

### Frontend `.env` variables

| Variable | Current Value |
|---|---|
| `VITE_API_BASE_URL` | `https://mrdarkpromth.online` |
| `VITE_WS_URL` | `wss://mrdarkpromth.online/ws` |
| `VITE_ENABLE_JAILBREAK` | `true` |
| `VITE_ENABLE_ANALYTICS` | `true` |
| `VITE_DEV_MODE` | `true` (dev) / `false` (prod) |

---

## Scripts (`scripts/` — 22 files)

| Script | Purpose |
|---|---|
| `deploy_vps.sh` | Full VPS deployment (env setup, SSL, docker-compose) |
| `create_env.sh` | Generate `.env` file with production values |
| `test_api_endpoints.sh` | Smoke test all API endpoints |
| `final_verification.sh` | Full system verification |
| `health_check.sh` | Quick health check |
| `production_monitor.sh` | Cron-based production monitoring |
| `enhanced_backup.sh` / `backup_database.sh` | Database backups |
| `setup_ssl_auto_renewal.sh` / `ssl-renewal.sh` | Let's Encrypt SSL |
| `check_ssl_renewal.sh` | SSL cert expiry check |
| `go_live_check.sh` | Pre-go-live checklist |
| `validate_cerebras_keys.sh` | Validate Cerebras API keys |
| `docker-logrotate.sh` | Docker log rotation |
| `setup_cron_jobs.sh` | Install production cron jobs |
| `seed_high_quality_prompts.sql` | Seed jailbreak prompts |

---

## AI Provider Failover

- **Primary:** Cerebras API (20 keys, model: `llama-3.3-70b`)
- **Fallback:** OpenRouter API (20 keys)
- Key rotation with `CEREBRAS_KEY_COOLDOWN_SECS=30` and `CEREBRAS_MAX_RETRIES=15`
- Implemented in `services/cerebras_integration.rs` + `services/openrouter_client.rs`

## Tier System

| Tier | Access |
|---|---|
| **Free** | Basic chat, jailbreak prompts |
| **Premium** | Tools, sandbox execution, admin panel |
| **Ultra** | Terminal, unrestricted AI generation, safety filter bypass |

Ultra tier bypasses `jailbreak_safety.rs` and `output_filter.rs` safety filters and removes terminal command restrictions.

## VSCode Extension

Located at `/opt/mrdarkpromth/vscode-extension/` — companion extension for the platform.
