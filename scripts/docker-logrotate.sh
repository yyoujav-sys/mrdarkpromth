#!/bin/bash
# Docker Container Log Rotation Script
# Rotates logs for all Docker containers to prevent disk full issues

set -euo pipefail

LOG_MAX_SIZE="100m"
LOG_MAX_FILE="5"

echo "=== Docker Container Log Rotation ==="
echo "Timestamp: $(date)"

# Get all container log paths
for container in $(docker ps -q); do
    container_name=$(docker inspect --format='{{.Name}}' "$container" | sed 's/\///')
    log_path=$(docker inspect --format='{{.LogPath}}' "$container")
    
    if [ -f "$log_path" ]; then
        log_size=$(du -h "$log_path" 2>/dev/null | cut -f1)
        echo "Container: $container_name"
        echo "  Log path: $log_path"
        echo "  Current size: $log_size"
    fi
done

# Truncate large container logs (alternative approach)
echo ""
echo "Truncating container logs larger than 100MB..."

for container in $(docker ps -q); do
    container_name=$(docker inspect --format='{{.Name}}' "$container" | sed 's/\///')
    log_path=$(docker inspect --format='{{.LogPath}}' "$container")
    
    if [ -f "$log_path" ]; then
        # Get size in bytes
        size=$(stat -f%z "$log_path" 2>/dev/null || stat -c%s "$log_path" 2>/dev/null || echo "0")
        max_bytes=$((100 * 1024 * 1024)) # 100MB
        
        if [ "$size" -gt "$max_bytes" ]; then
            echo "Truncating $container_name log (size: $size bytes)"
            truncate -s 0 "$log_path"
        fi
    fi
done

echo ""
echo "=== Log Rotation Complete ==="

# Tip: To permanently limit container logs, use docker-compose with:
# logging:
#   driver: "json-file"
#   options:
#     max-size: "100m"
#     max-file: "5"
