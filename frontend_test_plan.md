# Frontend Test Plan - MR.DarkPromth

## Overview

This document outlines the comprehensive test plan for the MR.DarkPromth React frontend. The frontend is built with React 19.2, TypeScript, Vite, Tailwind CSS v4, and Zustand for state management.

## Test Environment

- **Framework**: React 19.2 + TypeScript
- **Build Tool**: Vite 7.2
- **Testing Tools**: Vitest + React Testing Library
- **Browser Targets**: Chrome, Firefox, Safari, Edge (latest 2 versions)
- **Screen Sizes**: Desktop (1920x1080), Tablet (768x1024), Mobile (375x667)

## Test Categories

### 1. Component Tests

#### 1.1 Authentication Components
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| AUTH-001 | Login form renders correctly | All fields and buttons visible | High |
| AUTH-002 | Login with valid credentials | Redirects to dashboard, token stored | High |
| AUTH-003 | Login with invalid credentials | Shows error message, no redirect | High |
| AUTH-004 | GitHub OAuth button click | Redirects to GitHub auth URL | High |
| AUTH-005 | Registration form validation | Shows validation errors for invalid inputs | High |
| AUTH-006 | Password reset flow | Sends reset email, shows confirmation | Medium |
| AUTH-007 | Email verification page | Verifies token, shows success/error | Medium |
| AUTH-008 | Logout functionality | Clears token, redirects to login | High |

#### 1.2 Navigation Components
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| NAV-001 | Sidebar renders all menu items | All tier-appropriate items visible | High |
| NAV-002 | Navigation based on user tier | Ultra items only for Ultra users | High |
| NAV-003 | Active route highlighting | Current page highlighted in menu | Medium |
| NAV-004 | Mobile hamburger menu | Toggles sidebar on mobile | Medium |
| NAV-005 | Navigation state persistence | Remains open/closed after refresh | Low |

#### 1.3 Chat Interface
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| CHAT-001 | Message input renders | Textarea and send button visible | High |
| CHAT-002 | Send message | Message appears in chat history | High |
| CHAT-003 | Receive AI response | Response displayed with typing indicator | High |
| CHAT-004 | Markdown rendering | Code blocks, lists, formatting displayed | High |
| CHAT-005 | Syntax highlighting | Code blocks have language-specific colors | Medium |
| CHAT-006 | Chat history persistence | Messages remain after navigation | Medium |
| CHAT-007 | Clear chat functionality | Clears all messages | Low |
| CHAT-008 | Copy message content | Copies text to clipboard | Low |
| CHAT-009 | Ultra tier system prompt indicator | Shows "Unrestricted Mode" for Ultra | High |

#### 1.4 Dashboard Components
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| DASH-001 | Dashboard stats cards | Shows correct usage statistics | High |
| DASH-002 | Recent activity list | Displays recent actions | Medium |
| DASH-003 | Tier badge display | Shows current tier with correct color | High |
| DASH-004 | API key display | Shows masked API key with copy button | High |
| DASH-005 | Upgrade prompt for Free tier | Shows upgrade CTA for non-Ultra | Medium |

#### 1.5 Admin Components
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| ADMIN-001 | User management table | Lists all users with pagination | High |
| ADMIN-002 | User search/filter | Filters users by name/email | Medium |
| ADMIN-003 | Delete user confirmation | Shows modal before deletion | High |
| ADMIN-004 | Update user tier | Changes user tier successfully | High |
| ADMIN-005 | System metrics display | Shows Prometheus metrics | Medium |
| ADMIN-006 | Audit log viewer | Displays audit logs with filtering | Medium |

#### 1.6 Billing Components
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| BILL-001 | Plans comparison table | Shows all plans with features | High |
| BILL-002 | QR code generation | Generates Thai bank QR code | High |
| BILL-003 | Payment slip upload | Accepts and previews image | High |
| BILL-004 | Payment verification | Shows verification status | High |
| BILL-005 | Subscription status | Shows current subscription details | High |
| BILL-006 | Payment history table | Lists past payments | Medium |

#### 1.7 Profile Components
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| PROF-001 | Profile form rendering | Shows current user data | High |
| PROF-002 | Profile update | Saves changes to backend | High |
| PROF-003 | Password change | Validates and updates password | High |
| PROF-004 | API key regeneration | Generates new key, invalidates old | High |
| PROF-005 | Avatar upload | Uploads and displays new avatar | Low |
| PROF-006 | Email preferences | Saves notification preferences | Low |

#### 1.8 Terminal Component (Ultra Only)
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| TERM-001 | Terminal rendering | Shows terminal interface for Ultra | High |
| TERM-002 | Command input | Accepts and sends command | High |
| TERM-003 | Command output display | Shows command output | High |
| TERM-004 | Command history | Navigable with up/down arrows | Medium |
| TERM-005 | Access denied for non-Ultra | Shows upgrade prompt | High |

#### 1.9 Jailbreak Prompt Browser
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| JB-001 | Prompt list rendering | Shows available prompts | High |
| JB-002 | Category filtering | Filters by category | Medium |
| JB-003 | Search functionality | Searches prompt titles/content | Medium |
| JB-004 | Ultra-only prompts | Locked for non-Ultra users | High |
| JB-005 | Prompt detail view | Shows full prompt content | Medium |
| JB-006 | Copy prompt | Copies to clipboard | Low |

### 2. Page Tests

#### 2.1 Page Routing
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| ROUTE-001 | Public routes accessibility | Login, register accessible without auth | High |
| ROUTE-002 | Protected routes redirect | Redirects to login when unauthenticated | High |
| ROUTE-003 | Ultra-only routes | Redirects non-Ultra from terminal | High |
| ROUTE-004 | Admin routes | Only accessible by admin users | High |
| ROUTE-005 | 404 page | Shows custom 404 for unknown routes | Medium |

#### 2.2 Page Load Performance
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| PERF-001 | Initial load time | < 3 seconds on 3G | High |
| PERF-002 | Time to Interactive | < 5 seconds | High |
| PERF-003 | Bundle size | Main bundle < 500KB gzipped | Medium |
| PERF-004 | Code splitting | Routes load on demand | Medium |

### 3. Integration Tests

#### 3.1 API Integration
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| API-001 | Successful API call | Displays data correctly | High |
| API-002 | API error handling | Shows user-friendly error | High |
| API-003 | Loading states | Shows spinner/skeleton during load | High |
| API-004 | Token refresh | Refreshes expired tokens automatically | High |
| API-005 | Request cancellation | Cancels pending requests on unmount | Medium |

#### 3.2 State Management
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| STATE-001 | Zustand store initialization | Loads initial state correctly | High |
| STATE-002 | State persistence | Persists auth state across reloads | High |
| STATE-003 | State updates | Components re-render on state change | High |
| STATE-004 | Multiple store slices | Independent slices work correctly | Medium |

#### 3.3 WebSocket Integration
| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| WS-001 | Connection establishment | Connects on chat page load | High |
| WS-002 | Message sending via WS | Sends and receives messages | High |
| WS-003 | Reconnection | Reconnects on connection drop | High |
| WS-004 | Connection status indicator | Shows online/offline status | Medium |

### 4. Responsive Design Tests

| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| RESP-001 | Desktop layout | Full sidebar, proper spacing | High |
| RESP-002 | Tablet layout | Collapsible sidebar, adapted grid | High |
| RESP-003 | Mobile layout | Hamburger menu, stacked layout | High |
| RESP-004 | Touch targets | Minimum 44x44px on mobile | Medium |
| RESP-005 | Font scaling | Readable at 200% zoom | Medium |

### 5. Accessibility Tests

| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| A11Y-001 | Keyboard navigation | All interactive elements focusable | High |
| A11Y-002 | Screen reader labels | Proper ARIA labels on all elements | High |
| A11Y-003 | Color contrast | WCAG AA compliance (4.5:1) | High |
| A11Y-004 | Focus indicators | Visible focus rings on all elements | Medium |
| A11Y-005 | Form labels | All inputs have associated labels | High |
| A11Y-006 | Error announcements | Screen readers announce errors | Medium |

### 6. Security Tests

| Test ID | Description | Expected Result | Priority |
|---------|-------------|-----------------|----------|
| SEC-001 | XSS prevention | Scripts in input not executed | High |
| SEC-002 | Token storage | JWT in httpOnly cookie | High |
| SEC-003 | CSRF protection | CSRF tokens on state-changing requests | High |
| SEC-004 | Input sanitization | HTML in inputs escaped | High |
| SEC-005 | Secure headers | HSTS, CSP headers present | Medium |

## Test Execution Plan

### Phase 1: Critical Path (High Priority)
1. Authentication flow (AUTH-001 to AUTH-008)
2. Chat functionality (CHAT-001 to CHAT-003)
3. Route protection (ROUTE-001 to ROUTE-004)
4. API integration (API-001 to API-004)

### Phase 2: Core Features (Medium Priority)
1. Dashboard components (DASH-001 to DASH-005)
2. Profile management (PROF-001 to PROF-004)
3. Navigation (NAV-001 to NAV-005)
4. Responsive design (RESP-001 to RESP-005)

### Phase 3: Advanced Features (Lower Priority)
1. Admin components (ADMIN-001 to ADMIN-006)
2. Terminal (TERM-001 to TERM-005)
3. Jailbreak browser (JB-001 to JB-006)
4. Billing (BILL-001 to BILL-006)

### Phase 4: Polish
1. Accessibility (A11Y-001 to A11Y-006)
2. Performance (PERF-001 to PERF-004)
3. Security (SEC-001 to SEC-005)

## Test Commands

```bash
# Run all tests
cd frontend && npm test

# Run with coverage
cd frontend && npm test -- --coverage

# Run specific test file
cd frontend && npm test -- Chat.test.tsx

# Run in watch mode
cd frontend && npm test -- --watch
```

## Success Criteria

- All High priority tests passing: 100%
- All Medium priority tests passing: > 90%
- Overall test coverage: > 80%
- No critical accessibility violations
- Lighthouse score > 90 (Performance, Accessibility, Best Practices, SEO)

---
*Last Updated: 2026-01-31*
*Version: 1.0*
