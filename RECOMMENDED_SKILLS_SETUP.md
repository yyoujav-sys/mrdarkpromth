# 🛠️ Recommended Skills & Setup Guide

**Goal**: Enhance the production system with automated monitoring, documentation, and backups  
**Timeline**: 2-3 hours total  
**Priority**: Medium (can be done after initial deployment)

---

## Skill 1: API Auto-Documentation (utoipa Swagger UI)

### Why This Matters
- Currently, API endpoints require manual testing with `curl`
- With Swagger UI, any team member can explore and test endpoints via web interface
- Auto-generated docs stay in sync with code (no manual updates)

### Setup Instructions

#### 1.1: Add utoipa dependency
```bash
cd /opt/mrdarkpromth/mr_darkpromth/api

# Add to Cargo.toml
cargo add utoipa --features "openapi_3_1"
cargo add utoipa-swagger-ui --features "actix-web"
```

#### 1.2: Add OpenAPI annotations to handlers
Example with chat endpoint:
```rust
// In api/src/handlers/chat.rs

use utoipa::OpenApi;

/// Send a chat message
#[utoipa::path(
    post,
    path = "/api/chat",
    responses(
        (status = 200, description = "Message sent", body = ChatResponse),
        (status = 400, description = "Invalid input"),
        (status = 401, description = "Unauthorized"),
    ),
    params(
        ("Authorization" = String, Header, description = "Bearer token"),
    ),
    request_body = ChatRequest,
)]
pub async fn send_chat_message(
    req: web::Json<ChatRequest>,
    // ... handler code
) -> Result<HttpResponse> {
    // ...
}
```

#### 1.3: Register Swagger UI endpoint
```rust
// In api/src/main.rs

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::chat::send_chat_message,
        handlers::auth::login,
        handlers::auth::register,
        // Add all endpoints here
    ),
    components(
        schemas(ChatRequest, ChatResponse, // ... all DTOs)
    ),
    info(
        title = "MR DarkPromth API",
        version = "1.0.0",
        description = "Full-stack AI chat platform",
    )
)]
pub struct ApiDoc;

// In configure routes:
let openapi = ApiDoc::openapi();
web::scope("/api-docs")
    .route("/openapi.json", web::get().to(|| HttpResponse::Ok().json(openapi)))
    .service(SwaggerUi::new("/swagger-ui")
        .url("/api-docs/openapi.json", openapi))
```

#### 1.4: Access Swagger UI
```bash
# After deployment
open http://localhost:3000/api-docs/swagger-ui/
# Or on production
open https://mrdarkpromth.online/api-docs/swagger-ui/
```

**Estimated Effort**: 1-2 hours  
**ROI**: High - Reduces onboarding time, enables self-service API testing

---

## Skill 2: Real-time Monitoring & Alerts

### Why This Matters
- Right now, you must manually check logs and system metrics
- Missing alerts = delayed response to critical issues
- Real-time alerts to Telegram/Discord = instant notification

### Current Monitoring Setup

Check what's already configured:
```bash
# Check Prometheus
curl http://localhost:9090/-/healthy

# Check Grafana
curl http://localhost:3001 -s | head -20

# Check AlertManager
curl http://localhost:9093/-/healthy
```

### 2.1: Configure Grafana Alerting (Basic)

```bash
# Step 1: Access Grafana
# URL: http://localhost:3001
# Default: admin/admin

# Step 2: Go to Alerts → Notification Channels
# Step 3: Add new channel type "Telegram"

# Configuration:
Name: "TelegramAlerts"
Bot Token: YOUR_TELEGRAM_BOT_TOKEN
Chat ID: YOUR_CHAT_ID

# Step 4: Create alert rules
# - High Memory Usage (> 85%)
# - High CPU (> 80%)
# - API Response Time (> 5s)
# - Database Connection Count (> 45/50)
```

### 2.2: Telegram Bot Setup

```bash
# 1. Create bot via @BotFather on Telegram
# 2. Get token: /start, /newbot, follow prompts
# 3. Get Chat ID:
# - Start chat with bot
# - Send message: /start
# - URL: https://api.telegram.org/botYOUR_TOKEN/getUpdates
# - Look for "chat": {"id": YOUR_CHAT_ID}

# 4. Store in .env:
echo "TELEGRAM_BOT_TOKEN=123456:ABCdefGHI" >> /opt/mrdarkpromth/.env
echo "TELEGRAM_CHAT_ID=-1001234567890" >> /opt/mrdarkpromth/.env
```

### 2.3: Alert Rules Configuration

Edit `/opt/mrdarkpromth/monitoring/prometheus-alerts.yml`:

```yaml
groups:
  - name: api_alerts
    interval: 30s
    rules:
      # High API error rate
      - alert: HighAPIErrorRate
        expr: rate(http_requests_total{status=~"5.."}[5m]) > 0.1
        for: 5m
        annotations:
          summary: "High API error rate: {{ $value }}"
          description: "API error rate is {{ $value }}/sec"

      # API down
      - alert: APIDown
        expr: up{job="api"} == 0
        for: 1m
        annotations:
          summary: "API is DOWN"
          description: "API has been unreachable for 1 minute"

      # Database connection pool exhaustion
      - alert: DBPoolExhausted
        expr: db_pool_connections_in_use{job="api"} > 45
        for: 2m
        annotations:
          summary: "Database pool nearly exhausted: {{ $value }}/50"

      # High memory usage
      - alert: HighMemoryUsage
        expr: (container_memory_usage_bytes{name="mr_darkpromth_api"} / 1073741824) > 0.85
        for: 5m
        annotations:
          summary: "API memory usage high: {{ humanize $value }}GB"
```

### 2.4: Reload Alertmanager

```bash
cd /opt/mrdarkpromth
docker-compose restart prometheus alertmanager

# Verify
curl http://localhost:9093/api/v1/status | jq .
```

**Estimated Effort**: 1 hour  
**ROI**: Very High - Prevents silent failures

---

## Skill 3: Database Migration Tool (sqlx-cli)

### Why This Matters
- Currently, migrations may be applied manually or inconsistently
- sqlx-cli ensures migrations are tracked and applied in order
- Safe rollback/redo capabilities

### Installation

```bash
cargo install sqlx-cli --no-default-features --features postgres

# Verify
sqlx --version
```

### Usage

```bash
# Check migration status
cd /opt/mrdarkpromth/mr_darkpromth
sqlx migrate info

# Add new migration (creates file like: 20260211_123456_add_column.sql)
sqlx migrate add -r "Add language column to users"
# Edit migrations/20260211_123456_add_language_column.sql

# Apply all pending migrations
sqlx migrate run

# Redo last migration (if --reversible)
sqlx migrate revert

# List all migrations
sqlx migrate list
```

### Example Migration (add new column)

File: `migrations/20260211_add_user_preferences.sql`

```sql
-- Add up
CREATE TABLE user_preferences (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    theme VARCHAR(20) DEFAULT 'dark',
    notifications_enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);

-- Add down (for reversible migrations)
DROP TABLE IF EXISTS user_preferences;
```

**Estimated Effort**: 30 minutes  
**ROI**: Medium - Improves schema management

---

## Skill 4: Automated Backups (Enhanced)

### Why This Matters
- Currently, backups may be manual or on single schedule
- Automated backups every 4-6 hours = minimal data loss risk
- Cloud storage = backup survives if server fails

### 4.1: Current Backup Script

Check what's already implemented:
```bash
cat /opt/mrdarkpromth/backup.sh
```

### 4.2: Enhance with Cloud Upload

Edit `/opt/mrdarkpromth/backup.sh`:

```bash
#!/bin/bash

# Enhanced backup script with S3 upload

BACKUP_DIR="/opt/mrdarkpromth/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/postgres_backup_$TIMESTAMP.sql"
S3_BUCKET="mrdarkpromth-backups"
S3_REGION="us-east-1"
AWS_PROFILE="default"

# Create backup
echo "Creating database backup..."
docker-compose exec -T postgres pg_dump \
  -U postgres mrdarkpromth \
  | gzip > "$BACKUP_FILE.gz"

echo "Backup size: $(du -h "$BACKUP_FILE.gz" | cut -f1)"

# Upload to S3 (if aws cli installed)
if command -v aws &> /dev/null; then
  echo "Uploading to S3..."
  aws s3 cp "$BACKUP_FILE.gz" \
    "s3://$S3_BUCKET/$TIMESTAMP/" \
    --region "$S3_REGION" \
    --profile "$AWS_PROFILE"
  
  echo "✓ Backup uploaded to S3://$S3_BUCKET/$TIMESTAMP/"
fi

# Keep only last 30 days of local backups
echo "Cleaning old backups..."
find "$BACKUP_DIR" -name "postgres_backup_*.sql.gz" -mtime +30 -delete

echo "✓ Backup complete"
```

### 4.3: Setup Cron Job (every 6 hours)

```bash
# Edit crontab
crontab -e

# Add line:
0 */6 * * * /opt/mrdarkpromth/backup.sh >> /var/log/mrdarkpromth_backup.log 2>&1

# Verify
crontab -l
```

### 4.4: S3 Setup (Optional but Recommended)

```bash
# Install AWS CLI
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip && sudo ./aws/install

# Configure credentials
aws configure  # Enter AWS Access Key, Secret Key, region

# Create S3 bucket
aws s3 mb s3://mrdarkpromth-backups --region us-east-1

# Enable versioning (for rollback)
aws s3api put-bucket-versioning \
  --bucket mrdarkpromth-backups \
  --versioning-configuration Status=Enabled

# Set lifecycle policy (delete after 90 days)
cat > /tmp/lifecycle.json << 'EOF'
{
  "Rules": [
    {
      "Id": "DeleteOldBackups",
      "Status": "Enabled",
      "Expiration": {"Days": 90}
    }
  ]
}
EOF

aws s3api put-bucket-lifecycle-configuration \
  --bucket mrdarkpromth-backups \
  --lifecycle-configuration file:///tmp/lifecycle.json

# Test backup
/opt/mrdarkpromth/backup.sh

# Verify in S3
aws s3 ls s3://mrdarkpromth-backups/
```

**Estimated Effort**: 1 hour  
**ROI**: Very High - Protects against data loss

---

## Implementation Priority Roadmap

### Day 1 (After deployment)
1. ✅ Skill 2: Real-time Monitoring & Alerts (1 hour)
   - Set up Telegram notifications
   - Configure high-priority alerts

### Day 2-3
2. ⏳ Skill 4: Automated Backups (1 hour)
   - Set up S3 uploads
   - Configure cron jobs

### Week 1-2
3. ⏳ Skill 1: API Documentation (1-2 hours)
   - Add utoipa annotations
   - Deploy Swagger UI

### Week 2-4
4. ⏳ Skill 3: Database Migrations (30 min)
   - Set up sqlx-cli workflow
   - Document migration process

---

## Quick Reference: All Commands

```bash
# Monitoring
curl http://localhost:9090/-/healthy        # Prometheus
curl http://localhost:3001                  # Grafana
curl http://localhost:9093/-/healthy        # AlertManager

# Backups
./backup.sh                                 # Run manual backup
aws s3 ls s3://mrdarkpromth-backups/       # List S3 backups

# Migrations
sqlx migrate list                           # List all migrations
sqlx migrate add "description"              # Create new migration
sqlx migrate run                            # Apply pending

# API Docs
open http://localhost:3000/api-docs/swagger-ui/  # Swagger UI
```

---

## ROI Summary

| Skill | Setup Time | Long-term Benefit | Priority |
|-------|-----------|-------------------|----------|
| Monitoring | 1h | Instant alerts = 10x faster incident response | 🔴 High |
| Backups | 1h | Disaster recovery = avoid catastrophic loss | 🔴 High |
| Migrations | 0.5h | Safer deployments = less downtime | 🟡 Medium |
| API Docs | 2h | Self-service for team = less support | 🟡 Medium |

All four combined take ~4.5 hours but provide massive value for zero runtime cost.
