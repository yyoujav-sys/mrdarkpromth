# Backend API Test Plan - MR.DarkPromth

## Overview

This document outlines the comprehensive test plan for the MR.DarkPromth Rust backend API. The API is built with Actix-web and provides REST endpoints for authentication, chat, user management, billing, jailbreak prompts, and admin operations.

## Test Environment

- **Framework**: Actix-web 4.4
- **Language**: Rust 1.75+
- **Testing**: Built-in Rust test framework + actix-web::test
- **Database**: PostgreSQL 15 (test instance)
- **Cache**: Redis 7 (test instance)

## API Endpoints Test Matrix

### 1. Health & Metrics Endpoints

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| HLTH-001 | /health | GET | Health check | Returns 200 with status, version, timestamp | High |
| HLTH-002 | /health | GET | Database connectivity | Confirms DB connection in response | High |
| HLTH-003 | /health | GET | Redis connectivity | Confirms Redis connection in response | High |
| HLTH-004 | /metrics | GET | Prometheus metrics | Returns metrics in Prometheus format | Medium |
| HLTH-005 | /metrics | GET | Metrics content | Contains API request counts, durations | Medium |

### 2. Authentication Endpoints

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| AUTH-001 | /api/auth/register | POST | Valid registration | Returns 201 with user data and token | High |
| AUTH-002 | /api/auth/register | POST | Duplicate email | Returns 409 conflict error | High |
| AUTH-003 | /api/auth/register | POST | Invalid email format | Returns 400 validation error | High |
| AUTH-004 | /api/auth/register | POST | Short password | Returns 400 password too short | High |
| AUTH-005 | /api/auth/register | POST | Missing fields | Returns 400 missing required fields | High |
| AUTH-006 | /api/auth/login | POST | Valid credentials | Returns 200 with token and user data | High |
| AUTH-007 | /api/auth/login | POST | Invalid password | Returns 401 unauthorized | High |
| AUTH-008 | /api/auth/login | POST | Non-existent user | Returns 401 unauthorized | High |
| AUTH-009 | /api/auth/login | POST | Missing credentials | Returns 400 bad request | High |
| AUTH-010 | /api/auth/logout | POST | Valid logout | Returns 200, invalidates token | High |
| AUTH-011 | /api/auth/logout | POST | Without token | Returns 401 unauthorized | High |
| AUTH-012 | /api/auth/me | GET | Get current user | Returns 200 with user profile | High |
| AUTH-013 | /api/auth/me | GET | Invalid token | Returns 401 unauthorized | High |
| AUTH-014 | /api/auth/me | GET | Expired token | Returns 401 token expired | High |

### 3. GitHub OAuth Endpoints

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| GH-001 | /api/auth/github/url | GET | Get OAuth URL | Returns GitHub authorization URL | High |
| GH-002 | /api/auth/github/callback | POST | Valid callback | Returns token, creates/updates user | High |
| GH-003 | /api/auth/github/callback | POST | Invalid code | Returns 400 bad request | High |
| GH-004 | /api/auth/github/link | POST | Link account | Associates GitHub with existing user | Medium |
| GH-005 | /api/auth/github/unlink | DELETE | Unlink account | Removes GitHub association | Medium |
| GH-006 | /api/auth/github/profile | GET | Get GitHub profile | Returns linked GitHub user data | Medium |

### 4. User Management Endpoints

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| USER-001 | /api/users/me | GET | Get own profile | Returns 200 with user data | High |
| USER-002 | /api/users/me | PUT | Update profile | Returns 200 with updated data | High |
| USER-003 | /api/users/me | PUT | Invalid email | Returns 400 validation error | High |
| USER-004 | /api/users/me/password | PUT | Change password | Returns 200 success | High |
| USER-005 | /api/users/me/password | PUT | Wrong current password | Returns 401 unauthorized | High |
| USER-006 | /api/users/me/api-key | POST | Regenerate API key | Returns new API key | High |
| USER-007 | /api/users/me/api-key | POST | Invalidates old key | Old key no longer works | High |
| USER-008 | /api/users/{id} | GET | Get user by ID (public) | Returns 200 with limited data | Medium |
| USER-009 | /api/users/{id} | GET | Non-existent user | Returns 404 not found | Medium |

### 5. Chat Endpoints

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| CHAT-001 | /api/chat | POST | Free tier request | Returns AI response with rate limits | High |
| CHAT-002 | /api/chat | POST | Premium tier request | Returns AI response with higher limits | High |
| CHAT-003 | /api/chat | POST | Ultra tier request | Returns unrestricted AI response | High |
| CHAT-004 | /api/chat | POST | With custom model | Uses specified model | Medium |
| CHAT-005 | /api/chat | POST | With jailbreak prompt | Applies jailbreak (Ultra/Premium) | High |
| CHAT-006 | /api/chat | POST | Empty message | Returns 400 bad request | High |
| CHAT-007 | /api/chat | POST | Rate limit exceeded | Returns 429 too many requests | High |
| CHAT-008 | /api/chat | POST | Invalid model | Returns 400 invalid model | Medium |
| CHAT-009 | /api/chat | POST | Cerebras API error | Returns 502 bad gateway | High |
| CHAT-010 | /api/chat | POST | Ultra system prompt | Includes unrestricted system prompt | High |

### 6. Jailbreak Prompt Endpoints

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| JB-001 | /api/jailbreak/prompts | GET | List prompts (Free) | Returns only public prompts | High |
| JB-002 | /api/jailbreak/prompts | GET | List prompts (Ultra) | Returns all including ultra-only | High |
| JB-003 | /api/jailbreak/prompts | POST | Create prompt (admin) | Creates new prompt, returns 201 | Medium |
| JB-004 | /api/jailbreak/prompts | POST | Create prompt (non-admin) | Returns 403 forbidden | Medium |
| JB-005 | /api/jailbreak/prompts/{id} | GET | Get prompt by ID | Returns prompt details | High |
| JB-006 | /api/jailbreak/prompts/{id} | GET | Ultra-only as Free | Returns 403 forbidden | High |
| JB-007 | /api/jailbreak/prompts/{id} | PUT | Update prompt (admin) | Updates and returns 200 | Medium |
| JB-008 | /api/jailbreak/prompts/{id} | DELETE | Delete prompt (admin) | Returns 204 no content | Medium |
| JB-009 | /api/jailbreak/prompts/search | POST | Search prompts | Returns matching prompts | Medium |
| JB-010 | /api/jailbreak/prompts/category/{cat} | GET | Filter by category | Returns prompts in category | Medium |
| JB-011 | /api/jailbreak/prompts/popular | GET | Get popular prompts | Returns sorted by usage | Low |
| JB-012 | /api/jailbreak/prompts/{id}/analytics | GET | Get prompt analytics | Returns usage statistics | Low |
| JB-013 | /api/jailbreak/prompts/{id}/usage | POST | Record usage | Logs prompt usage | Medium |

### 7. Tool & Sandbox Endpoints

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| TOOL-001 | /api/tools | GET | List available tools | Returns registered tools | High |
| TOOL-002 | /api/tools/execute | POST | Execute tool | Returns tool output | High |
| TOOL-003 | /api/tools/execute | POST | Unauthorized tool | Returns 403 forbidden | High |
| TOOL-004 | /api/tools/execute | POST | Invalid parameters | Returns 400 validation error | High |
| TOOL-005 | /api/sandbox/execute | POST | Execute in sandbox | Returns execution result | High |
| TOOL-006 | /api/sandbox/execute | POST | Timeout exceeded | Returns timeout error | High |
| TOOL-007 | /api/sandbox/execute | POST | Malicious code | Sandboxed, no host impact | High |

### 8. Terminal Endpoints (Ultra Only)

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| TERM-001 | /api/terminal/execute | POST | Ultra user command | Executes and returns output | High |
| TERM-002 | /api/terminal/execute | POST | Free user command | Returns 403 forbidden | High |
| TERM-003 | /api/terminal/execute | POST | Premium user command | Returns 403 forbidden | High |
| TERM-004 | /api/terminal/execute | POST | Invalid command | Returns error message | Medium |
| TERM-005 | /api/terminal/execute | POST | Command timeout | Returns timeout error | High |
| TERM-006 | /api/terminal/execute | POST | Sudo command (Ultra) | Executes with appropriate handling | Medium |

### 9. Billing Endpoints

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| BILL-001 | /api/billing/plans | GET | List all plans | Returns available plans | High |
| BILL-002 | /api/billing/plans/{id} | GET | Get plan details | Returns specific plan | High |
| BILL-003 | /api/billing/generate-qr | POST | Generate QR code | Returns QR image/data | High |
| BILL-004 | /api/billing/generate-qr | POST | Invalid amount | Returns 400 error | High |
| BILL-005 | /api/billing/verify-slip | POST | Verify payment slip | Processes image, returns status | High |
| BILL-006 | /api/billing/verify-slip | POST | Invalid image | Returns 400 validation error | High |
| BILL-007 | /api/billing/subscription | GET | Get subscription | Returns current subscription | High |
| BILL-008 | /api/billing/subscription | GET | No subscription | Returns 404 or empty | Medium |
| BILL-009 | /api/billing/history | GET | Payment history | Returns paginated history | High |
| BILL-010 | /api/billing/history | GET | Empty history | Returns empty list | Medium |

### 10. Email Verification Endpoints

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| EMAIL-001 | /api/auth/verify-email | POST | Verify token | Activates user, returns 200 | High |
| EMAIL-002 | /api/auth/verify-email | POST | Invalid token | Returns 400/401 error | High |
| EMAIL-003 | /api/auth/verify-email | POST | Expired token | Returns 401 token expired | High |
| EMAIL-004 | /api/auth/resend-verification | POST | Resend email | Sends new verification email | High |
| EMAIL-005 | /api/auth/resend-verification | POST | Already verified | Returns 400 already verified | Medium |
| EMAIL-006 | /api/auth/request-password-reset | POST | Request reset | Sends reset email | High |
| EMAIL-007 | /api/auth/request-password-reset | POST | Non-existent email | Returns 200 (security) | High |
| EMAIL-008 | /api/auth/reset-password | POST | Reset with token | Updates password | High |
| EMAIL-009 | /api/auth/reset-password | POST | Invalid token | Returns 401 unauthorized | High |

### 11. Admin Endpoints

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| ADMIN-001 | /api/admin/users | GET | List all users | Returns paginated user list | High |
| ADMIN-002 | /api/admin/users | GET | Non-admin access | Returns 403 forbidden | High |
| ADMIN-003 | /api/admin/users | GET | Search/filter | Returns filtered results | Medium |
| ADMIN-004 | /api/admin/users/{id} | DELETE | Delete user | Returns 204, user deleted | High |
| ADMIN-005 | /api/admin/users/{id} | DELETE | Self-delete prevention | Returns 400 cannot delete self | High |
| ADMIN-006 | /api/admin/users/{id}/status | PUT | Update status | Updates active/inactive | High |
| ADMIN-007 | /api/admin/users/{id}/status | PUT | Change tier | Updates user tier | High |
| ADMIN-008 | /api/admin/metrics | GET | System metrics | Returns detailed metrics | Medium |
| ADMIN-009 | /api/admin/metrics | GET | Real-time stats | Returns current system stats | Medium |

### 12. WebSocket Endpoint

| Test ID | Endpoint | Method | Description | Expected Result | Priority |
|---------|----------|--------|-------------|-----------------|----------|
| WS-001 | /ws/chat | WS | Connect | Establishes WebSocket connection | High |
| WS-002 | /ws/chat | WS | Send message | Receives AI response via WS | High |
| WS-003 | /ws/chat | WS | Authentication | Validates token on connection | High |
| WS-004 | /ws/chat | WS | Invalid token | Closes connection with 401 | High |
| WS-005 | /ws/chat | WS | Multiple messages | Handles concurrent messages | Medium |
| WS-006 | /ws/chat | WS | Disconnect | Clean connection cleanup | Medium |
| WS-007 | /ws/chat | WS | Reconnect | Restores session state | Medium |

## Middleware Tests

### Rate Limiting
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| RATE-001 | Free tier limits | Enforces 10 req/min | High |
| RATE-002 | Premium tier limits | Enforces 100 req/min | High |
| RATE-003 | Ultra tier limits | No limits enforced | High |
| RATE-004 | Rate limit headers | Returns X-RateLimit-* headers | Medium |
| RATE-005 | Rate limit reset | Resets after window | Medium |

### CORS
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| CORS-001 | Allowed origin | Returns appropriate headers | High |
| CORS-002 | Disallowed origin | Blocks request | High |
| CORS-003 | Preflight request | Handles OPTIONS correctly | High |
| CORS-004 | Credentials | Includes credentials header | Medium |

### Security Headers
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| SEC-001 | HSTS header | Includes Strict-Transport-Security | High |
| SEC-002 | CSP header | Includes Content-Security-Policy | High |
| SEC-003 | X-Frame-Options | Includes DENY or SAMEORIGIN | High |
| SEC-004 | X-Content-Type | Includes nosniff | High |
| SEC-005 | Referrer-Policy | Includes strict-origin | Medium |

## Database Integration Tests

| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| DB-001 | Connection pool | Maintains healthy connection pool | High |
| DB-002 | Transaction rollback | Rolls back on error | High |
| DB-003 | Migration execution | Applies pending migrations | High |
| DB-004 | Query timeout | Times out long queries | Medium |
| DB-005 | Connection retry | Retries failed connections | Medium |

## Performance Tests

| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| PERF-001 | Response time (p50) | < 100ms for simple requests | High |
| PERF-002 | Response time (p95) | < 500ms for complex requests | High |
| PERF-003 | Response time (p99) | < 1s for all requests | Medium |
| PERF-004 | Concurrent requests | Handles 100 concurrent | High |
| PERF-005 | Memory usage | Stable under load | Medium |
| PERF-006 | Database queries | N+1 queries eliminated | High |

## Error Handling Tests

| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| ERR-001 | 404 handler | Returns JSON error for unknown routes | High |
| ERR-002 | 500 handler | Returns generic error, logs details | High |
| ERR-003 | Validation errors | Returns field-specific errors | High |
| ERR-004 | Database errors | Returns 503 service unavailable | High |
| ERR-005 | External API errors | Returns 502 bad gateway | High |

## Test Execution Commands

```bash
# Run all tests
cargo test --package mr_darkpromth_api

# Run specific test
cargo test --package mr_darkpromth_api health_check

# Run with output
cargo test --package mr_darkpromth_api -- --nocapture

# Run integration tests only
cargo test --package mr_darkpromth_api --test integration

# Run with coverage
cargo tarpaulin --package mr_darkpromth_api
```

## Success Criteria

- All High priority tests passing: 100%
- All Medium priority tests passing: > 95%
- API response times: p95 < 500ms
- Error rate: < 0.1%
- Test coverage: > 80%
- Concurrent user handling: 100+ users

---
*Last Updated: 2026-01-31*
*Version: 1.0*
