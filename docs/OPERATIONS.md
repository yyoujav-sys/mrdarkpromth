# MR.DarkPromth Operations Manual

## Monitoring Stack

| Tool | URL | Purpose |
|------|-----|---------|
| Grafana | http://localhost:3000 | Dashboards & visualization |
| Prometheus | http://localhost:9090 | Metrics collection |
| Alertmanager | http://localhost:9093 | Alert routing |

### Grafana Dashboard

The production dashboard (`MR.DarkPromth Production Dashboard`) has 5 rows:
1. **Service Health** — API/Prometheus UP/DOWN, user & conversation counts
2. **API Performance** — request rate, latency percentiles, error rate, avg latency, rate limit rejections
3. **Nginx Traffic** — up/down, active connections, request rate
4. **Database & Cache** — PostgreSQL connections, Redis memory, Redis operations
5. **Active Alerts** — firing/pending alerts

### Prometheus Metrics

Custom metrics exposed at `GET /metrics`:

| Metric | Type | Description |
|--------|------|-------------|
| `mrdarkpromth_users_total` | gauge | Total users |
| `mrdarkpromth_conversations_total` | gauge | Total conversations |
| `mrdarkpromth_api_requests_total` | counter | Total API requests |
| `mrdarkpromth_api_requests_failed_total` | counter | Failed requests |
| `mrdarkpromth_api_avg_latency_ms` | gauge | Average latency |
| `mrdarkpromth_rate_limit_rejected_total` | counter | Rate-limited requests |
| `mrdarkpromth_rate_limit_rejected_auth_total` | counter | Auth rate limits |
| `mrdarkpromth_rate_limit_rejected_chat_total` | counter | Chat rate limits |

## Alert Rules (12 rules, 5 groups)

| Alert | Severity | Condition |
|-------|----------|-----------|
| `InstanceDown` | critical | Any target down >1min |
| `HighMemoryUsage` | warning | >85% memory |
| `HighCPUUsage` | warning | >80% CPU 5min |
| `APIHighLatency` | warning | p99 >2s for 5min |
| `APIHighErrorRate` | critical | >5% errors 5min |
| `PostgreSQLDown` | critical | DB down >1min |
| `PostgreSQLHighConnections` | warning | >80 connections |
| `RedisDown` | critical | Redis down >1min |
| `RedisHighMemoryUsage` | warning | Redis >256MB |
| `RateLimitExceeded` | warning | >10 rejections/min |
| `NginxHighErrorRate` | warning | >5% 5xx errors |

## Rate Limits

| Endpoint | Limit | Purpose |
|----------|-------|---------|
| Auth (login/register) | 5 req/min | Brute force prevention |
| Chat | 30 req/min | Resource protection |
| General | 100 req/min | Abuse prevention |

## Troubleshooting

### API not starting
```bash
docker logs mr_darkpromth_api --tail 50
# Check DATABASE_URL and REDIS_URL connectivity
```

### High error rate
```bash
# Check metrics
curl -s http://localhost:8080/metrics | grep failed
# Check recent logs
docker logs mr_darkpromth_api --since 5m 2>&1 | grep ERROR
```

### Database issues
```bash
# Check connections
docker exec mr_darkpromth_postgres psql -U postgres -c "SELECT count(*) FROM pg_stat_activity;"
# Run backup before any fix
./scripts/backup_database.sh
```
