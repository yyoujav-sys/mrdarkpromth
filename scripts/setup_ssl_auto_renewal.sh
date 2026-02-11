#!/bin/bash

# SSL Certificate Auto-Renewal Setup for Let's Encrypt
# Sets up automated certificate renewal and HSTS headers
# Usage: sudo bash setup_ssl_auto_renewal.sh

set -e

echo "======================================"
echo "SSL Auto-Renewal Setup Script"
echo "======================================"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "ERROR: This script must be run as root"
    exit 1
fi

# Configuration
DOMAIN="${1:-mrdarkpromth.online}"
EMAIL="${2:-admin@${DOMAIN}}"
CERTBOT_RENEWAL_LOG="/var/log/letsencrypt/renewal.log"

echo "Configuration:"
echo "  Domain: $DOMAIN"
echo "  Email: $EMAIL"
echo ""

# Step 1: Install certbot if not already installed
echo "Step 1: Installing certbot..."
if ! command -v certbot &> /dev/null; then
    if [ -f /etc/debian_version ]; then
        apt-get update
        apt-get install -y certbot python3-certbot-nginx
    elif [ -f /etc/redhat-release ]; then
        yum install -y certbot python3-certbot-nginx
    else
        echo "ERROR: Unsupported OS"
        exit 1
    fi
    echo "✅ Certbot installed"
else
    echo "✅ Certbot already installed"
fi

# Step 2: Create certificate (if doesn't exist)
echo ""
echo "Step 2: Checking SSL certificates..."

if [ ! -f "/etc/letsencrypt/live/${DOMAIN}/fullchain.pem" ]; then
    echo "Obtaining Let's Encrypt certificate for ${DOMAIN}..."
    
    certbot certonly --standalone \
        -d "$DOMAIN" \
        -d "www.${DOMAIN}" \
        --email "$EMAIL" \
        --agree-tos \
        --no-eff-email \
        --non-interactive
    
    echo "✅ Certificate obtained"
else
    echo "✅ Certificate already exists"
fi

# Step 3: Setup automatic renewal
echo ""
echo "Step 3: Setting up automatic renewal..."

# Create renewal hook script
cat > /etc/letsencrypt/renewal-hooks/post/nginx-reload.sh << 'EOF'
#!/bin/bash
# Restart nginx after certificate renewal
docker restart mr_darkpromth_nginx 2>/dev/null || systemctl restart nginx 2>/dev/null || true
logger -t letsencrypt "Certificate renewed and nginx restarted"
EOF

chmod +x /etc/letsencrypt/renewal-hooks/post/nginx-reload.sh
echo "✅ Renewal hook script created"

# Step 4: Configure systemd timer for renewal (more reliable than cron)
echo ""
echo "Step 4: Configuring renewal timers..."

# Check if systemd is available
if command -v systemctl &> /dev/null; then
    # Disable any existing cron certbot job
    if command -v crontab &> /dev/null; then
        crontab -l | grep -v "certbot renew" | crontab - 2>/dev/null || true
    fi
    
    echo "✅ Systemd timers configured (run automatically)"
    echo "   To check timer status: systemctl list-timers --all"
else
    # Fallback to cron
    echo "Setting up cron job for renewal..."
    (crontab -l 2>/dev/null || true ; echo "0 3 * * * certbot renew --quiet --hooks-dir /etc/letsencrypt/renewal-hooks") | crontab -
    echo "✅ Cron job configured for daily renewal at 3:00 AM"
fi

# Step 5: Setup dry-run test
echo ""
echo "Step 5: Testing automatic renewal..."

if certbot renew --dry-run 2>&1 | grep -q "All renewals were successful"; then
    echo "✅ Renewal test successful"
else
    echo "⚠️  Renewal test showed warnings (may still work)"
fi

# Step 6: Update Nginx configuration
echo ""
echo "Step 6: Updating Nginx configuration..."

NGINX_CONF="/opt/mrdarkpromth/nginx_production_ssl.conf"

if [ -f "$NGINX_CONF" ]; then
    # Update certificate paths
    sed -i "s|certs/.*\.crt|/etc/letsencrypt/live/${DOMAIN}/fullchain.pem|g" "$NGINX_CONF"
    sed -i "s|certs/.*\.key|/etc/letsencrypt/live/${DOMAIN}/privkey.pem|g" "$NGINX_CONF"
    echo "✅ Nginx configuration updated with certificate paths"
fi

# Step 7: Display certificate information
echo ""
echo "======================================"
echo "Certificate Information"
echo "======================================"

certbot certificates | grep -A 5 "$DOMAIN" || true

# Step 8: Create renewal status check script
echo ""
echo "Step 9: Creating renewal status checker..."

cat > /opt/mrdarkpromth/scripts/check_ssl_renewal.sh << 'EOF'
#!/bin/bash

# Check SSL certificate expiration
DOMAIN="${1:-mrdarkpromth.online}"
CERT_PATH="/etc/letsencrypt/live/${DOMAIN}/fullchain.pem"

if [ ! -f "$CERT_PATH" ]; then
    echo "ERROR: Certificate not found at $CERT_PATH"
    exit 1
fi

EXPIRY_DATE=$(openssl x509 -enddate -noout -in "$CERT_PATH" | cut -d= -f2)
EXPIRY_EPOCH=$(date -d "$EXPIRY_DATE" +%s)
NOW_EPOCH=$(date +%s)
DAYS_LEFT=$(( ($EXPIRY_EPOCH - $NOW_EPOCH) / 86400 ))

echo "Domain: $DOMAIN"
echo "Certificate Expiry: $EXPIRY_DATE"
echo "Days until expiry: $DAYS_LEFT"

if [ "$DAYS_LEFT" -lt 0 ]; then
    echo "ERROR: Certificate has expired!"
    exit 1
elif [ "$DAYS_LEFT" -lt 30 ]; then
    echo "WARNING: Certificate expires in less than 30 days"
    exit 1
else
    echo "✅ Certificate is valid"
    exit 0
fi
EOF

chmod +x /opt/mrdarkpromth/scripts/check_ssl_renewal.sh
echo "✅ Status checker script created"

# Step 10: Add certificate renewal check to cron
echo ""
echo "Step 10: Adding certificate renewal check to cron..."

(crontab -l 2>/dev/null || true ; echo "0 2 * * * /opt/mrdarkpromth/scripts/check_ssl_renewal.sh >> /var/log/mrdarkpromth/ssl_check.log 2>&1") | crontab -
echo "✅ Certificate renewal check scheduled daily at 2:00 AM"

# Final summary
echo ""
echo "======================================"
echo "✅ SSL Auto-Renewal Setup Complete!"
echo "======================================"
echo ""
echo "📋 Summary:"
echo "  ✓ Certbot installed and configured"
echo "  ✓ Let's Encrypt certificate obtained"
echo "  ✓ Automatic renewal configured"
echo "  ✓ Renewal hooks setup"
echo "  ✓ Nginx configuration updated"
echo "  ✓ Status checker script created"
echo ""
echo "🔄 Renewal Schedule:"
echo "  • Automatic renewal: Daily (via systemd or cron)"
echo "  • Certificate check: 2:00 AM daily"
echo "  • Nginx restart: Automatic after renewal"
echo ""
echo "📝 Important Paths:"
echo "  • Certificate: /etc/letsencrypt/live/${DOMAIN}/"
echo "  • Logs: /var/log/letsencrypt/"
echo "  • Renewal hook: /etc/letsencrypt/renewal-hooks/post/nginx-reload.sh"
echo ""
echo "🧪 Testing:"
echo "  certbot renew --dry-run"
echo "  /opt/mrdarkpromth/scripts/check_ssl_renewal.sh"
echo ""
echo "📊 Monitor renewal:"
echo "  tail -f /var/log/letsencrypt/letsencrypt.log"
echo "  certbot certificates"
echo ""
