# Grafana Manual Setup Guide

## Access Information
- URL: http://localhost:3001
- Username: admin
- Password: newadmin123

## Manual Steps Required

### 1. Add Prometheus Data Source
1. Go to Configuration > Data Sources
2. Click "Add data source"
3. Select "Prometheus"
4. URL: http://mr_darkpromth_prometheus:9090
5. Click "Save and Test"

### 2. Create Dashboard
1. Go to Dashboards > New Dashboard
2. Add panels:
   - HTTP Requests Rate: rate(http_requests_total[5m])
   - Response Time: histogram_quantile(0.95, http_request_duration_seconds_bucket)
   - Error Rate: rate(http_requests_total{status=~"5.."}[5m])
   - Container Status: up

### 3. Configure Alerts
1. Go to Alerting > Alert rules
2. Create alert for:
   - API down: up{job="mr-darkpromth-api"} == 0
   - High error rate: rate(http_requests_total{status=~"5.."}[5m]) > 0.05
   - High response time: histogram_quantile(0.95, http_request_duration_seconds_bucket) > 0.5

### 4. Test Alerts
1. Stop API container: docker stop mr_darkpromth_api
2. Check alert fires in Grafana
3. Restart API: docker start mr_darkpromth_api
4. Verify alert resolves

## Quick Commands
```bash
# Test API endpoint
curl http://localhost:8080/metrics

# Check Prometheus targets
curl http://localhost:9090/api/v1/targets

# Generate some load
ab -n 100 -c 10 http://localhost:8080/health
```
