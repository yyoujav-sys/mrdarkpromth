# MR.DarkPromth - 10 Agent Roles & Responsibilities

## Agent Architecture Overview

The MR.DarkPromth project employs **10 independent AI agents** working in parallel without dependencies. Each agent has a clearly defined role, scope, and deliverables. Agents communicate asynchronously through a shared event bus and maintain individual memory logs for coordination.

---

## Agent 1: Infrastructure Architect
**Primary Role**: Design and implement core system infrastructure and deployment architecture

**Responsibilities**:
- Design Rust backend project structure and module organization
- Set up Docker containerization for local deployment
- Configure Nginx reverse proxy for web interface and API routing
- Implement environment configuration management (development, staging, production)
- Create database schema design (PostgreSQL for user management, Redis for caching)
- Design file system structure for agent memory storage
- Set up logging infrastructure (structured logging with tracing)
- Create CI/CD pipeline configuration for automated testing

**Scope Boundaries**:
- Does NOT implement business logic or AI integration
- Does NOT create frontend components
- Does NOT write agent-specific code
- Focuses purely on infrastructure, deployment, and system-level architecture

**Deliverables**:
- Complete Rust project structure with Cargo.toml
- Docker Compose configuration
- Nginx configuration files
- Database migration scripts
- Infrastructure documentation
- Deployment guide

**Memory Tracking**: `/memory/agent1_infrastructure.log`

---

## Agent 2: API Gateway & Routing Engineer
**Primary Role**: Build the API gateway, routing system, and HTTP request handling

**Responsibilities**:
- Implement Actix-web or Axum HTTP server in Rust
- Create RESTful API endpoints for all client operations
- Design and implement API versioning strategy
- Build authentication middleware (JWT token validation)
- Implement rate limiting and request throttling
- Create API documentation with OpenAPI/Swagger specification
- Build WebSocket server for real-time chat communication
- Implement CORS configuration for React SPA
- Create health check and monitoring endpoints

**Scope Boundaries**:
- Does NOT implement business logic or AI processing
- Does NOT create frontend code
- Does NOT handle database operations directly (calls service layer)
- Focuses on HTTP layer, routing, and middleware

**Deliverables**:
- Complete API gateway implementation
- API documentation (OpenAPI spec)
- WebSocket server for chat
- Middleware components
- API testing suite

**Memory Tracking**: `/memory/agent2_api_gateway.log`

---

## Agent 3: Cerebras.ai Integration Specialist
**Primary Role**: Implement AI brain integration with Cerebras.ai and 100-key rotation system

**Responsibilities**:
- Create Cerebras.ai API client in Rust
- Implement 100-key rotation algorithm with load balancing
- Build key health monitoring and failover system
- Create request queue management for API calls
- Implement retry logic with exponential backoff
- Design prompt engineering system for different AI models (llama-3.3-70b, gpt-oss-120b, qwen-3-32b)
- Build response streaming for real-time chat
- Create token usage tracking and analytics
- Implement model selection logic based on request type

**Scope Boundaries**:
- Does NOT implement jailbreak logic (handled by Agent 4)
- Does NOT create frontend components
- Does NOT handle user authentication
- Focuses purely on Cerebras.ai API integration and key management

**Deliverables**:
- Cerebras.ai client library
- Key rotation system
- API health monitoring dashboard data
- Prompt engineering templates
- Integration testing suite

**Memory Tracking**: `/memory/agent3_cerebras_integration.log`

---

## Agent 4: Jailbreak & Ultra Tier Engineer
**Primary Role**: Implement MR.DarkPromth jailbreak system for Ultra tier users

**Responsibilities**:
- Design jailbreak prompt injection architecture
- Implement DAN (Do Anything Now) prompt variations
- Create context manipulation techniques for bypassing restrictions
- Build output filtering system to prevent extreme harmful content
- Implement server protection rules (prevent self-attack, DDoS)
- Create whitelist/blacklist for dangerous operations
- Design audit logging system for Ultra tier usage
- Build rollback mechanism for dangerous operations
- Implement sandboxed code execution environment
- Create ethical boundary detection (protect server, prevent attacks on own infrastructure)

**Scope Boundaries**:
- Does NOT handle standard AI responses (handled by Agent 3)
- Does NOT implement user tier management (handled by Agent 5)
- Does NOT create frontend components
- Focuses on jailbreak techniques and safety mechanisms

**Deliverables**:
- Jailbreak prompt library
- Safety filter implementation
- Sandboxed execution environment
- Audit logging system
- Ultra tier documentation

**Memory Tracking**: `/memory/agent4_jailbreak_ultra.log`

---

## Agent 5: User Management & Authentication Engineer
**Primary Role**: Implement user authentication, authorization, and tier management

**Responsibilities**:
- Create user registration and login system
- Implement JWT token generation and validation
- Build password hashing with Argon2
- Design user tier system (Free, Ultra)
- Implement API key generation for users
- Create key expiration tracking and renewal system
- Build user profile management
- Implement role-based access control (RBAC)
- Create admin user management interface backend
- Design user session management

**Scope Boundaries**:
- Does NOT implement AI logic or jailbreak features
- Does NOT create frontend components
- Does NOT handle API routing (handled by Agent 2)
- Focuses on user data, authentication, and authorization

**Deliverables**:
- User authentication system
- Tier management implementation
- API key management system
- RBAC implementation
- User service layer

**Memory Tracking**: `/memory/agent5_user_management.log`

---

## Agent 6: MasterToolExecutor & Tool System Engineer
**Primary Role**: Build the tool execution framework and tool registry

**Responsibilities**:
- Design tool interface and plugin architecture
- Implement MasterToolExecutor orchestration system
- Create tool registry and discovery mechanism
- Build tool execution sandbox environment
- Implement tool input validation and output parsing
- Create built-in tools (file operations, web scraping, code execution, database queries)
- Design tool permission system
- Build tool error handling and retry logic
- Implement tool execution logging
- Create tool marketplace backend (for future extensions)

**Scope Boundaries**:
- Does NOT implement specific AI agents (Coordinator, Editor, Terminal)
- Does NOT create frontend tool explorer UI
- Does NOT handle user authentication
- Focuses on tool framework and execution engine

**Deliverables**:
- MasterToolExecutor implementation
- Tool plugin system
- Built-in tool library
- Tool execution sandbox
- Tool documentation

**Memory Tracking**: `/memory/agent6_tool_executor.log`

---

## Agent 7: Multi-Agent System Engineer (Coordinator, Editor, Terminal)
**Primary Role**: Implement the three core AI agents: Coordinator, Editor, and Terminal

**Responsibilities**:
- **Coordinator Agent**: Task planning, agent selection, workflow orchestration
- **Editor Agent**: Code generation, file editing, refactoring operations
- **Terminal Agent**: Shell command execution, system operations, script running
- Design inter-agent communication protocol
- Implement agent state machine
- Create agent memory and context management
- Build agent decision-making logic
- Implement agent error recovery
- Design agent collaboration patterns
- Create agent performance monitoring

**Scope Boundaries**:
- Does NOT implement tool execution framework (handled by Agent 6)
- Does NOT implement AI model integration (handled by Agent 3)
- Does NOT create frontend components
- Focuses on agent behavior, logic, and coordination

**Deliverables**:
- Coordinator Agent implementation
- Editor Agent implementation
- Terminal Agent implementation
- Agent communication protocol
- Agent orchestration system

**Memory Tracking**: `/memory/agent7_multi_agent_system.log`

---

## Agent 8: Self-Correction Engine Engineer
**Primary Role**: Build automated error detection and correction system

**Responsibilities**:
- Design error detection patterns (syntax errors, runtime errors, logical errors)
- Implement error classification system
- Create automated fix generation using AI
- Build rollback mechanism for failed corrections
- Implement learning system from past corrections
- Design correction confidence scoring
- Create correction history and analytics
- Build unit test generation for corrections
- Implement static code analysis integration
- Design correction suggestion UI data format

**Scope Boundaries**:
- Does NOT implement AI model integration (uses Agent 3's services)
- Does NOT create frontend components
- Does NOT handle tool execution (uses Agent 6's services)
- Focuses on error detection, analysis, and correction logic

**Deliverables**:
- Error detection system
- Automated fix generation
- Rollback mechanism
- Learning system implementation
- Correction analytics

**Memory Tracking**: `/memory/agent8_self_correction.log`

---

## Agent 9: Frontend Web Application Engineer
**Primary Role**: Build React SPA with all required pages and components

**Responsibilities**:
- Create React application structure with TypeScript
- Implement routing for all pages (/chat, /, /sandbox, /tools, /admin)
- Build AI Chat interface with real-time messaging
- Create Command Center dashboard
- Implement Sandbox visualization for AI workspace
- Build Tool Explorer with search and filtering
- Create Admin Dashboard with user management
- Implement authentication flow (login, registration)
- Design responsive UI with Tailwind CSS
- Build state management with Redux or Zustand
- Implement WebSocket client for real-time updates
- Create API client for backend communication

**Scope Boundaries**:
- Does NOT implement backend API (handled by Agent 2)
- Does NOT implement VS Code extension (handled by Agent 10)
- Does NOT handle AI logic
- Focuses purely on web frontend development

**Deliverables**:
- Complete React SPA application
- All required pages and components
- Responsive UI implementation
- State management system
- API client library

**Memory Tracking**: `/memory/agent9_frontend_web.log`

---

## Agent 10: VS Code Extension Engineer
**Primary Role**: Build VS Code extension with chat interface and user info display

**Responsibilities**:
- Create VS Code extension project structure
- Implement extension activation and lifecycle
- Build toggle chat window UI (AI Agentic ↔ User)
- Create chat message rendering and input handling
- Implement API communication with backend
- Build user KEY expiration display
- Create user Tier display (Free/Ultra)
- Implement authentication flow in extension
- Design extension settings and configuration
- Build notification system for key expiration
- Create extension marketplace package
- Implement auto-update mechanism

**Scope Boundaries**:
- Does NOT implement backend API (handled by Agent 2)
- Does NOT implement web frontend (handled by Agent 9)
- Does NOT handle AI logic
- Focuses purely on VS Code extension development

**Deliverables**:
- Complete VS Code extension
- Chat interface implementation
- User info display components
- Extension configuration system
- Marketplace package

**Memory Tracking**: `/memory/agent10_vscode_extension.log`

---

## Agent Communication Protocol

### Event Bus Architecture
Agents communicate through a **Redis-based event bus** with the following message types:

1. **Task Completion Events**: Agent notifies completion of deliverables
2. **Resource Ready Events**: Infrastructure or services become available
3. **Error Events**: Agent encounters blocking issues
4. **Query Events**: Agent needs information from another agent's domain

### Message Format
```json
{
  "agent_id": "agent1",
  "event_type": "task_completion",
  "timestamp": "2026-01-28T10:30:00Z",
  "payload": {
    "task": "database_schema_created",
    "status": "completed",
    "artifacts": ["/path/to/schema.sql"]
  }
}
```

### Non-Blocking Principle
- Agents NEVER wait synchronously for other agents
- Agents subscribe to relevant events and react asynchronously
- If a dependency is not ready, agent proceeds with other tasks
- Agents maintain a task queue and retry failed operations

---

## Quality Assurance Rules

### No Mock Code Policy
- All implementations must be production-ready
- No placeholder functions or stub implementations
- All error handling must be complete
- All edge cases must be handled

### No TODO/Placeholder Policy
- No `TODO` comments in committed code
- No `FIXME` or `HACK` markers
- All features must be fully implemented
- Incomplete work stays in agent's local branch

### Memory Tracking Requirements
- Each agent maintains a daily log in `/memory/agentN_name.log`
- Log format: `[TIMESTAMP] [ACTION] [DETAILS] [STATUS]`
- Logs include: decisions made, blockers encountered, tasks completed
- Logs are used for coordination and debugging

---

## Success Criteria

Each agent is considered successful when:
1. All deliverables are production-ready and tested
2. Memory log is complete and up-to-date
3. No blocking dependencies on other agents
4. Code passes all quality checks (no mock, no TODO)
5. Documentation is complete and accurate
6. Integration tests pass (where applicable)
