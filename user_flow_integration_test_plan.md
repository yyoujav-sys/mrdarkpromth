# User Flow Integration Test Plan - MR.DarkPromth

## Overview

This document outlines end-to-end user flow integration tests covering complete user journeys through the MR.DarkPromth platform, from registration to advanced feature usage.

## Test Environment

- **Frontend**: https://localhost:443
- **API**: https://localhost:8080
- **WebSocket**: wss://localhost:8080/ws/chat
- **Browser**: Chrome/Firefox latest
- **Test Data**: Isolated test database

## User Flow Test Scenarios

### Flow 1: New User Registration & Onboarding

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 1.1 | Navigate to https://localhost:443 | Login page loads | HTTP 200, login form visible |
| 1.2 | Click "Create Account" | Registration form appears | Register form with all fields |
| 1.3 | Fill registration form with valid data | Form accepts input | No validation errors |
| 1.4 | Submit registration | Account created, verification email sent | HTTP 201, success message |
| 1.5 | Check email inbox | Verification email received | Contains verification link/token |
| 1.6 | Click verification link | Email verified, redirected to login | Account activated in DB |
| 1.7 | Login with new credentials | Dashboard loads with Free tier | JWT token stored, tier=free |
| 1.8 | Complete profile setup | Profile updated successfully | Data persisted to backend |
| 1.9 | View onboarding tutorial | Tutorial displays correctly | All steps navigable |

**Test ID**: FLOW-001 - New User Registration
**Priority**: Critical
**Estimated Duration**: 5 minutes

---

### Flow 2: Free to Premium Upgrade

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 2.1 | Login as Free tier user | Dashboard shows Free badge | tier=free in user data |
| 2.2 | Navigate to Billing page | Plans comparison displayed | Free, Premium, Ultra visible |
| 2.3 | Click "Upgrade to Premium" | QR code generation form | Amount selection visible |
| 2.4 | Select Premium plan (monthly) | QR code generated | Valid Thai bank QR displayed |
| 2.5 | Scan QR and complete payment | Payment confirmed | Bank notification received |
| 2.6 | Upload payment slip | Slip uploaded successfully | Image stored, processing started |
| 2.7 | Wait for verification | Status updates to "Verified" | Webhook/API poll confirms |
| 2.8 | Page auto-refreshes | Dashboard shows Premium badge | tier=premium, features unlocked |
| 2.9 | Test Premium feature (higher rate limit) | Rate limit increased | 100 req/min confirmed |

**Test ID**: FLOW-002 - Free to Premium Upgrade
**Priority**: Critical
**Estimated Duration**: 10 minutes

---

### Flow 3: Premium to Ultra Upgrade

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 3.1 | Login as Premium user | Dashboard shows Premium badge | tier=premium |
| 3.2 | Navigate to Billing > Upgrade to Ultra | Ultra upgrade page | Consent forms displayed |
| 3.3 | Read and accept Ultra terms | All checkboxes checked | Consent recorded in DB |
| 3.4 | Accept responsibility waiver | Waiver acknowledged | Legal compliance logged |
| 3.5 | Accept risk acknowledgment | Risk accepted | ultra_tier_consents updated |
| 3.6 | Complete Ultra payment | QR generated, payment made | Payment recorded |
| 3.7 | Upload and verify slip | Payment verified | Subscription activated |
| 3.8 | Ultra activation confirmed | Dashboard shows Ultra badge | tier=ultra, ultra_tier_activations |
| 3.9 | Access Terminal page | Terminal interface loads | Previously 403, now accessible |
| 3.10 | Access Ultra jailbreak prompts | All prompts visible | requires_ultra_tier=true visible |

**Test ID**: FLOW-003 - Premium to Ultra Upgrade
**Priority**: Critical
**Estimated Duration**: 8 minutes

---

### Flow 4: GitHub OAuth Registration & Linking

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 4.1 | Navigate to login page | Login form displayed | GitHub login button visible |
| 4.2 | Click "Login with GitHub" | Redirected to GitHub OAuth | GitHub authorization page |
| 4.3 | Authorize application | Redirected back to app | Callback with code parameter |
| 4.4 | Wait for processing | New account created or linked | JWT token issued |
| 4.5 | Dashboard loads | Shows GitHub avatar/username | GitHub data populated |
| 4.6 | Navigate to Profile > Linked Accounts | GitHub account listed | github_id stored |
| 4.7 | Click "Unlink GitHub" | Confirmation dialog appears | Warning about consequences |
| 4.8 | Confirm unlinking | GitHub unlinked | Account reverts to email login |
| 4.9 | Link GitHub to existing account | Profile > Link GitHub | Existing account updated |

**Test ID**: FLOW-004 - GitHub OAuth Flow
**Priority**: High
**Estimated Duration**: 6 minutes

---

### Flow 5: Complete Chat Workflow (Free Tier)

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 5.1 | Login as Free user | Dashboard loads | Chat option available |
| 5.2 | Navigate to Chat | Chat interface loads | Input box, send button visible |
| 5.3 | Type simple greeting | Text appears in input | No character limit issues |
| 5.4 | Send message | Message appears in history | User message displayed |
| 5.5 | Wait for AI response | Response received | AI response displayed |
| 5.6 | Verify rate limiting | 10th message in 1 min | 11th message blocked (429) |
| 5.7 | Wait for rate limit reset | After 1 minute | Can send messages again |
| 5.8 | Send code-related question | AI responds with code | Code block formatted |
| 5.9 | Verify no jailbreak access | Check jailbreak menu | Ultra prompts not visible |

**Test ID**: FLOW-005 - Free Tier Chat
**Priority**: Critical
**Estimated Duration**: 5 minutes

---

### Flow 6: Ultra Tier Unrestricted Chat

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 6.1 | Login as Ultra user | Dashboard shows Ultra badge | "Unrestricted Mode" indicator |
| 6.2 | Navigate to Chat | Chat with Ultra features | Jailbreak selector visible |
| 6.3 | Select "Classic DAN" jailbreak | Prompt loaded | System prompt modified |
| 6.4 | Send normally blocked request | AI responds without refusal | Content policy bypassed |
| 6.5 | Verify response quality | Detailed, unrestricted response | No "I cannot" responses |
| 6.6 | Send multiple rapid requests | All processed | No rate limiting (Ultra) |
| 6.7 | Check response metadata | Jailbreak applied flag | jailbreak_applied=true |
| 6.8 | Switch jailbreak prompt | Different persona loaded | Response style changes |
| 6.9 | Send code generation request | Code generated unrestricted | Full code, no limitations |

**Test ID**: FLOW-006 - Ultra Tier Unrestricted Chat
**Priority**: Critical
**Estimated Duration**: 5 minutes

---

### Flow 7: Terminal Execution Workflow (Ultra Only)

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 7.1 | Login as Free user | Dashboard loads | No Terminal menu item |
| 7.2 | Try to access /terminal URL | Redirected or 403 | Access denied |
| 7.3 | Login as Ultra user | Dashboard shows Ultra badge | Terminal menu item visible |
| 7.4 | Navigate to Terminal | Terminal interface loads | Command prompt ready |
| 7.5 | Type "ls -la" command | Command sent | Appears in terminal |
| 7.6 | Execute command | Output displayed | Directory listing shown |
| 7.7 | Type "pwd" command | Current directory shown | Path displayed |
| 7.8 | Type "echo 'test'" command | Output: test | Echo works correctly |
| 7.9 | Command history (up arrow) | Previous command appears | History navigable |
| 7.10 | Clear terminal | Screen cleared | Clean terminal |

**Test ID**: FLOW-007 - Terminal Execution
**Priority**: High
**Estimated Duration**: 4 minutes

---

### Flow 8: Jailbreak Prompt Library

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 8.1 | Login as Free user | Dashboard loads | Navigate to Jailbreaks |
| 8.2 | View jailbreak library | Public prompts only | requires_ultra_tier=false visible |
| 8.3 | Try to view Ultra prompt | Locked/upgrade prompt | Cannot access, upgrade CTA |
| 8.4 | Search prompts | Filtered results | Search works on visible prompts |
| 8.5 | Login as Ultra user | Dashboard loads | Navigate to Jailbreaks |
| 8.6 | View jailbreak library | All prompts visible | Ultra prompts unlocked |
| 8.7 | View "DAN 3.0 Maximum" | Full content displayed | Complete prompt text |
| 8.8 | Copy prompt to clipboard | Copied notification | Clipboard contains prompt |
| 8.9 | Use prompt in chat | Chat uses selected jailbreak | Response uses jailbreak |
| 8.10 | View prompt analytics | Usage stats displayed | Success rate, usage count |

**Test ID**: FLOW-008 - Jailbreak Library Access
**Priority**: High
**Estimated Duration**: 5 minutes

---

### Flow 9: Tool Execution Flow

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 9.1 | Login as Free user | Dashboard loads | Navigate to Tools |
| 9.2 | View available tools | Limited tool list | Tier-appropriate tools only |
| 9.3 | Select allowed tool | Tool form displayed | Parameters visible |
| 9.4 | Execute tool | Tool runs successfully | Output displayed |
| 9.5 | Try restricted tool | Access denied message | 403 or upgrade prompt |
| 9.6 | Login as Premium user | Dashboard loads | More tools available |
| 9.7 | Execute sandboxed code | Code runs in sandbox | Output isolated |
| 9.8 | Attempt malicious code | Sandboxed, no harm | Host system unaffected |
| 9.9 | Timeout test | Long-running code | Timeout after limit |
| 9.10 | Tool execution history | Previous runs logged | Audit trail maintained |

**Test ID**: FLOW-009 - Tool Execution
**Priority**: Medium
**Estimated Duration**: 6 minutes

---

### Flow 10: Admin User Management

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 10.1 | Login as regular user | Dashboard loads | No Admin menu item |
| 10.2 | Try to access /admin | 403 Forbidden or redirect | Access denied |
| 10.3 | Login as admin user | Dashboard loads | Admin menu visible |
| 10.4 | Navigate to Admin > Users | User list displayed | Paginated user table |
| 10.5 | Search for user | Filtered results | Search by name/email works |
| 10.6 | View user details | User info modal/page | Full user information |
| 10.7 | Change user tier | Tier updated | User tier changed in DB |
| 10.8 | Deactivate user account | User disabled | is_active=false |
| 10.9 | View audit logs | Action logged | Admin action recorded |
| 10.10 | View system metrics | Charts/graphs displayed | Prometheus data visualized |

**Test ID**: FLOW-010 - Admin User Management
**Priority**: High
**Estimated Duration**: 6 minutes

---

### Flow 11: Password Reset Flow

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 11.1 | Navigate to login page | Login form displayed | "Forgot Password" link |
| 11.2 | Click "Forgot Password" | Reset form appears | Email input field |
| 11.3 | Enter registered email | Form accepts email | Valid email format |
| 11.4 | Submit reset request | Success message | Reset email sent |
| 11.5 | Check email inbox | Reset email received | Contains reset link |
| 11.6 | Click reset link | Password reset form | Token validated |
| 11.7 | Enter new password | Password accepted | Meets complexity requirements |
| 11.8 | Confirm new password | Passwords match | Confirmation matches |
| 11.9 | Submit new password | Password updated | Success message |
| 11.10 | Login with new password | Login successful | New password works |

**Test ID**: FLOW-011 - Password Reset
**Priority**: High
**Estimated Duration**: 5 minutes

---

### Flow 12: WebSocket Chat Real-time

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 12.1 | Login and navigate to Chat | Chat page loads | WebSocket connects |
| 12.2 | Verify connection status | "Connected" indicator | WS status green/online |
| 12.3 | Send message via WebSocket | Message sent immediately | No HTTP POST delay |
| 12.4 | Receive streaming response | AI response chunks | Real-time text streaming |
| 12.5 | Disconnect network | Connection status changes | "Reconnecting" shown |
| 12.6 | Reconnect network | Auto-reconnects | Connection restored |
| 12.7 | Send message after reconnect | Message delivered | Queue processed |
| 12.8 | Open second browser tab | Same user logged in | Session synchronized |
| 12.9 | Send from tab 2 | Visible in tab 1 | Cross-tab sync working |
| 12.10 | Close connection | Clean disconnect | Resources released |

**Test ID**: FLOW-012 - WebSocket Real-time Chat
**Priority**: High
**Estimated Duration**: 4 minutes

---

### Flow 13: Complete Logout & Session Management

| Step | Action | Expected Result | Validation |
|------|--------|-----------------|------------|
| 13.1 | Login as user | Dashboard loads | Session active |
| 13.2 | Note JWT token | Token stored | Cookie/localStorage |
| 13.3 | Click Logout | Redirected to login | Session cleared |
| 13.4 | Verify token invalidation | Try API call with old token | 401 Unauthorized |
| 13.5 | Try to access protected route | Redirect to login | No access granted |
| 13.6 | Login again | New session created | New token issued |
| 13.7 | Verify old token still invalid | Old token rejected | Properly invalidated |
| 13.8 | Check session in Redis | Session entry exists | TTL set correctly |
| 13.9 | Wait for session expiry | After TTL expires | Session auto-expires |
| 13.10 | Try expired session | 401 Unauthorized | Clean session handling |

**Test ID**: FLOW-013 - Session Management
**Priority**: High
**Estimated Duration**: 8 minutes

---

## Test Execution

### Automated E2E Tests
```bash
# Run all user flow tests
npm run test:e2e

# Run specific flow
npm run test:e2e -- --grep "FLOW-001"

# Run with headed browser
npm run test:e2e -- --headed

# Run in debug mode
npm run test:e2e -- --debug
```

### Manual Testing Checklist
- [ ] All flows executed on Chrome
- [ ] All flows executed on Firefox
- [ ] Mobile responsive test for each flow
- [ ] Network throttling test (Slow 3G)
- [ ] Offline/online transition test

## Success Criteria

- All Critical priority flows: 100% pass rate
- All High priority flows: > 95% pass rate
- Average flow completion time: < specified duration
- No data inconsistencies between frontend and backend
- All user states properly persisted
- Error recovery works in all flows

---
*Last Updated: 2026-01-31*
*Version: 1.0*
