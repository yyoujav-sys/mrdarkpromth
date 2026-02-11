#!/bin/bash

# MR.DarkPromth Backup Script

BACKUP_DIR="/opt/mrdarkpromth/backups"
LOG_FILE="/opt/mrdarkpromth/logs/backup.log"
DATE=$(date +%Y%m%d_%H%M%S)

log_message() {
    mkdir -p "$(dirname "$LOG_FILE")"
    echo "[$(date)] $1" >> "$LOG_FILE"
    echo "[$(date)] $1"
}

backup_postgresql() {
    log_message "Starting PostgreSQL backup..."
    local DUMP_FILE="$BACKUP_DIR/postgres_backup_${DATE}.sql"
    
    if docker exec mr_darkpromth_postgres pg_dump -U postgres mr_darkpromth > "$DUMP_FILE"; then
        gzip "$DUMP_FILE"
        log_message "PostgreSQL backup completed: ${DUMP_FILE}.gz"
    else
        log_message "ERROR: PostgreSQL backup failed"
        rm -f "$DUMP_FILE"
    fi
}

backup_redis() {
    log_message "Starting Redis backup..."
    local RDB_FILE="$BACKUP_DIR/redis_backup_${DATE}.rdb"
    
    docker exec mr_darkpromth_redis redis-cli -a "${REDIS_PASSWORD}" BGSAVE > /dev/null 2>&1
    sleep 5
    
    if docker cp mr_darkpromth_redis:/data/dump.rdb "$RDB_FILE"; then
        gzip "$RDB_FILE"
        log_message "Redis backup completed: ${RDB_FILE}.gz"
    else
        log_message "ERROR: Redis backup failed"
        rm -f "$RDB_FILE"
    fi
}

cleanup_old_backups() {
    log_message "Cleaning up backups older than 7 days..."
    find "$BACKUP_DIR" -name "*.gz" -mtime +7 -delete
}

main() {
    # Load environment
    if [ -f /opt/mrdarkpromth/.env ]; then
        source /opt/mrdarkpromth/.env
    fi
    
    mkdir -p "$BACKUP_DIR"
    backup_postgresql
    backup_redis
    cleanup_old_backups
    log_message "All backups completed"
}

main
