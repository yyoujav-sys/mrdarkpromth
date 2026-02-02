from flask import Flask, jsonify
import time
import os
import psycopg2
import redis
from prometheus_client import Counter, Histogram, generate_latest, CONTENT_TYPE_LATEST
from gunicorn.app.base import BaseApplication
import multiprocessing

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

class StandaloneApplication(BaseApplication):
    def __init__(self, app, options=None):
        self.application = app
        self.options = options or {}
        super().__init__()

    def load_config(self):
        config = {
            "bind": "0.0.0.0:8080",
            "workers": multiprocessing.cpu_count() * 2 + 1,
            "worker_class": "sync",
            "worker_connections": 1000,
            "timeout": 30,
            "keepalive": 2,
            "max_requests": 1000,
            "max_requests_jitter": 100,
            "preload_app": True,
            "accesslog": "-",
            "errorlog": "-",
            "loglevel": "info",
        }
        
        # Override with custom options
        config.update(self.options)
        
        for key, value in config.items():
            self.cfg.set(key, value)

    def load(self):
        return self.application

if __name__ == "__main__":
    # For development
    app.run(host="0.0.0.0", port=8080, debug=False)
else:
    # For production with Gunicorn
    standalone_app = StandaloneApplication(app)
