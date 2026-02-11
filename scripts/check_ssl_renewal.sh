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
