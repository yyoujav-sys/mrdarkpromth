# MR.DarkPromth - Setup Guide

## Prerequisites

- Rust 1.70+ (for local development)
- Docker & Docker Compose
- PostgreSQL 15+
- Redis 7+
- Git

## Quick Start

### 1. Clone Repository

```bash
git clone <repository-url>
cd MR.Darkpromth
```

### 2. Environment Configuration

Copy the example environment file and configure:

```bash
cp .env.example .env
```

Edit `.env` with your values:
- `CEREBRAS_API_KEYS` - Your Cerebras.ai API keys (comma-separated)
- `JWT_SECRET` - Secret key for JWT token signing
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string

### 3. Start Services

Using Docker Compose (recommended):

```bash
docker-compose up -d
```

This starts:
- PostgreSQL (port 5432)
- Redis (port 6379)
- Backend service (port 8080)

### 4. Verify Setup

```bash
# Check health endpoint
curl http://localhost:8080/health

# Check database connection
docker-compose exec postgres psql -U postgres -d mr_darkpromth -c "SELECT 1"

# Check Redis connection
docker-compose exec redis redis-cli ping
```

### 5. Run Database Migrations

```bash
docker-compose exec postgres psql -U postgres -d mr_darkpromth -f /docker-entrypoint-initdb.d/001_create_ultra_tier_tables.sql
docker-compose exec postgres psql -U postgres -d mr_darkpromth -f /docker-entrypoint-initdb.d/002_create_users_table.sql
docker-compose exec postgres psql -U postgres -d mr_darkpromth -f /docker-entrypoint-initdb.d/003_create_audit_tables.sql
docker-compose exec postgres psql -U postgres -d mr_darkpromth -f /docker-entrypoint-initdb.d/003_create_jailbreak_prompt_library.sql
docker-compose exec postgres psql -U postgres -d mr_darkpromth -f /docker-entrypoint-initdb.d/004_create_correction_history_table.sql
docker-compose exec postgres psql -U postgres -d mr_darkpromth -f /docker-entrypoint-initdb.d/005_upgrade_user_tier_enum.sql
```

## Development Setup

### Install Dependencies

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Run with Hot Reload

```bash
cargo run
```

### Check Code Quality

```bash
cargo clippy --all-targets --all-features
cargo fmt --all -- --check
```

## Configuration

Configuration files are located in `config/`:

- `default.toml` - Default configuration
- `development.toml` - Development overrides
- `production.toml` - Production overrides

Environment variables override configuration files:

- `SERVER_HOST` - Server host (default: 0.0.0.0)
- `SERVER_PORT` - Server port (default: 8080)
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `RUST_LOG` - Logging level (debug, info, warn, error)
- `JWT_SECRET` - JWT signing secret
- `SANDBOX_USE_DOCKER` - Enable Docker-based sandbox execution (true/false)

## Project Structure

```
MR.Darkpromth/
├── mr_darkpromth/          # Rust workspace
│   ├── api/               # API layer
│   ├── core/              # Core types and utilities
│   ├── services/          # Business logic services
│   └── db/                # Database layer
├── src/                   # Root application entry point
├── migrations/            # Database migrations
├── memory/                # Agent memory logs
├── config/                # Configuration files
├── .github/workflows/     # CI/CD pipelines
├── docker-compose.yml     # Service orchestration
├── Dockerfile             # Container definition
└── .env.example           # Environment template
```

## Agent Coordination

Agents communicate via Redis Streams. Key channels:

- `mr_darkpromth:global:task_completion` - Task completion events
- `mr_darkpromth:global:resource_ready` - Resource readiness events
- `mr_darkpromth:system:heartbeat` - Agent heartbeat events
- `mr_darkpromth:{agent_id}:query_event` - Query events
- `mr_darkpromth:{agent_id}:response_event` - Response events

Event format:

```json
{
  "event_id": "uuid",
  "agent_id": "agent1",
  "event_type": "TaskCompletion|ResourceReady|Heartbeat|...",
  "timestamp": "ISO8601",
  "correlation_id": "uuid|null",
  "payload": { ... }
}
```

## Memory Logging

Each agent maintains a memory log in `memory/agentN_{name}.log`:

```
[TIMESTAMP] [ACTION] [DETAILS] [STATUS] {optional_json}
```

Example:

```
[2026-01-28T12:00:00.000Z] [COMPLETE] [Phase 1 setup] [SUCCESS] {"workspace":"mr_darkpromth"}
```

## CI/CD

The project uses GitHub Actions for CI/CD:

- `.github/workflows/ci.yml` - Continuous integration (tests, linting, security scan)
- `.github/workflows/deploy.yml` - Deployment to production

Workflows run on:
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches

## Troubleshooting

### Port Already in Use

```bash
# Check what's using the port
netstat -ano | findstr :8080

# Change port in .env
SERVER_PORT=8081
```

### Database Connection Failed

```bash
# Check PostgreSQL is running
docker-compose ps postgres

# Check logs
docker-compose logs postgres

# Restart service
docker-compose restart postgres
```

### Redis Connection Failed

```bash
# Check Redis is running
docker-compose ps redis

# Check logs
docker-compose logs redis

# Restart service
docker-compose restart redis
```

### Build Failures

```bash
# Clean build
cargo clean
cargo build

# Update dependencies
cargo update
```

## Next Steps

1. Review agent workflows in `Agent_*_Workflow.md`
2. Start implementing agent-specific features
3. Set up monitoring and alerting
4. Configure production deployment

## Support

For issues or questions:
- Check agent workflow documentation
- Review memory logs in `memory/` directory
- Check Redis event bus for coordination issues
- Review logs in `logs/` directory
