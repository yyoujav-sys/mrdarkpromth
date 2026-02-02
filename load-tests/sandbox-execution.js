import http from 'k6/http'
import { check, sleep, fail } from 'k6'

const baseUrl = __ENV.LOAD_TEST_BASE_URL || 'http://localhost:8080'
const vus = Number(__ENV.LOAD_VUS || 5)
const duration = __ENV.LOAD_DURATION || '30s'
const sleepSeconds = Number(__ENV.LOAD_SLEEP || 1)

export const options = {
  vus,
  duration,
  thresholds: {
    http_req_failed: ['rate<0.05'],
    http_req_duration: ['p(95)<2000']
  }
}

function resolveToken() {
  const token = __ENV.SANDBOX_AUTH_TOKEN
  if (token) {
    return token
  }

  const email = __ENV.SANDBOX_AUTH_EMAIL
  const password = __ENV.SANDBOX_AUTH_PASSWORD
  if (!email || !password) {
    fail('Provide SANDBOX_AUTH_TOKEN or SANDBOX_AUTH_EMAIL/SANDBOX_AUTH_PASSWORD for sandbox load tests.')
  }

  const loginResponse = http.post(
    `${baseUrl}/api/auth/login`,
    JSON.stringify({ email, password }),
    { headers: { 'Content-Type': 'application/json' } }
  )

  check(loginResponse, { 'login succeeded': (res) => res.status === 200 })
  return loginResponse.json('token')
}

export function setup() {
  const token = resolveToken()
  if (!token) {
    fail('Failed to resolve auth token for sandbox load tests.')
  }
  return { token }
}

export default function (data) {
  const payload = JSON.stringify({
    code: 'print("sandbox load")',
    language: 'python',
    timeout_seconds: 5
  })

  const response = http.post(`${baseUrl}/api/sandbox/execute`, payload, {
    headers: {
      'Content-Type': 'application/json',
      Authorization: `Bearer ${data.token}`
    }
  })

  check(response, { 'sandbox responded': (res) => res.status === 200 })
  sleep(sleepSeconds)
}
