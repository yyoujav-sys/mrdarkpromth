#!/bin/bash

# ==========================================
# MR.DarkPromth Production Deployment
# ==========================================

set -e

echo "🚀 Deploying MR.DarkPromth to Production..."
echo "=========================================="

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    echo "❌ Please don't run as root"
    exit 1
fi

# Check Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running"
    exit 1
fi

# Check .env file exists
if [ ! -f ./.env ]; then
    echo "❌ .env file not found"
    exit 1
fi

# Check required tools
command -v jq >/dev/null 2>&1 || { echo "❌ jq is required but not installed"; exit 1; }
command -v openssl >/dev/null 2>&1 || { echo "❌ openssl is required but not installed"; exit 1; }

echo "1. 🔧 Setting up Production Environment..."

# Create necessary directories
mkdir -p logs/nginx
mkdir -p backups
mkdir -p monitoring/grafana/dashboards
mkdir -p monitoring/grafana/datasources

# Generate SSL certificates if they don't exist
if [ ! -f ./certs/cert.pem ]; then
    echo "   📋 Generating SSL certificates..."
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout ./certs/key.pem \
        -out ./certs/cert.pem \
        -subj "/C=US/ST=State/L=City/O=Organization/CN=localhost"
fi

# Create monitoring configuration
if [ ! -f ./monitoring/prometheus.yml ]; then
    echo "   📋 Creating Prometheus config..."
    cat > ./monitoring/prometheus.yml << EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

rule_files:
  # - "first_rules.yml"
  # - "second_rules.yml"

scrape_configs:
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'mr-darkpromth-api'
    static_configs:
      - targets: ['api:8080']
    metrics_path: '/metrics'
    scrape_interval: 5s

  - job_name: 'nginx'
    static_configs:
      - targets: ['nginx:9113']
    scrape_interval: 5s
EOF
fi

echo "2. 🐳 Building Production Images..."

# Build images
docker-compose -f docker-compose.prod.yml build --no-cache

echo "3. 🚀 Starting Production Services..."

# Stop any existing services
docker-compose -f docker-compose.prod.yml down

# Start production services
docker-compose -f docker-compose.prod.yml up -d

echo "4. ⏳ Waiting for services to be ready..."

# Wait for database
echo "   Waiting for PostgreSQL..."
timeout 60 bash -c 'until docker exec mr_darkpromth_postgres_prod pg_isready -U postgres; do sleep 1; done'

# Wait for Redis
echo "   Waiting for Redis..."
timeout 60 bash -c 'until docker exec mr_darkpromth_redis_prod redis-cli ping; do sleep 1; done'

# Wait for API
echo "   Waiting for API..."
timeout 60 bash -c 'until curl -k -f https://localhost/health; do sleep 2; done'

echo "5. 🧪 Running Production Tests..."

# Make test script executable
chmod +x ./production_test.sh

# Run tests
./production_test.sh

echo "6. 📊 Setting up Monitoring..."

# Wait for Grafana to start
sleep 10

# Create Grafana datasource (if API is available)
curl -k -s -X POST "http://localhost:3001/api/datasources" \
    -H "Content-Type: application/json" \
    -u admin:admin \
    -d '{
        "name": "Prometheus",
        "type": "prometheus",
        "url": "http://prometheus:9090",
        "access": "proxy",
        "isDefault": true
    }' > /dev/null 2>&1 || echo "   Grafana datasource setup skipped"

echo "7. 🔄 Setting up Backup Script..."

# Create backup script
cat > ./backup.sh << 'EOF'
#!/bin/bash
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="./backups"

# Database backup
docker exec mr_darkpromth_postgres_prod pg_dump -U postgres mr_darkpromth > "$BACKUP_DIR/db_backup_$DATE.sql"

# Config backup
tar -czf "$BACKUP_DIR/config_backup_$DATE.tar.gz" .env nginx/ monitoring/

# Clean old backups (keep 7 days)
find $BACKUP_DIR -name "*.sql" -mtime +7 -delete
find $BACKUP_DIR -name "*.tar.gz" -mtime +7 -delete

echo "Backup completed: $DATE"
EOF

chmod +x ./backup.sh

# Add to crontab (if not already there)
(crontab -l 2>/dev/null | grep -q "./backup.sh") || (crontab -l 2>/dev/null; echo "0 2 * * * cd $(pwd) && ./backup.sh") | crontab -

echo "8. 📈 Creating Health Check Script..."

# Create health check script
cat > ./health_check.sh << 'EOF'
#!/bin/bash

echo "🏥 MR.DarkPromth Health Check"
echo "=========================="

# Check services
services=("mr_darkpromth_postgres_prod" "mr_darkpromth_redis_prod" "mr_darkpromth_api_prod" "mr_darkpromth_nginx_prod" "mr_darkpromth_grafana_prod" "mr_darkpromth_prometheus_prod")

for service in "${services[@]}"; do
    if docker ps --format "table {{.Names}}" | grep -q "$service"; then
        echo "✅ $service is running"
    else
        echo "❌ $service is not running"
    fi
done

# Check endpoints
echo ""
echo "📡 Endpoint Health:"
if curl -k -f -s https://localhost/health > /dev/null; then
    echo "✅ Health endpoint"
else
    echo "❌ Health endpoint"
fi

if curl -k -f -s https://localhost/api/auth/me > /dev/null; then
    echo "✅ API endpoint"
else
    echo "❌ API endpoint"
fi

# Check disk space
echo ""
echo "💾 Disk Usage:"
df -h | grep -E "(Filesystem|/dev/)"

# Check memory
echo ""
echo "🧠 Memory Usage:"
free -h

echo ""
echo "📊 Docker Stats:"
docker stats --no-stream --format "table {{.Container}}\t{{.MemUsage}}\t{{.CPUPerc}}" | grep mr_darkpromth
EOF

chmod +x ./health_check.sh

echo "9. 🔒 Security Check..."

# Check for exposed ports
echo "   Checking exposed ports..."
docker ps --format "table {{.Names}}\t{{.Ports}}" | grep mr_darkpromth

# Check environment variables
echo "   Checking for sensitive data in .env..."
if grep -E "(PASSWORD|SECRET|KEY)" .env | grep -v "example\|test\|localhost"; then
    echo "   ⚠️  Sensitive data found - ensure these are production values"
fi

echo "10. 📝 Final Setup..."

# Create production documentation
cat > ./PRODUCTION_README.md << 'EOF'
# MR.DarkPromth Production Deployment

## Services Status
- API: https://localhost/api
- Frontend: https://localhost
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3001

## Default Credentials
- Grafana: admin/admin

## Important Files
- `.env`: Environment configuration
- `docker-compose.prod.yml`: Production services
- `nginx/nginx.conf`: Nginx configuration
- `certs/`: SSL certificates
- `backups/`: Database backups

## Maintenance Commands
```bash
# Health check
./health_check.sh

# Backup
./backup.sh

# View logs
docker logs mr_darkpromth_api_prod

# Restart services
docker-compose -f docker-compose.prod.yml restart

# Update services
git pull
docker-compose -f docker-compose.prod.yml build --no-cache
docker-compose -f docker-compose.prod.yml up -d
```

## Monitoring
- Prometheus metrics: http://localhost:9090
- Grafana dashboards: http://localhost:3001

## Security Notes
- Change default passwords
- Update SSL certificates for production domain
- Configure firewall rules
- Set up proper CORS origins
EOF

echo ""
echo "=========================================="
echo "🎉 PRODUCTION DEPLOYMENT COMPLETE!"
echo "=========================================="
echo ""
echo "🌐 Access URLs:"
echo "  - Frontend: https://localhost"
echo "  - API: https://localhost/api"
echo "  - Prometheus: http://localhost:9090"
echo "  - Grafana: http://localhost:3001"
echo ""
echo "📚 Documentation: ./PRODUCTION_README.md"
echo "🏥 Health Check: ./health_check.sh"
echo "💾 Backup: ./backup.sh"
echo ""
echo "🔐 IMPORTANT SECURITY REMINDERS:"
echo "  1. Change JWT_SECRET in .env"
echo "  2. Update CORS_ALLOWED_ORIGINS to your domain"
echo "  3. Configure real SSL certificates"
echo "  4. Set up production email service"
echo "  5. Configure firewall rules"
echo "  6. Set up monitoring alerts"
echo ""
echo "✅ System is PRODUCTION READY!"
