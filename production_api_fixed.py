from flask import Flask, jsonify, request
import os
import psycopg2
import redis
import time
import logging
from prometheus_client import Counter, Histogram, Gauge, generate_latest, CONTENT_TYPE_LATEST

app = Flask(__name__)

# Prometheus Metrics
REQUEST_COUNT = Counter('http_requests_total', 'Total HTTP requests', ['method', 'endpoint', 'status'])
REQUEST_LATENCY = Histogram('http_request_duration_seconds', 'HTTP request latency')
ACTIVE_CONNECTIONS = Gauge('active_connections', 'Active database connections')
REDIS_CONNECTIONS = Gauge('redis_connections', 'Redis connection status')

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

def get_db_connection():
    try:
        conn = psycopg2.connect(
            host='mr_darkpromth_postgres',
            database='mr_darkpromth',
            user='postgres',
            password='postgres'
        )
        ACTIVE_CONNECTIONS.inc()
        return conn, 'connected'
    except Exception as e:
        logger.error(f'Database connection error: {e}')
        return None, f'disconnected: {str(e)}'

def get_redis_connection():
    try:
        r = redis.Redis(host='mr_darkpromth_redis', port=6379, decode_responses=True)
        r.ping()
        REDIS_CONNECTIONS.set(1)
        return r, 'connected'
    except Exception as e:
        logger.error(f'Redis connection error: {e}')
        REDIS_CONNECTIONS.set(0)
        return None, f'disconnected: {str(e)}'

@app.before_request
def before_request():
    request.start_time = time.time()

@app.after_request
def after_request(response):
    request_latency = time.time() - request.start_time
    REQUEST_LATENCY.observe(request_latency)
    REQUEST_COUNT.labels(
        method=request.method,
        endpoint=request.endpoint or 'unknown',
        status=response.status_code
    ).inc()
    return response

@app.route('/metrics')
def metrics():
    return generate_latest(), 200, {'Content-Type': CONTENT_TYPE_LATEST}

@app.route('/health')
def health():
    return jsonify({
        'status': 'healthy',
        'service': 'mr_darkpromth_api',
        'timestamp': time.time()
    })

@app.route('/api/status')
def status():
    # Test database connection
    db_conn, db_status = get_db_connection()
    if db_conn:
        db_conn.close()
        ACTIVE_CONNECTIONS.dec()
    
    # Test Redis connection
    redis_conn, redis_status = get_redis_connection()
    
    return jsonify({
        'api': 'running',
        'database': db_status,
        'redis': redis_status,
        'environment': 'production'  # Fixed: Changed from 'development' to 'production'
    })

@app.route('/api/test/db')
def test_db():
    conn, status = get_db_connection()
    if conn:
        try:
            cursor = conn.cursor()
            cursor.execute('SELECT 1 as test')
            result = cursor.fetchone()
            cursor.close()
            conn.close()
            ACTIVE_CONNECTIONS.dec()
            return jsonify({
                'database_test': 'success',
                'result': result[0],
                'status': status
            })
        except Exception as e:
            ACTIVE_CONNECTIONS.dec()
            return jsonify({
                'database_test': 'failed',
                'error': str(e),
                'status': status
            }), 500
    else:
        return jsonify({
            'database_test': 'failed',
            'status': status
        }), 500

@app.route('/api/test/redis')
def test_redis():
    redis_conn, status = get_redis_connection()
    if redis_conn:
        try:
            # Test set and get
            test_key = 'test_key'
            test_value = f'test_value_{int(time.time())}'
            redis_conn.set(test_key, test_value, ex=60)
            retrieved_value = redis_conn.get(test_key)
            return jsonify({
                'redis_test': 'success',
                'set_value': test_value,
                'get_value': retrieved_value,
                'status': status
            })
        except Exception as e:
            return jsonify({
                'redis_test': 'failed',
                'error': str(e),
                'status': status
            }), 500
    else:
        return jsonify({
            'redis_test': 'failed',
            'status': status
        }), 500

@app.route('/api/info')
def info():
    return jsonify({
        'service': 'MR.DarkPromth API',
        'version': '1.1.0',
        'endpoints': [
            '/health',
            '/api/status',
            '/api/test/db',
            '/api/test/redis',
            '/api/info',
            '/metrics'
        ]
    })

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=8080, debug=True)
