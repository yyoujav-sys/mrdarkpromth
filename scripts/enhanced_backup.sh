#!/bin/bash
# Enhanced Backup Script for MR.DarkPromth
# Features: Proper error handling, logging, notification support

set -euo pipefail

# Configuration
BACKUP_DIR="/opt/mrdarkpromth/backups"
LOG_FILE="/var/log/mrdarkpromth-backup.log"
RETENTION_DAYS=7
DATE_FORMAT=$(date +%Y%m%d_%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging function
log_message() {
    local level=$1
    local message=$2
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" >> "$LOG_FILE"
    
    case $level in
        "INFO")  echo -e "${GREEN}[$timestamp] $message${NC}" ;;
        "WARN")  echo -e "${YELLOW}[$timestamp] $message${NC}" ;;
        "ERROR") echo -e "${RED}[$timestamp] $message${NC}" ;;
        *)       echo "[$timestamp] $message" ;;
    esac
}

# Error handler
error_exit() {
    log_message "ERROR" "$1"
    # Send notification on failure (uncomment and configure as needed)
    # send_notification "Backup Failed: $1"
    exit 1
}

# Check prerequisites
check_prerequisites() {
    log_message "INFO" "Checking prerequisites..."
    
    # Check if Docker is running
    if ! docker info > /dev/null 2>&1; then
        error_exit "Docker is not running"
    fi
    
    # Check if PostgreSQL container is running
    if ! docker ps --format '{{.Names}}' | grep -q "mr_darkpromth_postgres"; then
        error_exit "PostgreSQL container is not running"
    fi
    
    # Check if Redis container is running
    if ! docker ps --format '{{.Names}}' | grep -q "mr_darkpromth_redis"; then
        log_message "WARN" "Redis container is not running, skipping Redis backup"
    fi
    
    # Create backup directory if not exists
    mkdir -p "$BACKUP_DIR"
}

# Backup PostgreSQL
backup_postgresql() {
    log_message "INFO" "Starting PostgreSQL backup..."
    local backup_file="$BACKUP_DIR/postgres_backup_${DATE_FORMAT}.sql"
    
    if docker exec mr_darkpromth_postgres pg_dump -U postgres mr_darkpromth > "$backup_file" 2>/dev/null; then
        gzip -f "$backup_file"
        local size=$(du -h "${backup_file}.gz" | cut -f1)
        log_message "INFO" "PostgreSQL backup completed: ${backup_file}.gz ($size)"
        
        # Verify backup
        if ! gzip -t "${backup_file}.gz" 2>/dev/null; then
            error_exit "PostgreSQL backup verification failed"
        fi
        log_message "INFO" "PostgreSQL backup verified successfully"
    else
        error_exit "PostgreSQL backup failed"
    fi
}

# Backup Redis
backup_redis() {
    if ! docker ps --format '{{.Names}}' | grep -q "mr_darkpromth_redis"; then
        log_message "WARN" "Skipping Redis backup - container not running"
        return 0
    fi
    
    log_message "INFO" "Starting Redis backup..."
    
    # Trigger Redis BGSAVE
    docker exec mr_darkpromth_redis redis-cli BGSAVE > /dev/null 2>&1 || true
    
    # Wait for background save to complete
    local max_wait=30
    local wait_count=0
    while docker exec mr_darkpromth_redis redis-cli LASTSAVE 2>/dev/null | grep -q "in progress"; do
        sleep 1
        ((wait_count++))
        if [ $wait_count -ge $max_wait ]; then
            log_message "WARN" "Redis BGSAVE taking too long, proceeding anyway"
            break
        fi
    done
    
    local backup_file="$BACKUP_DIR/redis_backup_${DATE_FORMAT}.rdb"
    if docker cp mr_darkpromth_redis:/data/dump.rdb "$backup_file" 2>/dev/null; then
        gzip -f "$backup_file"
        local size=$(du -h "${backup_file}.gz" | cut -f1)
        log_message "INFO" "Redis backup completed: ${backup_file}.gz ($size)"
    else
        log_message "WARN" "Redis backup failed (non-critical)"
    fi
}

# Backup application data
backup_application_data() {
    log_message "INFO" "Backing up application data..."
    
    local backup_file="$BACKUP_DIR/app_data_${DATE_FORMAT}.tar.gz"
    
    # Backup memory and logs directories
    if tar -czf "$backup_file" \
        -C /opt/mrdarkpromth \
        --exclude='*.tmp' \
        --exclude='*.log' \
        memory 2>/dev/null; then
        local size=$(du -h "$backup_file" | cut -f1)
        log_message "INFO" "Application data backup completed: $backup_file ($size)"
    else
        log_message "WARN" "Application data backup failed (non-critical)"
    fi
}

# Cleanup old backups
cleanup_old_backups() {
    log_message "INFO" "Cleaning up backups older than $RETENTION_DAYS days..."
    
    local deleted_count=$(find "$BACKUP_DIR" -name "*.gz" -mtime +$RETENTION_DAYS -delete -print | wc -l)
    local deleted_rdb=$(find "$BACKUP_DIR" -name "*.rdb" -mtime +$RETENTION_DAYS -delete -print | wc -l)
    
    log_message "INFO" "Deleted $((deleted_count + deleted_rdb)) old backup files"
}

# Show backup summary
show_summary() {
    log_message "INFO" "=== Backup Summary ==="
    log_message "INFO" "Backup directory: $BACKUP_DIR"
    
    local total_size=$(du -sh "$BACKUP_DIR" 2>/dev/null | cut -f1)
    local file_count=$(find "$BACKUP_DIR" -type f | wc -l)
    
    log_message "INFO" "Total backup size: $total_size ($file_count files)"
    log_message "INFO" "Retention period: $RETENTION_DAYS days"
}

# Test mode
test_mode() {
    log_message "INFO" "Running in test mode..."
    check_prerequisites
    log_message "INFO" "All prerequisites passed"
    show_summary
    log_message "INFO" "Test completed successfully"
    exit 0
}

# Main function
main() {
    log_message "INFO" "=== Starting MR.DarkPromth Backup ==="
    
    # Parse arguments
    if [[ "${1:-}" == "--test" ]]; then
        test_mode
    fi
    
    check_prerequisites
    backup_postgresql
    backup_redis
    backup_application_data
    cleanup_old_backups
    show_summary
    
    log_message "INFO" "=== Backup Completed Successfully ==="
}

# Run main
main "$@"
