# MR.DarkPromth - Ultra Tier AI Platform

## 🚀 Quick Start

### Prerequisites
- Docker & Docker Compose
- Rust (for local development)

### Development Setup

1. **Clone and Setup**:
```bash
git clone <repository>
cd MR.Darkpromth
cp .env.example .env  # Edit with your values
```

2. **Start Services**:
```bash
docker-compose up -d
```

3. **Verify Setup**:
```bash
curl http://localhost:8080/health
```

## 🎯 Ultra Tier System

### Safe Implementation (Current)
- ✅ Consent gating with 4 mandatory checkboxes
- ✅ Audit logging for all Ultra Tier activities
- ✅ Redis event bus for agent coordination
- ✅ Database persistence for consents & activations

### API Endpoints

#### POST /api/ultra/consent
```json
{
  "user_id": "uuid",
  "accepted_unrestricted": true,
  "accepted_responsibility": true,
  "accepted_legal_compliance": true,
  "accepted_risks": true
}
```

#### POST /api/ultra/activate
```json
{
  "user_id": "uuid"
}
```

#### GET /api/ultra/status/{user_id}
Returns consent and activation status.

## 🏗️ Architecture

- **Backend**: Rust with Axum framework
- **Database**: PostgreSQL with SQLx
- **Event Bus**: Redis Streams
- **Containerization**: Docker & Docker Compose
- **10 Agent System**: Parallel execution with memory logs

## 📁 Project Structure

```
MR.Darkpromth/
├── src/                    # Rust source code
├── migrations/             # Database migrations
├── memory/                 # Agent memory logs
├── Agent_*_Workflow.md     # Individual agent workflows
├── docker-compose.yml      # Service orchestration
├── Dockerfile             # Container definition
└── .env.example           # Environment template
```

## 🔧 Development

### Local Development
```bash
# Install dependencies
cargo build

# Run with hot reload
cargo run

# Run tests
cargo test
```

### Database Migrations
```bash
# Apply migrations
docker-compose exec postgres psql -U postgres -d mr_darkpromth -f /docker-entrypoint-initdb.d/001_create_ultra_tier_tables.sql
```

## 🚨 Security Notes

- Ultra Tier requires explicit consent for all 4 conditions
- All activities are logged for audit purposes
- System protection prevents infrastructure damage
- Sandbox execution for generated code

## 📝 Agent Coordination

Agents communicate via Redis Streams:
- `ultra_tier_events` - Ultra Tier related events
- Individual agent memory logs in `/memory/`

## 🎯 Next Steps

1. Start Agents 1-4 (Foundation Layer)
2. Implement remaining agent workflows
3. Add frontend components
4. Deploy to production
