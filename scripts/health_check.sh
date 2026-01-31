#!/bin/bash

# MR.DarkPromth Health Check & Failover Script
# Monitors API health and restarts services if necessary

API_URL="http://localhost:8080/health"
MAX_RETRIES=3
RETRY_INTERVAL=5

check_health() {
    local retries=0
    while [ $retries -lt $MAX_RETRIES ]; do
        if curl -s -f "$API_URL" > /dev/null; then
            return 0
        fi
        echo "Health check failed (attempt $((retries+1))/$MAX_RETRIES). Retrying in $RETRY_INTERVAL seconds..."
        retries=$((retries+1))
        sleep $RETRY_INTERVAL
    done
    return 1
}

if check_health; then
    echo "$(date): System is healthy."
else
    echo "$(date): System is UNHEALTHY! Initiating failover/restart..."
    # Log the failure
    echo "$(date): API failure detected" >> /home/ubuntu/MR.Darkpromth/logs/failover.log
    
    # Restart the API container
    docker-compose -f /home/ubuntu/MR.Darkpromth/docker-compose.yml restart api
    
    # Notify admin (mock)
    echo "Admin notified of system restart."
fi
