# MR.DarkPromth System Architecture

## Overview

MR.DarkPromth is an Ultra Tier AI Platform with unrestricted AI generation capabilities, built with a Rust backend and React frontend. The system supports multiple user tiers (Free, Premium, Ultra) with the Ultra tier having full unrestricted access including jailbreak prompts and terminal execution.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CLIENT LAYER                                    │
├─────────────────────────────────────────────────────────────────────────────┤
│  React Frontend (Vite)  │  VS Code Extension  │  External Clients           │
│  - TypeScript 5.7       │  - TypeScript       │  - REST API                 │
│  - Tailwind CSS v4      │  - Webview UI       │  - WebSocket                │
│  - Zustand 5.0          │  - Redis Pub/Sub    │                             │
│  - React Router v7      │                     │                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           GATEWAY LAYER                                      │
├─────────────────────────────────────────────────────────────────────────────┤
│  Nginx Reverse Proxy                                                         │
│  - Port 80/443 (HTTPS)                                                       │
│  - SSL/TLS Termination                                                       │
│  - Static file serving for frontend                                          │
│  - Load balancing                                                            │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           API LAYER (Rust/Actix)                             │
├─────────────────────────────────────────────────────────────────────────────┤
│  mr_darkpromth_api crate                                                     │
│  ├─ HTTP Routes (REST API)                                                   │
│  │   ├─ /health - Health checks                                              │
│  │   ├─ /api/auth/* - Authentication (register, login, logout, GitHub OAuth) │
│  │   ├─ /api/users/* - User management                                       │
│  │   ├─ /api/chat - AI chat completion                                       │
│  │   ├─ /api/tools/* - Tool execution                                        │
│  │   ├─ /api/terminal/execute - Terminal execution (Ultra only)              │
│  │   ├─ /api/jailbreak/* - Prompt library (filtered by tier)               │
│  │   ├─ /api/sandbox/* - Sandboxed code execution                           │
│  │   └─ /metrics - Prometheus metrics                                        │
│  ├─ WebSocket Handler (/ws/chat)                                             │
│  ├─ Middleware                                                               │
│  │   ├─ CORS handling                                                        │
│  │   ├─ JWT Authentication                                                   │
│  │   ├─ Rate Limiting                                                        │
│  │   ├─ Security Headers (HSTS, CSP)                                         │
│  │   └─ Tier-based access control                                            │
│  └─ OpenAPI/Swagger Documentation                                            │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        SERVICES LAYER (Rust)                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│  mr_darkpromth_services crate                                                │
│  ├─ Core Services                                                            │
│  │   ├─ UserService - User management, authentication                        │
│  │   ├─ JailbreakPromptService - Prompt library management                   │
│  │   ├─ UltraTierLogic - Ultra tier processing with jailbreak                │
│  │   ├─ CerebrasIntegration - AI model integration                           │
│  │   ├─ BillingService - Payment processing, QR generation                   │
│  │   ├─ EmailService - Email verification, password reset                    │
│  │   └─ CacheService - Redis caching                                         │
│  ├─ AI Integration                                                           │
│  │   ├─ KeyPool - Multiple API key management                                │
│  │   ├─ ResponseAnalyzer - AI response analysis                              │
│  │   └─ SafetyFilter - Content filtering (bypassed for Ultra)                │
│  ├─ Tool System (Agent 6)                                                    │
│  │   ├─ ToolRegistry - Plugin management                                     │
│  │   ├─ MasterToolExecutor - Tool execution                                  │
│  │   ├─ ToolSandbox - Isolated tool execution                                │
│  │   └─ ToolPermissions - Permission management                              │
│  ├─ Multi-Agent System (Agent 7)                                             │
│  │   ├─ AgentFramework - Agent orchestration                                 │
│  │   ├─ CoordinatorAgent - Task coordination                                 │
│  │   ├─ EditorAgent - Code editing                                           │
│  │   └─ TerminalAgent - Terminal operations                                  │
│  ├─ Self-Correction (Agent 8)                                                │
│  │   ├─ ErrorDetector - Error identification                                 │
│  │   ├─ FixGenerator - Automated fixes                                       │
│  │   └─ LearningSystem - Continuous improvement                              │
│  ├─ Security & Monitoring                                                    │
│  │   ├─ ServerProtection - Host protection                                    │
│  │   ├─ SandboxedExecution - Secure code execution                           │
│  │   ├─ Audit - Audit logging                                                │
│  │   ├─ Monitoring - System metrics                                          │
│  │   └─ TierValidation - Tier access control                                 │
│  └─ Redis Coordination                                                       │
│      ├─ RedisCoordinator - Event bus                                         │
│      └─ AgentCommunication - Inter-agent messaging                           │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        DATA LAYER                                            │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐              │
│  │   PostgreSQL    │  │     Redis       │  │   File System   │              │
│  │   (Port 5432)   │  │   (Port 6379)   │  │                 │              │
│  ├─────────────────┤  ├─────────────────┤  ├─────────────────┤              │
│  │ - users         │  │ - Session cache │  │ - memory/       │              │
│  │ - jailbreak_    │  │ - Rate limiting │  │   agent logs    │              │
│  │   prompts       │  │ - Event bus     │  │ - logs/         │              │
│  │ - prompt_usage_ │  │ - Coordination  │  │   app logs      │              │
│  │   records       │  │                 │  │ - certs/        │              │
│  │ - ultra_tier_*  │  │                 │  │   SSL certs     │              │
│  │ - audit_logs    │  │                 │  │                 │              │
│  │ - billing_*     │  │                 │  │                 │              │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘              │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                     MONITORING & LOGGING STACK                               │
├─────────────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│  │  Prometheus  │  │   Grafana    │  │Elasticsearch │  │   Kibana     │    │
│  │  (Port 9090) │  │  (Port 3000) │  │ (Port 9200)  │  │ (Port 5601)  │    │
│  ├──────────────┤  ├──────────────┤  ├──────────────┤  ├──────────────┤    │
│  │ - Metrics    │  │ - Dashboards │  │ - Log store  │  │ - Log viz    │    │
│  │ - Alerts     │  │ - Analytics  │  │ - Search     │  │ - Discovery  │    │
│  └──────────────┘  └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                                              │
│  ┌──────────────┐                                                            │
│  │   Jaeger     │                                                            │
│  │ (Port 16686) │                                                            │
│  ├──────────────┤                                                            │
│  │ - Tracing    │                                                            │
│  │ - Spans      │                                                            │
│  └──────────────┘                                                            │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Technology Stack

### Backend
- **Language**: Rust 1.75+
- **Web Framework**: Actix-web 4.4
- **Database**: PostgreSQL 15+ with SQLx
- **Cache**: Redis 7+
- **Authentication**: JWT with Argon2 password hashing
- **AI Integration**: Cerebras API with key pooling
- **Documentation**: OpenAPI 3.0 with Utoipa

### Frontend
- **Framework**: React 19.2 with TypeScript
- **Build Tool**: Vite 7.2
- **Styling**: Tailwind CSS 4.1
- **State Management**: Zustand 5.0
- **Routing**: React Router 7
- **Icons**: Lucide React
- **Markdown**: React Markdown + React Syntax Highlighter

### Infrastructure
- **Containerization**: Docker & Docker Compose
- **Reverse Proxy**: Nginx
- **Monitoring**: Prometheus + Grafana
- **Logging**: ELK Stack (Elasticsearch, Kibana)
- **Tracing**: Jaeger
- **SSL**: Self-signed certificates for local development

### External Services
- **AI Provider**: Cerebras (Llama 3.3 70B)
- **OAuth**: GitHub
- **Payment**: Thai Bank QR Code Generation
- **Email**: SMTP (configurable)

## Database Schema

### Core Tables
1. **users** - User accounts with tier-based access
2. **jailbreak_prompts** - Prompt library with categories and effectiveness
3. **prompt_usage_records** - Usage tracking and feedback
4. **ultra_tier_consents** - Ultra tier acceptance records
5. **ultra_tier_activations** - Ultra tier activation status
6. **audit_logs** - Security and action audit trail
7. **billing_plans** - Subscription plans
8. **payments** - Payment records
9. **subscriptions** - User subscriptions

### User Tiers
- **Free**: Basic access, rate limited
- **Premium**: Enhanced features, higher limits
- **Ultra**: Unrestricted access, jailbreak prompts, terminal execution

## API Endpoints

### Public Endpoints
- `GET /health` - Health check
- `POST /api/auth/register` - User registration
- `POST /api/auth/login` - User login
- `GET /api/auth/github/url` - GitHub OAuth URL
- `POST /api/auth/github/callback` - GitHub OAuth callback

### Authenticated Endpoints
- `GET /api/auth/me` - Current user info
- `PUT /api/users/me` - Update profile
- `POST /api/chat` - AI chat completion
- `GET /api/jailbreak/prompts` - List prompts (filtered by tier)
- `GET /api/billing/plans` - List subscription plans
- `POST /api/billing/generate-qr` - Generate payment QR

### Ultra Tier Only
- `POST /api/terminal/execute` - Execute terminal commands
- Access to all jailbreak prompts (requires_ultra_tier = true)
- Bypass all safety filters

### Admin Endpoints
- `GET /api/admin/users` - List all users
- `DELETE /api/admin/users/{id}` - Delete user
- `PUT /api/admin/users/{id}/status` - Update user status
- `GET /api/admin/metrics` - System metrics

## Security Features

1. **Authentication**: JWT tokens with secure cookie storage
2. **Password Hashing**: Argon2id
3. **CORS**: Whitelist-based origin validation
4. **Rate Limiting**: Tier-based request limits
5. **Security Headers**: HSTS, CSP, X-Frame-Options
6. **Input Validation**: SQL injection prevention via SQLx
7. **Sandboxed Execution**: Isolated tool/code execution
8. **Audit Logging**: All actions logged with user context

## Ultra Tier Features

1. **Unrestricted AI Generation**: All safety filters bypassed
2. **Jailbreak Prompt Library**: Access to DAN variations, system overrides
3. **Terminal Access**: Full shell command execution
4. **Priority Processing**: Faster response times
5. **No Rate Limits**: Unlimited requests

## Development Setup

### Prerequisites
- Docker Desktop
- Git
- 8GB+ RAM recommended

### Local URLs
- Frontend: `https://localhost:443`
- API: `https://localhost:8080`
- Grafana: `http://localhost:3000` (admin/admin)
- Prometheus: `http://localhost:9090`
- Kibana: `http://localhost:5601`
- Jaeger: `http://localhost:16686`

## Deployment

The system is containerized with Docker Compose including:
- Backend service (Rust)
- Frontend (Nginx serving built React app)
- PostgreSQL database
- Redis cache
- Prometheus metrics
- Grafana dashboards
- Elasticsearch logging
- Kibana visualization
- Jaeger tracing
- Nginx reverse proxy

---
*Last Updated: 2026-01-31*
*Version: 2.0 (Aligned with Actual System)*

---
