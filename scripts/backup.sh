#!/bin/bash

# MR.DarkPromth Automated Backup Script
# Backs up PostgreSQL database and application logs

BACKUP_DIR="/home/ubuntu/MR.Darkpromth/backups"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
DB_NAME="mr_darkpromth"
DB_USER="postgres"

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

echo "Starting backup at $TIMESTAMP..."

# 1. Backup PostgreSQL Database
echo "Backing up database $DB_NAME..."
docker exec mrdarkpromth-postgres pg_dump -U $DB_USER $DB_NAME > "$BACKUP_DIR/db_backup_$TIMESTAMP.sql"

# 2. Backup Application Logs
echo "Backing up application logs..."
tar -czf "$BACKUP_DIR/logs_backup_$TIMESTAMP.tar.gz" /home/ubuntu/MR.Darkpromth/logs

# 3. Compress Database Backup
gzip "$BACKUP_DIR/db_backup_$TIMESTAMP.sql"

# 4. Cleanup old backups (keep last 7 days)
echo "Cleaning up old backups..."
find "$BACKUP_DIR" -name "*.gz" -mtime +7 -delete

echo "Backup completed successfully!"
ls -lh "$BACKUP_DIR" | grep "$TIMESTAMP"
