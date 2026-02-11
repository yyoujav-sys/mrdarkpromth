#!/bin/bash
set -e

# Configuration
DB_URL="postgres://test_user:test_password@localhost:5433/mr_darkpromth_test"
REDIS_URL="redis://localhost:6380"

echo "🚀 Starting Integration Test Environment..."

# Start containers
docker compose -f docker-compose.test.yml up -d

# Wait for DB to be ready
echo "⏳ Waiting for Database..."
until docker exec mr_darkpromth-test-db-1 pg_isready -U test_user -d mr_darkpromth_test > /dev/null 2>&1; do
  sleep 1
done
echo "✅ Database is ready!"

# Run migrations (assuming sqlx-cli is installed)
echo "📦 Running Migrations..."
export DATABASE_URL=$DB_URL
sqlx migrate run --source db/migrations

# Run Tests
echo "🧪 Running Integration Tests..."
export TEST_INTEGRATION=true
export REDIS_URL_TEST=$REDIS_URL

# Run specific integration tests associated with learning and self-correction
cargo test --package mr_darkpromth_services --test agent_integration -- --nocapture

# Cleanup
echo "🧹 Cleaning up..."
docker compose -f docker-compose.test.yml down

echo "✨ Integration Tests Completed Successfully!"
