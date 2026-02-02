#!/bin/bash
# 🔥 SSL Certificate Renewal with Webroot Mode
# ไม่ต้องหยุด Nginx ระหว่างขอใบรับรอง

echo "=== 🔥 SSL Certificate Renewal (Webroot Mode) ==="

# 1. สร้าง webroot directory สำหรับ certbot
mkdir -p /var/www/certbot
chown -R www-data:www-data /var/www/certbot

# 2. ขอใบรับรองด้วย webroot mode
echo "2. Requesting SSL certificate with webroot mode..."
certbot certonly \
    --webroot \
    -w /var/www/certbot \
    -d bt-shop-dark.online \
    -d www.bt-shop-dark.online \
    --email admin@bt-shop-dark.online \
    --agree-tos \
    --non-interactive \
    --force-renewal

# 3. ตรวจสอบว่าได้ใบรับรองหรือไม่
if [ -f "/etc/letsencrypt/live/bt-shop-dark.online/fullchain.pem" ]; then
    echo "✅ SSL certificate obtained successfully!"
    
    # 4. Reload Nginx (ไม่ต้อง restart)
    echo "4. Reloading Nginx..."
    nginx -t && systemctl reload nginx
    
    # 5. ตั้งค่า auto-renewal
    echo "5. Setting up auto-renewal..."
    (crontab -l 2>/dev/null; echo "0 12 * * * /usr/bin/certbot renew --quiet && systemctl reload nginx") | crontab -
    
    echo "✅ SSL setup complete!"
else
    echo "❌ Failed to obtain SSL certificate"
    exit 1
fi

# 6. แสดงข้อมูลใบรับรอง
echo ""
echo "=== 📋 Certificate Info ==="
certbot certificates
