# Infrastructure & Performance Test Plan - MR.DarkPromth

## Overview

This document outlines infrastructure, load, and performance testing for the MR.DarkPromth platform to ensure stability, scalability, and responsiveness under various conditions.

## Test Environment

### Production-like Environment
- **Server**: 4 vCPU, 8GB RAM (minimum)
- **OS**: Linux (Ubuntu 22.04 LTS)
- **Docker**: Docker Engine 24.0+
- **Docker Compose**: v2.20+
- **Network**: 1Gbps internal, 100Mbps external

### Monitoring Stack
- **Prometheus**: Metrics collection (port 9090)
- **Grafana**: Visualization (port 3000)
- **Jaeger**: Distributed tracing (port 16686)
- **Elasticsearch**: Log aggregation (port 9200)
- **Kibana**: Log visualization (port 5601)

## Performance Test Categories

### 1. Load Testing

#### 1.1 API Endpoint Load Tests

| Test ID | Endpoint | Method | Concurrent Users | Duration | Expected RPS | Max Response Time |
|---------|----------|--------|------------------|----------|--------------|-------------------|
| LOAD-001 | /health | GET | 100 | 5 min | > 1000 | < 50ms |
| LOAD-002 | /api/auth/login | POST | 50 | 5 min | > 200 | < 200ms |
| LOAD-003 | /api/chat | POST | 50 | 10 min | > 100 | < 2000ms |
| LOAD-004 | /api/users/me | GET | 100 | 5 min | > 500 | < 100ms |
| LOAD-005 | /api/jailbreak/prompts | GET | 100 | 5 min | > 800 | < 100ms |
| LOAD-006 | /metrics | GET | 50 | 5 min | > 300 | < 100ms |

#### 1.2 WebSocket Load Tests

| Test ID | Scenario | Concurrent Connections | Messages/Sec | Duration | Expected Latency |
|---------|----------|------------------------|--------------|----------|------------------|
| WS-LOAD-001 | Chat connections | 500 | - | 10 min | < 100ms connect |
| WS-LOAD-002 | Message broadcast | 100 | 10/user | 10 min | < 500ms delivery |
| WS-LOAD-003 | Concurrent chats | 200 | 5/user | 10 min | < 1000ms response |

#### 1.3 Mixed Load Profile

| Test ID | Scenario | Users | Ratio | Duration |
|---------|----------|-------|-------|----------|
| MIX-001 | Realistic usage | 200 | 60% read, 30% chat, 10% auth | 30 min |
| MIX-002 | Peak traffic | 500 | 40% read, 50% chat, 10% auth | 15 min |
| MIX-003 | Sustained load | 300 | Even distribution | 60 min |

### 2. Stress Testing

#### 2.1 Breaking Point Tests

| Test ID | Scenario | Starting Load | Increment | Max Load | Target |
|---------|----------|---------------|-----------|----------|--------|
| STRESS-001 | API stress | 100 users | +50/30s | 1000 users | Find breaking point |
| STRESS-002 | DB connections | 10 conn | +10/1min | 500 conn | Connection limit |
| STRESS-003 | Memory pressure | Normal | +100MB/min | 8GB | Memory leak detection |
| STRESS-004 | CPU saturation | 50% | +10%/5min | 100% | CPU bottleneck |

#### 2.2 Recovery Tests

| Test ID | Scenario | Stress Duration | Recovery Time | Success Criteria |
|---------|----------|-----------------|---------------|------------------|
| REC-001 | Crash recovery | 5 min overload | < 60s | Services auto-restart |
| REC-002 | DB failover | Primary down | < 30s | Seamless failover |
| REC-003 | Redis restart | Cache cleared | < 10s | Cache rebuild |
| REC-004 | Network partition | 2 min isolation | < 30s | Reconnection sync |

### 3. Endurance Testing

| Test ID | Scenario | Duration | Target Load | Success Criteria |
|---------|----------|----------|-------------|------------------|
| ENDUR-001 | Sustained API load | 24 hours | 100 users | < 0.1% error rate |
| ENDUR-002 | WebSocket stability | 24 hours | 50 connections | No memory leaks |
| ENDUR-003 | Background jobs | 48 hours | Continuous | All jobs complete |
| ENDUR-004 | Log rotation | 7 days | Normal usage | No disk full errors |

### 4. Spike Testing

| Test ID | Scenario | Normal Load | Spike Load | Spike Duration | Recovery |
|---------|----------|-------------|------------|----------------|----------|
| SPIKE-001 | Flash crowd | 50 users | 1000 users | 2 minutes | < 5 minutes |
| SPIKE-002 | Marketing campaign | 100 users | 2000 users | 5 minutes | < 10 minutes |
| SPIKE-003 | Viral content | 200 users | 5000 users | 10 minutes | < 15 minutes |

### 5. Scalability Testing

#### 5.1 Horizontal Scaling

| Test ID | Scenario | Initial Instances | Scale To | Metric | Target |
|---------|----------|-------------------|----------|--------|--------|
| SCALE-001 | API scaling | 1 | 5 | CPU > 70% | Auto-scale trigger |
| SCALE-002 | DB read scaling | 1 replica | 3 replicas | Read latency | < 50ms |
| SCALE-003 | Cache scaling | 1 Redis | 3 Redis cluster | Hit rate | > 95% |

#### 5.2 Vertical Scaling

| Test ID | Resource | Initial | Scale To | Improvement |
|---------|----------|---------|----------|-------------|
| VERT-001 | API CPU | 2 cores | 4 cores | 80%+ throughput |
| VERT-002 | API RAM | 4GB | 8GB | Handle 2x connections |
| VERT-003 | DB RAM | 4GB | 16GB | Cache 4x data |

## Infrastructure Component Tests

### 1. Docker Containers

| Test ID | Component | Test | Expected Result |
|---------|-----------|------|-----------------|
| DOCKER-001 | Backend | Container start | < 30s startup |
| DOCKER-002 | Backend | Health check | Returns 200 within 60s |
| DOCKER-003 | Backend | Auto-restart | Restarts on crash |
| DOCKER-004 | Backend | Resource limits | Respects CPU/memory limits |
| DOCKER-005 | Frontend | Nginx serving | Serves static files |
| DOCKER-006 | Postgres | Data persistence | Data survives restart |
| DOCKER-007 | Redis | Cache persistence | Key expiration works |
| DOCKER-008 | All | Network isolation | Containers isolated |

### 2. Database Performance

| Test ID | Test | Query Type | Target QPS | Max Latency |
|---------|------|------------|------------|-------------|
| DB-001 | Simple SELECT | Indexed | > 10,000 | < 5ms |
| DB-002 | Complex JOIN | Multi-table | > 1,000 | < 50ms |
| DB-003 | INSERT batch | Single row | > 5,000 | < 10ms |
| DB-004 | UPDATE | Indexed | > 3,000 | < 10ms |
| DB-005 | Full-text search | GIN index | > 500 | < 100ms |
| DB-006 | Connection pool | - | 100 active | No exhaustion |

### 3. Redis Performance

| Test ID | Test | Operation | Target QPS | Max Latency |
|---------|------|-----------|------------|-------------|
| REDIS-001 | GET | String | > 100,000 | < 1ms |
| REDIS-002 | SET | String | > 80,000 | < 1ms |
| REDIS-003 | HGETALL | Hash | > 50,000 | < 2ms |
| REDIS-004 | LPUSH/RPOP | List | > 60,000 | < 1ms |
| REDIS-005 | PUBLISH | Pub/Sub | > 100,000 | < 1ms |
| REDIS-006 | Pipeline | Mixed | > 200,000 | < 5ms |

### 4. Network Performance

| Test ID | Test | Scenario | Target | Max Latency |
|---------|------|----------|--------|-------------|
| NET-001 | Internal latency | Container-to-container | < 1ms | - |
| NET-002 | External latency | API to Cerebras | < 100ms | - |
| NET-003 | Throughput | File upload | > 10MB/s | - |
| NET-004 | Concurrent connections | WebSocket | 10,000 | < 50ms |

## Resource Utilization Targets

### API Server
| Metric | Normal | Warning | Critical |
|--------|--------|---------|----------|
| CPU | < 50% | 50-80% | > 80% |
| Memory | < 60% | 60-85% | > 85% |
| Disk I/O | < 100MB/s | 100-500MB/s | > 500MB/s |
| Network | < 50MB/s | 50-100MB/s | > 100MB/s |

### Database
| Metric | Normal | Warning | Critical |
|--------|--------|---------|----------|
| CPU | < 40% | 40-70% | > 70% |
| Memory | < 70% | 70-90% | > 90% |
| Connections | < 80% | 80-95% | > 95% |
| Lock wait | < 10ms | 10-100ms | > 100ms |

### Redis
| Metric | Normal | Warning | Critical |
|--------|--------|---------|----------|
| Memory | < 70% | 70-90% | > 90% |
| Hit rate | > 95% | 90-95% | < 90% |
| Evicted keys | 0 | < 100/min | > 100/min |
| Connected clients | < 500 | 500-1000 | > 1000 |

## Test Tools

### Load Testing
```bash
# k6 - Modern load testing
k6 run --vus 100 --duration 5m load-test.js

# Apache Bench (ab)
ab -n 10000 -c 100 http://localhost:8080/health

# wrk
wrk -t12 -c400 -d30s http://localhost:8080/health

# Artillery
artillery quick --count 100 --num 50 http://localhost:8080/api/chat
```

### WebSocket Testing
```bash
# websocat for manual testing
websocat wss://localhost:8080/ws/chat

# Custom WebSocket load tester
node ws-load-test.js --connections 500 --duration 600
```

### Monitoring During Tests
```bash
# Docker stats
docker stats --no-stream

# PostgreSQL stats
docker exec mr_darkpromth_postgres psql -U postgres -c "SELECT * FROM pg_stat_activity;"

# Redis stats
docker exec mr_darkpromth_redis redis-cli info stats

# API metrics
curl http://localhost:8080/metrics
```

## Performance Test Scenarios

### Scenario 1: Normal Business Hours
```javascript
// k6 script example
export const options = {
  stages: [
    { duration: '5m', target: 50 },   // Ramp up
    { duration: '30m', target: 50 },  // Steady state
    { duration: '5m', target: 0 },    // Ramp down
  ],
};

export default function() {
  http.get('https://localhost:8080/health');
  sleep(1);
}
```

### Scenario 2: Peak Traffic
```javascript
export const options = {
  stages: [
    { duration: '2m', target: 100 },
    { duration: '5m', target: 400 },
    { duration: '10m', target: 400 },
    { duration: '5m', target: 100 },
    { duration: '2m', target: 0 },
  ],
};
```

### Scenario 3: Stress to Breaking Point
```javascript
export const options = {
  stages: [
    { duration: '2m', target: 100 },
    { duration: '5m', target: 200 },
    { duration: '5m', target: 400 },
    { duration: '5m', target: 600 },
    { duration: '5m', target: 800 },
    { duration: '5m', target: 1000 },
    { duration: '10m', target: 1000 },
    { duration: '5m', target: 0 },
  ],
};
```

## Acceptance Criteria

### Performance Requirements
- [ ] p50 response time < 100ms for simple queries
- [ ] p95 response time < 500ms for complex queries
- [ ] p99 response time < 1000ms for all queries
- [ ] Error rate < 0.1% under normal load
- [ ] Error rate < 1% under peak load
- [ ] Zero downtime during rolling updates
- [ ] Automatic recovery from failures < 60s

### Scalability Requirements
- [ ] Linear scaling up to 5x normal load
- [ ] Auto-scaling triggers within 30s
- [ ] Database handles 10,000 QPS
- [ ] Redis handles 100,000 ops/sec
- [ ] WebSocket supports 10,000 concurrent

### Resource Requirements
- [ ] CPU utilization < 70% at normal load
- [ ] Memory utilization < 80% at normal load
- [ ] No memory leaks over 24h test
- [ ] Disk usage growth < 10GB/day
- [ ] Network bandwidth < 50% capacity

## Test Reports

### Metrics to Capture
1. Response times (p50, p95, p99, max)
2. Throughput (requests per second)
3. Error rates (by status code)
4. Resource utilization (CPU, memory, disk, network)
5. Database metrics (connections, slow queries, locks)
6. Redis metrics (hit rate, evictions, memory)
7. Custom business metrics (chat messages, active users)

### Report Template
- Test date and duration
- Test scenario description
- Load pattern used
- Results summary
- Performance graphs
- Bottleneck identification
- Recommendations

---
*Last Updated: 2026-01-31*
*Version: 1.0*
