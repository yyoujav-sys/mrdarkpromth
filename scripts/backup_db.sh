#!/bin/bash
# =============================================================================
# MR.DarkPromth — Automated PostgreSQL Backup Script
# Runs via cron every 6 hours. Retains backups for 7 days.
# =============================================================================
set -euo pipefail

# ── Configuration ────────────────────────────────────────────────────────────
BACKUP_DIR="/opt/mrdarkpromth/backups"
RETENTION_DAYS=7
CONTAINER_NAME="mr_darkpromth_postgres"
PG_USER="postgres"
PG_DB="mr_darkpromth"

DATE=$(date +%Y%m%d_%H%M%S)
DUMP_FILE="${BACKUP_DIR}/mr_darkpromth_${DATE}.sql.gz"
LOG_FILE="/var/log/mrdarkpromth/backup_db.log"

# ── Helpers ──────────────────────────────────────────────────────────────────
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# ── Pre-flight ───────────────────────────────────────────────────────────────
mkdir -p "$BACKUP_DIR"
mkdir -p "$(dirname "$LOG_FILE")"

# Verify the container is running
if ! docker inspect --format='{{.State.Running}}' "$CONTAINER_NAME" 2>/dev/null | grep -q true; then
    log "ERROR: Container '$CONTAINER_NAME' is not running. Aborting."
    exit 1
fi

# ── Dump ─────────────────────────────────────────────────────────────────────
log "Starting database backup → $DUMP_FILE"

if docker exec "$CONTAINER_NAME" pg_dump -U "$PG_USER" -d "$PG_DB" --no-owner --no-acl | gzip > "$DUMP_FILE"; then
    FILESIZE=$(du -h "$DUMP_FILE" | cut -f1)
    log "OK  Backup created: $DUMP_FILE ($FILESIZE)"
else
    log "FAIL Backup command failed!"
    rm -f "$DUMP_FILE"   # remove partial file
    exit 1
fi

# ── Integrity check ─────────────────────────────────────────────────────────
if gzip -t "$DUMP_FILE" 2>/dev/null; then
    log "OK  Integrity verified (gzip -t)"
else
    log "FAIL Integrity check failed!"
    exit 1
fi

# ── Retention ────────────────────────────────────────────────────────────────
DELETED=$(find "$BACKUP_DIR" -name "mr_darkpromth_*.sql.gz" -mtime +${RETENTION_DAYS} -print -delete | wc -l)
if [ "$DELETED" -gt 0 ]; then
    log "CLEANUP  Removed $DELETED backup(s) older than ${RETENTION_DAYS} days"
fi

# ── Summary ──────────────────────────────────────────────────────────────────
TOTAL=$(find "$BACKUP_DIR" -name "mr_darkpromth_*.sql.gz" | wc -l)
TOTAL_SIZE=$(du -sh "$BACKUP_DIR" 2>/dev/null | cut -f1)
log "SUMMARY  $TOTAL backup(s) on disk, total size: $TOTAL_SIZE"
