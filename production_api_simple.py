from flask import Flask, jsonify, request
import time
import os
import psycopg2
import redis
from prometheus_client import Counter, Histogram, generate_latest, CONTENT_TYPE_LATEST

app = Flask(__name__)

# Metrics
REQUEST_COUNT = Counter("http_requests_total", "Total HTTP requests", ["method", "endpoint"])
REQUEST_DURATION = Histogram("http_request_duration_seconds", "HTTP request duration")
ACTIVE_CONNECTIONS = Counter("active_connections", "Active database connections")

def get_db_connection():
    try:
        conn = psycopg2.connect(os.environ["DATABASE_URL"])
        ACTIVE_CONNECTIONS.inc()
        return conn, "connected"
    except Exception as e:
        return None, str(e)

def get_redis_connection():
    try:
        r = redis.Redis.from_url(os.environ["REDIS_URL"])
        r.ping()
        return r, "connected"
    except Exception as e:
        return None, str(e)

@app.route("/health")
def health():
    return jsonify({
        "status": "healthy",
        "service": "mr_darkpromth_api",
        "timestamp": time.time()
    })

@app.route("/api/status")
def status():
    db_conn, db_status = get_db_connection()
    if db_conn:
        db_conn.close()
    
    redis_conn, redis_status = get_redis_connection()
    
    return jsonify({
        "api": "running",
        "database": db_status,
        "redis": redis_status,
        "environment": "production"
    })

@app.route("/api/info")
def info():
    return jsonify({
        "name": "MR.DarkPromth API",
        "version": "1.0.0",
        "environment": "production",
        "features": [
            "health",
            "status",
            "database",
            "redis",
            "metrics"
        ]
    })

@app.route("/metrics")
def metrics():
    return generate_latest(), 200, {"Content-Type": CONTENT_TYPE_LATEST}

@app.route("/api/auth/register", methods=["POST"])
def register():
    try:
        # Simple mock response
        return jsonify({
            "message": "User registered successfully",
            "user": {
                "email": "test@example.com",
                "name": "Test User",
                "id": "mock_user_id"
            }
        }), 201
    except Exception as e:
        return jsonify({"error": str(e)}), 400

@app.route("/api/auth/login", methods=["POST"])
def login():
    try:
        # Simple mock response
        return jsonify({
            "message": "Login successful",
            "token": "mock_jwt_token",
            "user": {
                "email": "test@example.com",
                "name": "Test User",
                "id": "mock_user_id"
            }
        }), 200
    except Exception as e:
        return jsonify({"error": str(e)}), 400

@app.route("/api/auth/logout", methods=["POST"])
def logout():
    return jsonify({"message": "Logged out successfully"}), 200

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=8080)
