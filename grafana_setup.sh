#!/bin/bash

echo '=== Grafana Auto Setup Script ==='

# Wait for Grafana to be ready
until curl -f http://localhost:3001/api/health; do
  echo 'Waiting for Grafana...'
  sleep 5
done

echo 'Grafana is ready!'

# Create API key
API_KEY=$(curl -X POST -H 'Content-Type: application/json' \
  -d '{"name":"setup-key","role":"Admin"}' \
  http://admin:newadmin123@localhost:3001/api/auth/keys | \
  grep -o '"key":"[^"]*"' | cut -d'"' -f4)

echo "API Key: $API_KEY"

# Add Prometheus datasource
curl -X POST -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "name":"Prometheus",
    "type":"prometheus",
    "url":"http://mr_darkpromth_prometheus:9090",
    "access":"proxy"
  }' \
  http://localhost:3001/api/datasources

echo 'Datasource added!'

# Create alert rule
curl -X POST -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $API_KEY" \
  -d '{
    "name":"High CPU Alert",
    "folderId":1,
    "orgId":1,
    "ruleGroup":"System Alerts",
    "for":"5m",
    "condition":"A",
    "data":[
      {
        "refId":"A",
        "queryType":"",
        "relativeTimeRange":{"from":300,"to":0},
        "datasourceUid":"prometheus",
        "model":{
          "expr":"up{job=\"mr-darkpromth-api\"} == 0",
          "interval":"",
          "refId":"A"
        }
      }
    ],
    "noDataState":"NoData",
    "execErrState":"Alerting",
    "frequency":"60s"
  }' \
  http://localhost:3001/api/v1/provisioning/alert-rules

echo 'Alert rule created!'
