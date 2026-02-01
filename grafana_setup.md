# Grafana Setup Instructions

## 1. Access Grafana
- URL: http://localhost:3001
- Username: admin
- Password: admin

## 2. Add Prometheus Data Source
1. Go to Configuration > Data Sources
2. Click "Add data source"
3. Select "Prometheus"
4. URL: http://mr_darkpromth_prometheus:9090
5. Click "Save & Test"

## 3. Create Alert Dashboard

### Dashboard 1: System Health
- Panel 1: CPU Usage (node_cpu_seconds_total)
- Panel 2: Memory Usage (node_memory_MemAvailable_bytes)
- Panel 3: Disk Usage (node_filesystem_avail_bytes)
- Panel 4: Network Traffic (node_network_receive_bytes_total)

### Dashboard 2: Application Metrics
- Panel 1: HTTP Requests Rate (rate(http_requests_total[5m]))
- Panel 2: Response Time (histogram_quantile(0.95, http_request_duration_seconds_bucket))
- Panel 3: Error Rate (rate(http_requests_total{status=~"5.."}[5m]))
- Panel 4: Active Connections (active_connections)

## 4. Configure Alerts
1. Go to Alerting > Alert rules
2. Create new alert rule
3. Set conditions:
   - CPU > 80% for 5 minutes
   - Memory > 85% for 5 minutes
   - Error Rate > 5% for 2 minutes
   - Response Time > 500ms for 5 minutes

## 5. Notification Channels
1. Go to Alerting > Contact points
2. Add email/webhook/slack notification
3. Test notification channel

## 6. Import Dashboard Templates
- Node Exporter Full: 1860
- Docker Dashboard: 893
- Prometheus Stats: 2

## Quick API Commands (after login)
```bash
# Get auth token
TOKEN=$(curl -X POST -H "Content-Type: application/json" -d '{"user":"admin","password":"admin"}' http://localhost:3001/api/auth/login | jq -r '.authToken')

# Add datasource
curl -X POST -H "Content-Type: application/json" -H "Authorization: Bearer $TOKEN" \
  -d '{"name":"Prometheus","type":"prometheus","url":"http://mr_darkpromth_prometheus:9090","access":"proxy"}' \
  http://localhost:3001/api/datasources
```
