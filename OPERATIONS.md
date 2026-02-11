# MR.DarkPromth Production & Operations Checklist

## Quick Commands

### Health Check
```bash
/opt/mrdarkpromth/scripts/production_monitor.sh
```

### Service Management
```bash
# Restart all services
cd /opt/mrdarkpromth && docker compose restart

# View logs
docker logs mr_darkpromth_api --tail 100 -f

# Check container status
docker compose ps
```

### Database
```bash
# Connect to database
docker exec -it mr_darkpromth_postgres psql -U postgres -d mrdarkpromth

# Run backup
/opt/mrdarkpromth/scripts/enhanced_backup.sh
```

### SSL Certificate
```bash
# Check certificate expiry
/opt/mrdarkpromth/scripts/ssl-renewal.sh
```

## API Endpoints

| Endpoint | Method | Auth | Description |
|----------|--------|------|-------------|
| `/health` | GET | No | Health check |
| `/api/auth/register` | POST | No | User registration |
| `/api/auth/login` | POST | No | User login |
| `/api/chat` | POST | Yes | AI chat |
| `/api/billing/plans` | GET | No | List plans |
| `/api/billing/generate-qr` | POST | Yes | Generate payment QR |
| `/api/billing/verify-slip` | POST | Yes | Verify payment |
| `/api/admin/users` | GET | Admin | List users |

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `CEREBRAS_API_KEYS` | Primary AI provider (comma-separated) |
| `OPENROUTER_API_KEYS` | Failover AI provider (comma-separated) |
| `JWT_SECRET` | Token signing key |
| `DATABASE_URL` | PostgreSQL connection |
| `REDIS_URL` | Redis connection |

## Troubleshooting

### API Not Responding
```bash
docker restart mr_darkpromth_api
docker logs mr_darkpromth_api --tail 50
```

### Database Connection Failed
```bash
docker exec mr_darkpromth_postgres pg_isready -U postgres
docker restart mr_darkpromth_postgres
```

### High Memory Usage
```bash
docker stats --no-stream
docker system prune -f
```
