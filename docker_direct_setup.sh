#!/bin/bash
# 🔥 Docker Management Script - Direct Commands

echo "=== 🔥 MR.DarkPromth Docker Setup ==="

# 1. Create network
echo "1. Creating network..."
docker network create mr_darkpromth_network 2>/dev/null || echo "Network exists"

# 2. Run PostgreSQL
echo "2. Starting PostgreSQL..."
docker run -d --name mr_darkpromth_postgres \
  --network mr_darkpromth_network \
  -e POSTGRES_DB=mr_darkpromth \
  -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD=postgres \
  -v postgres_data:/var/lib/postgresql/data \
  postgres:15-alpine

# 3. Run Redis
echo "3. Starting Redis..."
docker run -d --name mr_darkpromth_redis \
  --network mr_darkpromth_network \
  redis:7-alpine

# 4. Pull and run API (from Docker Hub)
echo "4. Starting API..."
docker run -d --name mr_darkpromth_api \
  --network mr_darkpromth_network \
  -p 8080:8080 \
  -e DATABASE_URL=postgres://postgres:postgres@mr_darkpromth_postgres:5432/mr_darkpromth \
  -e REDIS_URL=redis://mr_darkpromth_redis:6379 \
  mr-darkpromth/backend:latest

echo "✅ Docker setup complete!"
