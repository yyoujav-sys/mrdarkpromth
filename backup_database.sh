#!/bin/bash

# Automated Database Backup Script
# Backs up PostgreSQL database daily with rotation

set -e

BACKUP_DIR="/backups/databases"
DATE=$(date +%Y%m%d_%H%M%S)
DUMP_FILE="$BACKUP_DIR/mr_darkpromth_$DATE.sql.gz"
LOG_FILE="/var/log/database_backup.log"

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Initialize log if needed
touch "$LOG_FILE"

echo "[$(date '+%Y-%m-%d %H:%M:%S')] Starting database backup..." >> "$LOG_FILE"

# Create backup
if docker exec mr_darkpromth_postgres pg_dump -U postgres -d mr_darkpromth | gzip > "$DUMP_FILE" 2>> "$LOG_FILE"; then
    SIZE=$(du -sh "$DUMP_FILE" | cut -f1)
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ✅ Backup succeeded: $DUMP_FILE ($SIZE)" >> "$LOG_FILE"
    
    # Keep only last 7 days of backups
    find "$BACKUP_DIR" -name "mr_darkpromth_*.sql.gz" -mtime +7 -exec rm -f {} \;
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Cleaned backups older than 7 days" >> "$LOG_FILE"
    
    exit 0
else
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ❌ Backup FAILED" >> "$LOG_FILE"
    
    # Alert (in production, send email or alert)
    echo "Alert: Database backup failed on $(hostname)" | \
      mail -s "Database Backup Failed - $(hostname)" "admin@example.com" 2>/dev/null || true
    
    exit 1
fi
