#!/bin/bash

echo "🔧 Fixing SSL certificate issue..."

# Step 1: Stop Nginx to free port 80
echo "⏹️  Stopping Nginx container..."
docker-compose stop nginx

# Step 2: Wait a moment for port to be released
sleep 3

# Step 3: Get SSL certificate with standalone mode
echo "🔐 Requesting SSL certificate..."
certbot certonly --standalone -d bt-shop-dark.online -d www.bt-shop-dark.online --force-renewal

# Step 4: Copy certificates to certs folder
echo "📋 Copying certificates..."
cp /etc/letsencrypt/live/bt-shop-dark.online/fullchain.pem certs/cert.pem
cp /etc/letsencrypt/live/bt-shop-dark.online/privkey.pem certs/key.pem

# Step 5: Restart Nginx
echo "🔄 Restarting Nginx..."
docker-compose start nginx

echo "✅ SSL certificate setup complete!"
echo "🌐 Your site should now be accessible at https://bt-shop-dark.online"
