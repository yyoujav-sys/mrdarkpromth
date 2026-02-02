#!/bin/bash
# 🔥 Script สำหรับ Build และ Deploy Rust API + React Frontend

echo "=== 🔥 Deploy MR.DarkPromth Full System ==="

# 1. Stop old containers
echo "1. Stopping old containers..."
docker stop mr_darkpromth_api mr_darkpromth_frontend 2>/dev/null
docker rm mr_darkpromth_api mr_darkpromth_frontend 2>/dev/null

# 2. Build Rust API
echo "2. Building Rust API..."
cd /opt/mrdarkpromth/mr_darkpromth/api
cargo build --release 2>&1 | tail -5

# 3. Build Docker image for Rust API
echo "3. Building Docker image for Rust API..."
docker build -t mr-darkpromth-api-rust:latest -f Dockerfile.rust . 2>&1 | tail -5

# 4. Build React Frontend
echo "4. Building React Frontend..."
cd /opt/mrdarkpromth/frontend
npm install 2>&1 | tail -3
npm run build 2>&1 | tail -5

# 5. Build Docker image for Frontend
echo "5. Building Docker image for Frontend..."
docker build -t mr-darkpromth-frontend-react:latest . 2>&1 | tail -5

# 6. Run new containers
echo "6. Starting new containers..."
docker run -d --name mr_darkpromth_api_rust \
  --network mr_darkpromth_network \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://postgres:postgres@mr_darkpromth_postgres:5432/mr_darkpromth \
  -e REDIS_URL=redis://mr_darkpromth_redis:6379 \
  -e RUST_LOG=info \
  --restart unless-stopped \
  mr-darkpromth-api-rust:latest

docker run -d --name mr_darkpromth_frontend_react \
  --network mr_darkpromth_network \
  -p 3000:80 \
  --restart unless-stopped \
  mr-darkpromth-frontend-react:latest

# 7. Health check
echo "7. Health check..."
sleep 10
curl -s http://localhost:8080/health
curl -s http://localhost:8080/api/info

echo ""
echo "=== ✅ Deployment Complete ==="
