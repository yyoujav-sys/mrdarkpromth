import http from 'k6/http'
import { check, sleep, group } from 'k6'
import { Counter, Rate, Trend } from 'k6/metrics'

// Configuration
const BASE_URL = __ENV.LOAD_TEST_BASE_URL || 'http://localhost:8080'

// Custom metrics
const errorRate = new Rate('errors')
const apiLatency = new Trend('api_latency')
const successfulRequests = new Counter('successful_requests')

// Test scenarios
export const options = {
    scenarios: {
        // Smoke test - verify system works
        smoke: {
            executor: 'constant-vus',
            vus: 1,
            duration: '30s',
            startTime: '0s',
            exec: 'smokeTest',
        },
        // Load test - normal load
        load: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '1m', target: 20 },   // Ramp up to 20 users
                { duration: '3m', target: 20 },   // Stay at 20 users
                { duration: '1m', target: 50 },   // Ramp up to 50 users
                { duration: '3m', target: 50 },   // Stay at 50 users
                { duration: '1m', target: 0 },    // Ramp down
            ],
            startTime: '30s',
            exec: 'loadTest',
        },
        // Stress test - find breaking point
        stress: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '2m', target: 100 },  // Ramp to 100 users
                { duration: '5m', target: 100 },  // Stay at 100 users
                { duration: '2m', target: 200 },  // Ramp to 200 users
                { duration: '5m', target: 200 },  // Stay at 200 users
                { duration: '2m', target: 0 },    // Ramp down
            ],
            startTime: '10m',
            exec: 'stressTest',
        },
        // Spike test - sudden traffic spike
        spike: {
            executor: 'ramping-vus',
            startVUs: 0,
            stages: [
                { duration: '10s', target: 10 },  // Warmup
                { duration: '10s', target: 300 }, // Spike to 300 users
                { duration: '30s', target: 300 }, // Hold spike
                { duration: '10s', target: 10 },  // Drop back
                { duration: '1m', target: 10 },   // Recovery
            ],
            startTime: '27m',
            exec: 'spikeTest',
        },
    },
    thresholds: {
        http_req_duration: ['p(95)<2000', 'p(99)<5000'],
        http_req_failed: ['rate<0.05'],
        errors: ['rate<0.1'],
    },
}

// Helper function to make authenticated requests
function getAuthHeader(token) {
    return token ? { Authorization: `Bearer ${token}` } : {}
}

// Smoke test - basic functionality
export function smokeTest() {
    group('Health Check', () => {
        const res = http.get(`${BASE_URL}/health`)
        check(res, {
            'health check status 200': (r) => r.status === 200,
            'health check has body': (r) => r.body && r.body.length > 0,
        })
        apiLatency.add(res.timings.duration)
    })

    group('API Version', () => {
        const res = http.get(`${BASE_URL}/api/version`)
        check(res, {
            'version endpoint accessible': (r) => r.status === 200 || r.status === 404,
        })
    })

    sleep(1)
}

// Load test - normal traffic simulation
export function loadTest() {
    group('Public Endpoints', () => {
        // Health check
        let res = http.get(`${BASE_URL}/health`)
        check(res, { 'health OK': (r) => r.status === 200 })

        // Metrics endpoint
        res = http.get(`${BASE_URL}/metrics`)
        check(res, { 'metrics accessible': (r) => r.status === 200 })

        // API documentation (if exists)
        res = http.get(`${BASE_URL}/api/docs`)
        check(res, { 'docs accessible': (r) => r.status === 200 || r.status === 404 })
    })

    group('Authentication Flow', () => {
        const payload = JSON.stringify({
            email: 'loadtest@example.com',
            password: 'LoadTest123!',
        })

        const res = http.post(`${BASE_URL}/api/auth/login`, payload, {
            headers: { 'Content-Type': 'application/json' },
        })

        // We expect either 200 (success) or 401 (invalid credentials)
        const success = check(res, {
            'login endpoint responsive': (r) => r.status === 200 || r.status === 401 || r.status === 400,
        })

        if (!success) {
            errorRate.add(1)
        } else {
            successfulRequests.add(1)
        }

        apiLatency.add(res.timings.duration)
    })

    sleep(Math.random() * 2 + 1) // Random sleep 1-3 seconds
}

// Stress test - push the system
export function stressTest() {
    group('High Load Endpoints', () => {
        // Health check
        let res = http.get(`${BASE_URL}/health`)
        check(res, { 'health under stress': (r) => r.status === 200 })

        // Simulate API calls
        const endpoints = [
            '/api/status',
            '/api/version',
            '/health',
            '/metrics',
        ]

        const endpoint = endpoints[Math.floor(Math.random() * endpoints.length)]
        res = http.get(`${BASE_URL}${endpoint}`)

        const success = check(res, {
            'endpoint responsive under stress': (r) => r.status === 200 || r.status === 404,
        })

        if (!success) {
            errorRate.add(1)
        } else {
            successfulRequests.add(1)
        }

        apiLatency.add(res.timings.duration)
    })

    sleep(0.5)
}

// Spike test - sudden traffic surge
export function spikeTest() {
    group('Spike Handling', () => {
        const res = http.get(`${BASE_URL}/health`)

        const success = check(res, {
            'survives spike': (r) => r.status === 200,
            'responds under 5s during spike': (r) => r.timings.duration < 5000,
        })

        if (!success) {
            errorRate.add(1)
        } else {
            successfulRequests.add(1)
        }

        apiLatency.add(res.timings.duration)
    })

    sleep(0.1) // Minimal sleep during spike
}

// Default function for CLI overrides
export default function () {
    smokeTest()
}

// Summary handler
export function handleSummary(data) {
    return {
        'stdout': textSummary(data, { indent: ' ', enableColors: true }),
        '/opt/mrdarkpromth/load-tests/results/summary.json': JSON.stringify(data, null, 2),
    }
}

function textSummary(data, opts) {
    const { metrics, root_group } = data

    let summary = '\n=== Load Test Summary ===\n\n'

    if (metrics && metrics.http_req_duration && metrics.http_req_duration.values) {
        summary += `HTTP Request Duration:\n`
        summary += `  - avg: ${metrics.http_req_duration.values.avg.toFixed(2)}ms\n`
        summary += `  - p95: ${metrics.http_req_duration.values['p(95)'].toFixed(2)}ms\n`
        summary += `  - p99: ${metrics.http_req_duration.values['p(99)'].toFixed(2)}ms\n\n`
    } else {
        summary += `HTTP Request Duration: [NO DATA]\n\n`
    }

    if (metrics && metrics.http_reqs && metrics.http_reqs.values) {
        summary += `Total Requests: ${metrics.http_reqs.values.count}\n`
        summary += `Request Rate: ${metrics.http_reqs.values.rate.toFixed(2)}/s\n\n`
    }

    if (metrics && metrics.http_req_failed && metrics.http_req_failed.values) {
        const failRate = (metrics.http_req_failed.values.rate * 100).toFixed(2)
        summary += `Failure Rate: ${failRate}%\n`
    }

    return summary
}
