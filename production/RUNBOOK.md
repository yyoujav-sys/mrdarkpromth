# Mr.DarkPromth AI - Production Runbook

## System Overview
- **Domain**: https://bt-shop-dark.online
- **Server**: 150.95.31.224
- **Deploy Date**: $(date)

## Quick Commands

### Check System Status
```bash
docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'
curl -s http://localhost:8080/health
```

### View Logs
```bash
# API Logs
docker logs mr_darkpromth_api --tail 100 -f

# Database Logs
docker logs mr_darkpromth_postgres --tail 50

# All Services
docker-compose logs -f
```

### Restart Services
```bash
# Restart API
docker restart mr_darkpromth_api

# Restart All
docker-compose restart

# Full Redeploy
cd /opt/mrdarkpromth && docker-compose down && docker-compose up -d
```

## Backup & Recovery

### Manual Backup
```bash
/opt/mrdarkpromth/scripts/backup.sh
```

### Restore Database
```bash
# From backup file
docker exec -i mr_darkpromth_postgres psql -U postgres mr_darkpromth < backup_file.sql

# From compressed backup
gunzip -c db_backup_YYYYMMDD_HHMMSS.sql.gz | docker exec -i mr_darkpromth_postgres psql -U postgres mr_darkpromth
```

## Troubleshooting

### API Not Responding
1. Check container: `docker ps | grep api`
2. Check logs: `docker logs mr_darkpromth_api --tail 50`
3. Restart: `docker restart mr_darkpromth_api`

### Database Connection Failed
1. Check postgres: `docker ps | grep postgres`
2. Test connection: `docker exec mr_darkpromth_postgres pg_isready -U postgres`
3. Restart: `docker restart mr_darkpromth_postgres && sleep 5 && docker restart mr_darkpromth_api`

### SSL Certificate Issues
```bash
# Renew certificate
certbot renew --nginx
systemctl reload nginx
```

## Access URLs

| Service | URL | Credentials |
|---------|-----|-------------|
| Production | https://bt-shop-dark.online | - |
| API | https://bt-shop-dark.online/api | - |
| Grafana | http://150.95.31.224:3001 | admin/admin |
| Kibana | http://150.95.31.224:5601 | - |

## Support Contacts
- Email: support@mrdarkpromth.ai
- Emergency: +66-xxx-xxxx

## Maintenance Windows
- **Daily Backup**: 02:00 AM (UTC+7)
- **Health Checks**: Every 5 minutes
- **Log Rotation**: Weekly (Sunday 03:00 AM)
