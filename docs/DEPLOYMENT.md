# MR.DarkPromth Deployment Guide

## Prerequisites

- Docker & Docker Compose v2
- 2GB RAM minimum, 4GB recommended
- Ports: 80, 443, 8080, 3000, 9090, 9093

## Quick Start

```bash
# Clone and configure
cd /opt/mrdarkpromth
cp .env.example .env  # Edit with production values

# Start all services
docker compose up -d

# Verify
curl http://localhost:8080/health
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection | `postgresql://postgres:password@postgres:5432/mr_darkpromth` |
| `REDIS_URL` | Redis connection | `redis://redis:6379` |
| `JWT_SECRET` | JWT signing secret | **Must set in production** |
| `PORT` | API listen port | `8080` |
| `RUST_LOG` | Log level | `info` |
| `LOG_FORMAT` | `json` for structured logs | human-readable |
| `CORS_ALLOWED_ORIGINS` | Comma-separated origins | `*` |
| `CEREBRAS_API_KEY_*` | LLM API keys (1-5) | — |

## Services

| Service | Port | Health |
|---------|------|--------|
| API | 8080 | `GET /health` |
| Nginx | 80/443 | — |
| PostgreSQL | 5432 | `pg_isready` |
| Redis | 6379 | `redis-cli ping` |
| Prometheus | 9090 | `GET /-/healthy` |
| Grafana | 3000 | `GET /api/health` |
| Alertmanager | 9093 | `GET /-/healthy` |

## Backup & Restore

### Backup
```bash
./scripts/backup_database.sh
# Creates compressed backup in /backups/databases/
# Automatic 7-day retention
```

### Restore
```bash
gunzip -c /backups/databases/mr_darkpromth_YYYYMMDD_HHMMSS.sql.gz | \
  docker exec -i mr_darkpromth_postgres psql -U postgres mr_darkpromth
```

## Rebuilding

```bash
# Rebuild API image (full compile, ~4min)
docker build -t mr_darkpromth_api:latest .

# Redeploy API only
docker compose up -d api

# Redeploy everything
docker compose up -d
```
