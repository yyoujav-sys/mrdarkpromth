# 🛠️ MR.DarkPromth - Operations Manual

Welcome to the command center. This guide provides everything you need to run and maintain MR.DarkPromth efficiently.

## 1. Administrative Actions (Dashboard)
Access the admin tools at `https://mrdarkpromth.online/admin` (or through the profile menu when logged in as `admin`).

### Payment Verification
1. Go to **"Verify Slips"** tab.
2. Review the bank transfer slip uploaded by the user.
3. Click **"Approve"** to automatically upgrade the user to the correct Tier (Premium/Ultra) and activate their subscription.
4. Click **"Reject"** if the payment is invalid (add notes if necessary).

### User Management
- **Search**: Find users by email or username.
- **Suspend/Delete**: Use the action buttons to restrict access for malicious users.
- **Manual Upgrade**: You can manually change a user's Tier by clicking **"View Details"** (Eye icon).

---

## 2. Infrastructure Management (CLI)

### Service Status
Check if all systems are running:
```bash
cd /opt/mrdarkpromth
docker compose -f docker-compose.production.yml ps
```

### Logs & Debugging
Watch real-time API traffic or errors:
```bash
docker compose -f docker-compose.production.yml logs -f api
```

### Restart / Update
If you change environment variables or need a fresh start:
```bash
./deploy_production.sh
```

---

## 3. Database & Backups
Backups are stored in `/opt/mrdarkpromth/backups/`.

### Manual Backup
Run this before any major change:
```bash
cd /opt/mrdarkpromth
./scripts/backup_db.sh
```

### Restore Database
```bash
# Locate the backup file (e.g., pg_backup_2026.sql)
docker exec -i mr_darkpromth_postgres psql -U postgres -d mr_darkpromth < backups/your_backup.sql
```

---

## 4. AI Key Management
Keys are managed in the `.env.production` file.

### Adding New keys
1. Open the file: `nano /opt/mrdarkpromth/.env.production`
2. Add keys to `CEREBRAS_API_KEYS` (comma-separated).
3. Restart the API: `docker compose -f docker-compose.production.yml restart api`

---

## 5. Emergency Contacts & Monitoring
- **Health Check Status**: `https://mrdarkpromth.online/health`
- **Internal Metrics**: Accessible via Grafana at `http://localhost:3001` (if port-forwarded).

> [!IMPORTANT]
> Always keep a copy of your `.env.production` file in a secure, off-site location (e.g., a password manager).
