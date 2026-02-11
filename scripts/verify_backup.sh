#!/bin/bash

# Backup verification script - checks integrity and recoverability
# Run weekly to ensure backups are viable
# Place in /opt/mrdarkpromth/scripts/verify_backup.sh

set -e

# Configuration
BACKUP_DIR="/opt/mrdarkpromth/backups"
LOG_DIR="/var/log/mrdarkpromth"
LOG_FILE="${LOG_DIR}/backup_verify.log"

# Database configuration
DB_HOST="${DB_HOST:-postgres}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-postgres}"
DB_NAME="${DB_NAME:-mr_darkpromth}"
DB_PASSWORD="${DB_PASSWORD}"
TEST_DB_NAME="mr_darkpromth_test_restore"

# Ensure log directory exists
mkdir -p "$LOG_DIR"

# Function to log messages
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Function to verify backup file integrity
verify_backup_file() {
    local backup_file=$1
    
    log_message "Verifying backup file: $backup_file"
    
    if [ ! -f "$backup_file" ]; then
        log_message "ERROR: Backup file not found: $backup_file"
        return 1
    fi
    
    # Check file size (must be > 1KB)
    local size=$(stat -f%z "$backup_file" 2>/dev/null || stat -c%s "$backup_file" 2>/dev/null || echo 0)
    if [ "$size" -lt 1024 ]; then
        log_message "ERROR: Backup file too small ($size bytes): $backup_file"
        return 1
    fi
    
    # Verify gzip integrity
    if ! gzip -t "$backup_file" 2>/dev/null; then
        log_message "ERROR: Backup file is corrupted (gzip test failed): $backup_file"
        return 1
    fi
    
    # Check for SQL content
    if ! gunzip -c "$backup_file" | head -1000 | grep -q "\\(CREATE\\|INSERT\\|SELECT\\|--\\)"; then
        log_message "ERROR: Backup doesn't contain valid SQL content: $backup_file"
        return 1
    fi
    
    log_message "✅ Backup file integrity verified: $backup_file"
    return 0
}

# Function to test restore (optional - can be expensive)
test_restore() {
    local backup_file=$1
    
    log_message "Testing backup restoration..."
    
    # Create test database
    if PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -tc "SELECT 1 FROM pg_database WHERE datname = '$TEST_DB_NAME'" | grep -q 1; then
        log_message "Dropping existing test database..."
        PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -c "DROP DATABASE $TEST_DB_NAME;"
    fi
    
    log_message "Creating test database..."
    if ! PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -c "CREATE DATABASE $TEST_DB_NAME;"; then
        log_message "ERROR: Failed to create test database"
        return 1
    fi
    
    # Try to restore backup
    log_message "Attempting to restore backup to test database..."
    if gunzip -c "$backup_file" | PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$TEST_DB_NAME" > /dev/null 2>&1; then
        log_message "✅ Backup restoration successful"
        
        # Basic sanity checks on restored data
        local table_count=$(PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -d "$TEST_DB_NAME" -tc "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='public';" | tr -d ' ')
        log_message "✅ Restored database has $table_count tables"
        
        # Cleanup test database
        PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -c "DROP DATABASE $TEST_DB_NAME;"
        log_message "✅ Test database cleaned up"
        
        return 0
    else
        log_message "ERROR: Failed to restore backup to test database"
        # Cleanup test database
        PGPASSWORD="$DB_PASSWORD" psql -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -c "DROP DATABASE $TEST_DB_NAME;" 2>/dev/null || true
        return 1
    fi
}

# Function to verify backup age
verify_backup_age() {
    local backup_file=$1
    local max_age_hours=26  # Allow up to 26 hours (for daily backups)
    
    local file_age_seconds=$(($(date +%s) - $(stat -f%m "$backup_file" 2>/dev/null || stat -c%Y "$backup_file" 2>/dev/null)))
    local file_age_hours=$((file_age_seconds / 3600))
    
    if [ "$file_age_hours" -gt "$max_age_hours" ]; then
        log_message "WARNING: Backup is $file_age_hours hours old (max allowed: $max_age_hours)"
        return 1
    fi
    
    log_message "✅ Backup age is acceptable ($file_age_hours hours)"
    return 0
}

# Main execution
main() {
    log_message "======================================"
    log_message "Starting backup verification process"
    log_message "======================================"
    
    # Find the most recent backup
    local latest_backup=$(find "$BACKUP_DIR" -name "postgres_backup_*.sql.gz" -type f -printf '%T@ %p\n' | sort -rn | head -1 | cut -d' ' -f2-)
    
    if [ -z "$latest_backup" ]; then
        log_message "ERROR: No backups found in $BACKUP_DIR"
        exit 1
    fi
    
    log_message "Latest backup found: $latest_backup"
    
    # Verify backup file integrity
    if ! verify_backup_file "$latest_backup"; then
        exit 1
    fi
    
    # Verify backup age
    if ! verify_backup_age "$latest_backup"; then
        log_message "WARNING: Backup age check failed"
    fi
    
    # Test restore (uncomment to enable - requires extra time and disk space)
    # if ! test_restore "$latest_backup"; then
    #     exit 1
    # fi
    
    log_message "======================================"
    log_message "✅ Backup verification completed successfully"
    log_message "======================================"
}

# Execute main function
main
exit $?
