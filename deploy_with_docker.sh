#!/bin/bash
# 🔥 Script สำหรับ Build และ Deploy ด้วย Docker ทั้งหมด

echo "=== 🔥 Deploy MR.DarkPromth with Docker ==="

# 1. Stop old containers
echo "1. Stopping old containers..."
docker stop mr_darkpromth_api mr_darkpromth_frontend 2>/dev/null
docker rm mr_darkpromth_api mr_darkpromth_frontend 2>/dev/null

# 2. Build Rust API using Docker
echo "2. Building Rust API with Docker..."
cd /opt/mrdarkpromth

# Create Dockerfile for Rust API
cat > Dockerfile.api << 'EOF'
FROM rust:1.75-slim as builder
WORKDIR /app
COPY mr_darkpromth/ ./mr_darkpromth/
WORKDIR /app/mr_darkpromth/api
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/mr_darkpromth/api/target/release/mr_darkpromth_api /app/
EXPOSE 8080
CMD ["./mr_darkpromth_api"]
EOF

# Build the API image
docker build -t mr-darkpromth-api-rust:latest -f Dockerfile.api . 2>&1 | tail -10

# 3. Build React Frontend using Docker
echo "3. Building React Frontend with Docker..."

# Create Dockerfile for Frontend
cat > Dockerfile.frontend << 'EOF'
FROM node:20-slim as builder
WORKDIR /app
COPY frontend/package*.json ./
RUN npm install
COPY frontend/ ./
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx/frontend.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
EOF

# Build the Frontend image
docker build -t mr-darkpromth-frontend-react:latest -f Dockerfile.frontend . 2>&1 | tail -10

# 4. Run new containers
echo "4. Starting new containers..."
docker run -d --name mr_darkpromth_api_rust \
  --network mr_darkpromth_network \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://postgres:postgres@mr_darkpromth_postgres:5432/mr_darkpromth \
  -e REDIS_URL=redis://mr_darkpromth_redis:6379 \
  -e RUST_LOG=info \
  --restart unless-stopped \
  mr-darkpromth-api-rust:latest 2>&1 || echo "API start failed"

docker run -d --name mr_darkpromth_frontend_react \
  --network mr_darkpromth_network \
  -p 3000:80 \
  --restart unless-stopped \
  mr-darkpromth-frontend-react:latest 2>&1 || echo "Frontend start failed"

# 5. Health check
echo "5. Health check..."
sleep 15
echo "API Health:"
curl -s http://localhost:8080/health || echo "API not responding"
echo ""
echo "API Info:"
curl -s http://localhost:8080/api/info || echo "API info not available"

echo ""
echo "=== ✅ Deployment Complete ==="
