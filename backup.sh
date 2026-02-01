#!/bin/bash

BACKUP_DIR="/opt/mrdarkpromth/backups"
LOG_FILE="/var/log/backup.log"

log_message() {
    echo "[$(date)] $1" >> $LOG_FILE
    echo "[$(date)] $1"
}

backup_postgresql() {
    log_message "Starting PostgreSQL backup..."
    
    if docker exec mr_darkpromth_postgres pg_dump -U postgres mr_darkpromth > "$BACKUP_DIR/postgres_backup_$(date +%Y%m%d_%H%M%S).sql"; then
        log_message "PostgreSQL backup completed"
        gzip "$BACKUP_DIR/postgres_backup_$(date +%Y%m%d_%H%M%S).sql"
    else
        log_message "ERROR: PostgreSQL backup failed"
    fi
}

backup_redis() {
    log_message "Starting Redis backup..."
    
    docker exec mr_darkpromth_redis redis-cli BGSAVE > /dev/null 2>&1
    sleep 5
    
    if docker cp mr_darkpromth_redis:/data/dump.rdb "$BACKUP_DIR/redis_backup_$(date +%Y%m%d_%H%M%S).rdb"; then
        log_message "Redis backup completed"
        gzip "$BACKUP_DIR/redis_backup_$(date +%Y%m%d_%H%M%S).rdb"
    else
        log_message "ERROR: Redis backup failed"
    fi
}

cleanup_old_backups() {
    log_message "Cleaning up old backups..."
    find "$BACKUP_DIR" -name "*.gz" -mtime +7 -delete
}

main() {
    mkdir -p "$BACKUP_DIR"
    backup_postgresql
    backup_redis
    cleanup_old_backups
    log_message "Backup completed"
}

main
