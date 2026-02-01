#!/bin/bash

echo "🔧 Fixing SSL certificate using webroot method..."

# Step 1: Create webroot directory for Let's Encrypt challenges
mkdir -p /var/www/certbot

# Step 2: Update Nginx config to handle ACME challenges
echo "📝 Updating Nginx configuration for ACME challenges..."
cat > nginx_letsencrypt.conf << 'EOF'
server {
    listen 80;
    server_name bt-shop-dark.online www.bt-shop-dark.online;

    # Let's Encrypt ACME challenges
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
        try_files $uri =404;
    }

    # Redirect all other HTTP traffic to HTTPS
    location / {
        return 301 https://$host$request_uri;
    }
}
EOF

# Step 3: Temporarily replace Nginx config
cp nginx/nginx.conf nginx/nginx.conf.backup
cp nginx_letsencrypt.conf nginx/nginx.conf

# Step 4: Restart Nginx with new config
echo "🔄 Restarting Nginx with ACME challenge config..."
docker-compose restart nginx

# Step 5: Wait for Nginx to be ready
sleep 5

# Step 6: Get SSL certificate using webroot
echo "🔐 Requesting SSL certificate with webroot method..."
certbot certonly --webroot -w /var/www/certbot -d bt-shop-dark.online -d www.bt-shop-dark.online --force-renewal

# Step 7: Copy certificates
echo "📋 Copying certificates..."
cp /etc/letsencrypt/live/bt-shop-dark.online/fullchain.pem certs/cert.pem
cp /etc/letsencrypt/live/bt-shop-dark.online/privkey.pem certs/key.pem

# Step 8: Restore original Nginx config
echo "🔄 Restoring original Nginx configuration..."
mv nginx/nginx.conf.backup nginx/nginx.conf

# Step 9: Restart Nginx with SSL
docker-compose restart nginx

echo "✅ SSL certificate setup complete!"
echo "🌐 Your site should now be accessible at https://bt-shop-dark.online"
