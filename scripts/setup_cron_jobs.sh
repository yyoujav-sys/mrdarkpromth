#!/bin/bash

# Setup automated backup and maintenance cron jobs
# Run this script as root to install cron jobs
# Usage: sudo bash setup_cron_jobs.sh

set -e

echo "Setting up automated cron jobs for MR.DarkPromth..."

# Create log directory
mkdir -p /var/log/mrdarkpromth
chmod 755 /var/log/mrdarkpromth

# Make scripts executable
chmod +x /opt/mrdarkpromth/scripts/backup.sh
chmod +x /opt/mrdarkpromth/setup_dns_and_ssl.sh

# Create a temporary crontab file
CRON_FILE="/tmp/mrdarkpromth_cron_$$.txt"
cat > "$CRON_FILE" << 'EOF'
# MR.DarkPromth Automated Tasks
# ========================================

# Daily backup at 3:00 AM
0 3 * * * /opt/mrdarkpromth/scripts/backup.sh >> /var/log/mrdarkpromth/backup.log 2>&1

# Weekly backup verification on Sundays at 4:00 AM
0 4 * * 0 /opt/mrdarkpromth/scripts/verify_backup.sh >> /var/log/mrdarkpromth/backup_verify.log 2>&1

# SSL certificate renewal check - runs at 2:00 AM daily
0 2 * * * certbot renew --quiet && docker restart mr_darkpromth_nginx 2>> /var/log/mrdarkpromth/ssl_renew.log

# Log rotation - cleanup old logs daily at 5:00 AM
0 5 * * * find /var/log/mrdarkpromth -name "*.log" -mtime +30 -delete

# Database maintenance (vacuum/analyze) - weekly at 6:00 AM Sunday
0 6 * * 0 PGPASSWORD="${DB_PASSWORD}" vacuumdb -h postgres -U postgres -d mr_darkpromth --analyze >> /var/log/mrdarkpromth/db_maintenance.log 2>&1

# Health check - every 5 minutes
*/5 * * * * curl -f http://localhost:8080/health >/dev/null 2>&1 || echo "Health check failed at $(date)" >> /var/log/mrdarkpromth/health_check.log

EOF

# Check if crontab exists for current user (root)
if crontab -l >/dev/null 2>&1; then
    # Backup existing crontab
    crontab -l > /tmp/crontab_backup_$(date +%Y%m%d_%H%M%S).txt
    echo "✅ Existing crontab backed up"
fi

# Install new crontab
crontab "$CRON_FILE"
echo "✅ Cron jobs installed successfully"

# Display installed cron jobs
echo ""
echo "=== Installed Cron Jobs ==="
crontab -l | grep -v '^#' | grep -v '^$'
echo ""

# Clean up
rm -f "$CRON_FILE"

echo "✅ Setup completed successfully!"
echo ""
echo "📋 Cron jobs installed:"
echo "  • Daily backup at 3:00 AM"
echo "  • Weekly backup verification on Sundays at 4:00 AM"
echo "  • SSL certificate renewal check at 2:00 AM"
echo "  • Log rotation daily at 5:00 AM"
echo "  • Database maintenance Sundays at 6:00 AM"
echo "  • Health check every 5 minutes"
echo ""
echo "📝 Log files:"
echo "  • /var/log/mrdarkpromth/backup.log - Backup logs"
echo "  • /var/log/mrdarkpromth/backup_verify.log - Verification logs"
echo "  • /var/log/mrdarkpromth/ssl_renew.log - SSL renewal logs"
echo "  • /var/log/mrdarkpromth/db_maintenance.log - Database maintenance logs"
echo "  • /var/log/mrdarkpromth/health_check.log - Health check logs"
echo ""
echo "🔍 To view cron logs:"
echo "  tail -f /var/log/mrdarkpromth/*.log"
echo ""
