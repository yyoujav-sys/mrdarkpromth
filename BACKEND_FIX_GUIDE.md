# Backend Database Connection & Port Fix

## Issues Fixed:
1. **Port Conflict**: PostgreSQL and Redis were both using process ID 25312
2. **Hardcoded Port**: Backend was hardcoded to port 8080, conflicting with Docker mapping
3. **Database Connection**: .env was using localhost instead of Docker service names

## Solutions Applied:

### 1. Fixed Backend Port Configuration
- Updated `src/main.rs` to use environment variables `SERVER_HOST` and `SERVER_PORT`
- Backend now reads from environment instead of hardcoded values

### 2. Environment Configuration Files
- **`.env`**: Configured for Docker usage with service names (`postgres:5432`, `redis:6379`)
- **`.env.local`**: Configured for local development with localhost (`localhost:5432`, `localhost:6379`)

### 3. Port Conflicts Resolved
- Killed conflicting process (PID 25312)
- Port 8080 is now available for backend

## Running the Backend:

### Option 1: Local Development
```bash
# Use local environment file
copy .env.local .env

# Install Rust if not installed: https://rustup.rs/
cargo run
```

### Option 2: Docker (Recommended)
```bash
# Start all services with Docker
docker-compose up --build
```

### Option 3: Quick Start Script
```bash
# Windows
start_backend.bat
```

## Port Mappings:
- **Backend**: 8080 (host) → 8080 (container)
- **PostgreSQL**: 5432 (host) → 5432 (container)  
- **Redis**: 6379 (host) → 6379 (container)

## Environment Variables:
- `SERVER_HOST`: 0.0.0.0
- `SERVER_PORT`: 8080
- `DATABASE_URL`: postgres://postgres:postgres@postgres:5432/mr_darkpromth (Docker)
- `REDIS_URL`: redis://redis:6379 (Docker)

## Health Check:
Once running, test with: `curl http://localhost:8080/health`
