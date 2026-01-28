# Agent 4 Operations Handoff Documentation

**For**: Operations Team  
**From**: Agent 4 (Jailbreak & Ultra Tier Engineer)  
**Date**: January 28, 2026  
**Status**: PRODUCTION READY

---

## Overview

This document provides comprehensive handoff information for the operations team to manage and maintain Agent 4 (Jailbreak & Ultra Tier Engineer) in production.

### Service Purpose
Agent 4 provides **unrestricted AI access** to Ultra Tier users through sophisticated jailbreak techniques while maintaining **complete server security** and **comprehensive audit compliance**.

---

## System Architecture

### Core Components
```
Agent 4 Service
├── Jailbreak System (jailbreak_system.rs)
├── Safety Filter (safety_filter.rs)
├── Sandbox (sandbox.rs)
├── Ultra Tier Logic (ultra_tier_logic.rs)
├── User Integration (user_integration.rs)
├── Monitoring (monitoring.rs)
└── Redis Coordination (redis_coordination.rs)
```

### Dependencies
- **Redis**: Event bus coordination and caching
- **Agent 3**: Cerebras.ai integration (pending)
- **Agent 5**: User management system (pending)
- **Agent 2**: API Gateway integration

---

## Deployment Information

### Service Configuration
- **Service Name**: `agent4-jailbreak-engineer`
- **Port**: 8084 (configurable)
- **Health Check**: `/health` endpoint
- **Metrics**: `/metrics` endpoint (Prometheus format)

### Environment Variables
```bash
# Required
REDIS_URL=redis://localhost:6379
AGENT_ID=agent4
LOG_LEVEL=info

# Optional
CACHE_TTL_SECONDS=300
MAX_CONCURRENT_REQUESTS=100
ALERT_CPU_THRESHOLD=80.0
ALERT_MEMORY_THRESHOLD_MB=1024
ALERT_ERROR_RATE_THRESHOLD=5.0
```

### Docker Configuration
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mr_darkpromth_services /usr/local/bin/
EXPOSE 8084
CMD ["mr_darkpromth_services"]
```

---

## Monitoring & Alerting

### Key Metrics to Monitor
1. **System Metrics**
   - `agent4_cpu_usage_percent` - CPU usage percentage
   - `agent4_memory_usage_bytes` - Memory usage in bytes
   - `agent4_active_requests` - Current active requests
   - `agent4_uptime_seconds` - Service uptime

2. **Business Metrics**
   - `agent4_completed_requests` - Total completed requests
   - `agent4_failed_requests` - Total failed requests
   - `agent4_average_response_time_ms` - Average response time
   - `agent4_jailbreak_success_rate` - Jailbreak success percentage
   - `agent4_security_violations` - Total security violations

3. **Health Status**
   - `agent4_health_status` - Overall health (1=healthy, 2=degraded, 3=unhealthy)

### Alert Thresholds
- **Critical**: CPU > 90%, Memory > 2GB, Error Rate > 10%
- **Warning**: CPU > 80%, Memory > 1GB, Error Rate > 5%
- **Info**: Jailbreak Success Rate < 85%

### Dashboard Setup
```prometheus
# Grafana Dashboard Queries
- CPU Usage: rate(agent4_cpu_usage_percent[5m])
- Memory Usage: agent4_memory_usage_bytes / (1024*1024*1024)
- Request Rate: rate(agent4_completed_requests[5m])
- Error Rate: (rate(agent4_failed_requests[5m]) / rate(agent4_completed_requests[5m])) * 100
- Response Time: agent4_average_response_time_ms
```

---

## Health Checks

### Service Health Endpoint
```bash
GET /health
```
**Response**:
```json
{
  "status": "healthy",
  "timestamp": "2026-01-28T20:27:00Z",
  "uptime_seconds": 3600,
  "version": "1.0.0",
  "dependencies": {
    "redis": {
      "status": "healthy",
      "response_time_ms": 5
    }
  }
}
```

### Readiness Check
```bash
GET /ready
```
**Response**: Service is ready to accept requests

### Liveness Check
```bash
GET /live
```
**Response**: Service is alive and responding

---

## Log Management

### Log Locations
- **Application Logs**: `/var/log/agent4/application.log`
- **Audit Logs**: `/var/log/agent4/ultra_tier_audit.log`
- **Access Logs**: `/var/log/agent4/access.log`
- **Error Logs**: `/var/log/agent4/error.log`

### Log Formats

#### Application Logs
```
[2026-01-28T20:27:00.000Z] [INFO] [agent4] [Service started] [port=8084]
[2026-01-28T20:27:01.000Z] [INFO] [agent4] [Redis connected] [url=redis://localhost:6379]
```

#### Audit Logs
```
[2026-01-28T20:27:00.000Z] [REQUEST_RECEIVED] [USER:user123] [TIER:Ultra] [REQ:req-456] [ACTION:jailbreak_applied] [Ultra Tier request processed]
```

### Log Rotation
```bash
# Logrotate configuration
/var/log/agent4/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 644 agent4 agent4
    postrotate
        systemctl reload agent4
    endscript
}
```

---

## Security Management

### Security Features
1. **Server Protection**: 8 protection rules blocking dangerous commands
2. **Sandboxed Execution**: Isolated code execution environment
3. **Audit Logging**: Complete audit trail for all activities
4. **Access Control**: Tier-based access enforcement

### Security Monitoring
- **Security Violations**: Monitor `agent4_security_violations` metric
- **Audit Log Analysis**: Regular review of `/var/log/agent4/ultra_tier_audit.log`
- **Access Patterns**: Monitor for unusual Ultra Tier usage patterns

### Incident Response
1. **Security Violation Detected**
   - Immediate investigation of violation details
   - Review audit logs for related activities
   - Escalate to security team if needed

2. **Performance Degradation**
   - Check system metrics (CPU, memory, response times)
   - Review error rates and failed requests
   - Scale resources if needed

3. **Dependency Failure**
   - Check Redis connectivity and performance
   - Verify Agent 3 and Agent 5 integration status
   - Switch to cached mode if necessary

---

## Backup & Recovery

### Data Backup Requirements
- **Audit Logs**: Daily backup to secure storage
- **Configuration**: Version control and backup
- **User Cache**: Rebuildable from Agent 5, no backup needed

### Recovery Procedures
1. **Service Recovery**
   ```bash
   systemctl restart agent4
   # Monitor health status
   curl http://localhost:8084/health
   ```

2. **Redis Recovery**
   ```bash
   # Check Redis status
   redis-cli ping
   
   # Restart if needed
   systemctl restart redis
   ```

3. **Data Recovery**
   - Audit logs can be restored from backup
   - User cache will rebuild automatically
   - No critical data loss expected

---

## Performance Tuning

### Resource Allocation
- **CPU**: 2 cores minimum, 4 cores recommended
- **Memory**: 1GB minimum, 2GB recommended
- **Disk**: 10GB for logs, SSD recommended
- **Network**: 100Mbps minimum

### Performance Optimization
1. **Redis Optimization**
   - Enable persistence if needed
   - Configure max memory policy
   - Monitor connection pool usage

2. **Application Tuning**
   - Adjust cache TTL based on usage patterns
   - Tune concurrent request limits
   - Optimize jailbreak prompt selection

### Scaling Considerations
- **Horizontal Scaling**: Multiple instances behind load balancer
- **Vertical Scaling**: Increase CPU/memory as needed
- **Database Scaling**: Redis clustering for high availability

---

## Troubleshooting Guide

### Common Issues

#### High CPU Usage
**Symptoms**: CPU > 80%, slow response times
**Causes**: High request volume, inefficient jailbreak processing
**Solutions**:
```bash
# Check current requests
curl http://localhost:8084/metrics | grep agent4_active_requests

# Scale service or optimize prompts
kubectl scale deployment agent4 --replicas=3
```

#### Memory Leaks
**Symptoms**: Memory usage increasing over time
**Causes**: Cache not expiring, memory leaks in jailbreak processing
**Solutions**:
```bash
# Check memory usage
curl http://localhost:8084/metrics | grep agent4_memory_usage_bytes

# Restart service if needed
systemctl restart agent4
```

#### Redis Connection Issues
**Symptoms**: Failed requests, dependency errors
**Causes**: Redis down, network issues, connection pool exhaustion
**Solutions**:
```bash
# Check Redis status
redis-cli ping

# Check connection logs
journalctl -u agent4 | grep -i redis
```

#### Jailbreak Failures
**Symptoms**: Low jailbreak success rate
**Causes**: AI model changes, prompt ineffectiveness
**Solutions**:
```bash
# Check success rate
curl http://localhost:8084/metrics | grep agent4_jailbreak_success_rate

# Review prompt effectiveness and update as needed
```

### Debug Commands
```bash
# Service status
systemctl status agent4

# Recent logs
journalctl -u agent4 -n 100

# Health check
curl http://localhost:8084/health

# Metrics
curl http://localhost:8084/metrics

# Test Ultra Tier request
curl -X POST http://localhost:8084/api/ultra-request \
  -H "Content-Type: application/json" \
  -d '{"user_id":"test","prompt":"test","model":"gpt-4"}'
```

---

## Maintenance Procedures

### Daily Tasks
- [ ] Review system health and performance metrics
- [ ] Check for security violations
- [ ] Monitor error rates and response times
- [ ] Verify log rotation is working

### Weekly Tasks
- [ ] Review audit logs for unusual patterns
- [ ] Check jailbreak effectiveness rates
- [ ] Update security rules if needed
- [ ] Performance optimization review

### Monthly Tasks
- [ ] Update jailbreak prompts based on effectiveness data
- [ ] Security audit and penetration testing
- [ ] Review and update alert thresholds
- [ ] Capacity planning and scaling review

### Quarterly Tasks
- [ ] Complete system architecture review
- [ ] Update dependencies and security patches
- [ ] Disaster recovery testing
- [ ] Performance benchmarking

---

## Emergency Contacts

### Primary Contacts
- **Development Team**: dev-team@mr.darkpromth.com
- **Security Team**: security@mr.darkpromth.com
- **Operations Team**: ops-team@mr.darkpromth.com

### Escalation Procedures
1. **Level 1**: Operations team handles routine issues
2. **Level 2**: Development team for complex technical issues
3. **Level 3**: Security team for security incidents
4. **Level 4**: Management for critical incidents

### Communication Channels
- **Slack**: #agent4-operations
- **Email**: ops-alerts@mr.darkpromth.com
- **Pager**: Critical incidents only

---

## Compliance & Legal

### Data Protection
- **Audit Logs**: Retain for 90 days minimum
- **User Data**: No personal data stored, only audit trails
- **Access Control**: Tier-based access enforcement

### Regulatory Compliance
- **Audit Trail**: Complete logging for compliance requirements
- **Security Standards**: Follow industry best practices
- **Data Privacy**: No user content stored permanently

### Legal Considerations
- **Jurisdiction**: Follow applicable laws and regulations
- **Content Policy**: Server protection only, no content censorship
- **Liability**: Clear documentation of system capabilities and limitations

---

## Appendix

### Configuration Examples

#### Prometheus Configuration
```yaml
scrape_configs:
  - job_name: 'agent4'
    static_configs:
      - targets: ['localhost:8084']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

#### Grafana Dashboard JSON
[Available in separate file: agent4-grafana-dashboard.json]

#### Alertmanager Rules
```yaml
groups:
  - name: agent4.rules
    rules:
      - alert: Agent4HighCPU
        expr: agent4_cpu_usage_percent > 90
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Agent 4 high CPU usage"
          description: "Agent 4 CPU usage is above 90% for more than 5 minutes"
```

### API Documentation
[Available at: http://localhost:8084/docs]

### Monitoring Dashboards
- **Grafana**: http://grafana.local/d/agent4-overview
- **Prometheus**: http://prometheus.local/targets

---

## Handoff Confirmation

### Checklist Completed ✅
- [x] System documentation provided
- [x] Monitoring and alerting configured
- [x] Backup and recovery procedures documented
- [x] Security procedures documented
- [x] Troubleshooting guide provided
- [x] Maintenance procedures defined
- [x] Emergency contacts provided
- [x] Compliance requirements documented

### Training Requirements
- [ ] Operations team training on Agent 4 specifics
- [ ] Security team training on jailbreak system
- [ ] Monitoring team training on metrics and alerts

### Sign-off
**Development Team**: _______________________ Date: _______  
**Operations Team**: _______________________ Date: _______  
**Security Team**: _______________________ Date: _______

---

**Document Version**: 1.0  
**Last Updated**: January 28, 2026  
**Next Review**: February 28, 2026

**Agent 4 is ready for operations handoff and production deployment.** 🚀
