#!/bin/bash
# MR.DarkPromth Automated Database Backup Script
# Creates compressed PostgreSQL backups with 7-day retention

set -euo pipefail

# Configuration
BACKUP_DIR="/backups/databases"
RETENTION_DAYS=7
DATE=$(date +%Y%m%d_%H%M%S)
DUMP_FILE="$BACKUP_DIR/mr_darkpromth_$DATE.sql.gz"
LOG_FILE="/var/log/mr_darkpromth_backup.log"

# PostgreSQL connection (uses Docker network)
PG_HOST="${POSTGRES_HOST:-postgres}"
PG_PORT="${POSTGRES_PORT:-5432}"
PG_USER="${POSTGRES_USER:-postgres}"
PG_DB="${POSTGRES_DB:-mr_darkpromth}"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Ensure backup directory exists
mkdir -p "$BACKUP_DIR"

log "Starting database backup..."
log "Target: $PG_USER@$PG_HOST:$PG_PORT/$PG_DB -> $DUMP_FILE"

# Create compressed backup
if docker exec mr_darkpromth_postgres pg_dump -U "$PG_USER" -d "$PG_DB" | gzip > "$DUMP_FILE"; then
    FILESIZE=$(du -h "$DUMP_FILE" | cut -f1)
    log "✅ Backup succeeded: $DUMP_FILE ($FILESIZE)"

    # Verify backup integrity (can decompress without errors)
    if gzip -t "$DUMP_FILE" 2>/dev/null; then
        log "✅ Backup integrity verified"
    else
        log "⚠️  Backup integrity check failed!"
        exit 1
    fi

    # Clean old backups (keep last N days)
    DELETED=$(find "$BACKUP_DIR" -name "mr_darkpromth_*.sql.gz" -mtime +$RETENTION_DAYS -print -delete | wc -l)
    if [ "$DELETED" -gt 0 ]; then
        log "🗑️  Cleaned $DELETED old backups (retention: ${RETENTION_DAYS} days)"
    fi

    # Show backup summary
    TOTAL=$(ls -1 "$BACKUP_DIR"/mr_darkpromth_*.sql.gz 2>/dev/null | wc -l)
    TOTAL_SIZE=$(du -sh "$BACKUP_DIR" | cut -f1)
    log "📊 Summary: $TOTAL backups, total size: $TOTAL_SIZE"
else
    log "❌ Backup FAILED!"
    exit 1
fi
