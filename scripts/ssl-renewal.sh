#!/bin/bash
# SSL Certificate Auto-Renewal Script
# Enhanced version with logging, pre-checks, and monitoring

set -euo pipefail

LOG_FILE="/var/log/ssl-renewal.log"
DOMAIN="mrdarkpromth.online"
CERT_PATH="/etc/letsencrypt/live/${DOMAIN}"
WEBROOT="/var/www/certbot"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_message() {
    local level=$1
    local message=$2
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "[$timestamp] [$level] $message" >> "$LOG_FILE"
    echo -e "[$timestamp] [$level] $message" >&2
}

# Check certificate expiry
check_cert_expiry() {
    if [ ! -f "${CERT_PATH}/fullchain.pem" ]; then
        log_message "WARN" "Certificate not found at ${CERT_PATH}"
        return 1
    fi
    
    local expiry_date=$(openssl x509 -enddate -noout -in "${CERT_PATH}/fullchain.pem" | cut -d= -f2)
    local expiry_epoch=$(date -d "$expiry_date" +%s 2>/dev/null || date -j -f "%b %d %H:%M:%S %Y %Z" "$expiry_date" +%s)
    local current_epoch=$(date +%s)
    local days_remaining=$(( (expiry_epoch - current_epoch) / 86400 ))
    
    log_message "INFO" "Certificate expires: $expiry_date"
    log_message "INFO" "Days remaining: $days_remaining"
    
    echo "$days_remaining"
}

# Renew certificate
renew_certificate() {
    log_message "INFO" "Starting certificate renewal..."
    
    # Ensure webroot exists
    mkdir -p "$WEBROOT"
    
    # Try renewal with certbot
    if certbot renew --webroot -w "$WEBROOT" --quiet --deploy-hook "docker exec mr_darkpromth_nginx nginx -s reload 2>/dev/null || nginx -s reload 2>/dev/null || true"; then
        log_message "INFO" "Certificate renewal successful"
        return 0
    else
        log_message "ERROR" "Certificate renewal failed"
        return 1
    fi
}

# Force renewal (for manual execution)
force_renewal() {
    log_message "INFO" "Force renewing certificate..."
    
    mkdir -p "$WEBROOT"
    
    if certbot certonly \
        --webroot \
        -w "$WEBROOT" \
        -d "$DOMAIN" \
        -d "www.${DOMAIN}" \
        -d "api.${DOMAIN}" \
        --email "admin@${DOMAIN}" \
        --agree-tos \
        --non-interactive \
        --force-renewal; then
        
        log_message "INFO" "Force renewal successful"
        
        # Reload nginx
        if docker exec mr_darkpromth_nginx nginx -s reload 2>/dev/null; then
            log_message "INFO" "Nginx reloaded in container"
        elif systemctl reload nginx 2>/dev/null; then
            log_message "INFO" "Nginx service reloaded"
        else
            log_message "WARN" "Could not reload nginx automatically"
        fi
        
        return 0
    else
        log_message "ERROR" "Force renewal failed"
        return 1
    fi
}

# Show certificate status
show_status() {
    log_message "INFO" "=== SSL Certificate Status ==="
    
    if [ -f "${CERT_PATH}/fullchain.pem" ]; then
        local days=$(check_cert_expiry)
        
        if [ "$days" -lt 7 ]; then
            log_message "ERROR" "Certificate expires in $days days - CRITICAL!"
        elif [ "$days" -lt 30 ]; then
            log_message "WARN" "Certificate expires in $days days - renewal recommended"
        else
            log_message "INFO" "Certificate is valid for $days more days"
        fi
    else
        log_message "ERROR" "No certificate found"
    fi
    
    # Show all certificates
    certbot certificates 2>/dev/null || true
}

# Main
main() {
    log_message "INFO" "=== SSL Auto-Renewal Script Started ==="
    
    case "${1:-check}" in
        renew)
            renew_certificate
            ;;
        force)
            force_renewal
            ;;
        status)
            show_status
            ;;
        check)
            local days=$(check_cert_expiry)
            if [ "$days" -lt 30 ]; then
                log_message "INFO" "Certificate expires in $days days, renewing..."
                renew_certificate
            else
                log_message "INFO" "Certificate valid for $days days, no renewal needed"
            fi
            ;;
        *)
            echo "Usage: $0 {check|renew|force|status}"
            exit 1
            ;;
    esac
    
    log_message "INFO" "=== SSL Auto-Renewal Script Completed ==="
}

main "$@"
