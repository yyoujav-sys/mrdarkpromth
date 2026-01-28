# Memory Tracking System & Agent Coordination Protocol

**Author**: Manus AI  
**Project**: MR.DarkPromth Multi-Agent Orchestration System  
**Version**: 1.0  
**Date**: January 28, 2026

---

## Executive Summary

The MR.DarkPromth project employs a sophisticated memory tracking and coordination system that enables ten independent AI agents to work in parallel without blocking dependencies. This document defines the memory log format, storage architecture, coordination protocols, and event-driven communication patterns that ensure seamless collaboration while maintaining agent autonomy.

---

## Memory Tracking System Architecture

### Design Principles

The memory tracking system is built on three core principles. First, **agent autonomy** ensures that each agent maintains its own memory log without requiring centralized coordination. Second, **asynchronous access** allows agents to read other agents' logs without blocking or waiting for writes to complete. Third, **append-only semantics** guarantee that logs are immutable once written, preventing race conditions and enabling reliable audit trails.

### Storage Structure

All agent memory logs are stored in a dedicated directory structure under `/memory` in the project root. Each agent maintains a single log file named according to its role identifier. The directory structure follows a flat hierarchy to simplify access patterns and reduce filesystem overhead.

```
/memory/
├── agent1_infrastructure.log
├── agent2_api_gateway.log
├── agent3_cerebras_integration.log
├── agent4_jailbreak_ultra.log
├── agent5_user_management.log
├── agent6_tool_executor.log
├── agent7_multi_agent_system.log
├── agent8_self_correction.log
├── agent9_frontend_web.log
└── agent10_vscode_extension.log
```

### Log Entry Format

Each log entry follows a structured format that enables both human readability and machine parsing. The format consists of four mandatory fields enclosed in square brackets, followed by optional metadata in JSON format.

**Standard Format**:
```
[TIMESTAMP] [ACTION] [DETAILS] [STATUS] {optional_json_metadata}
```

**Field Specifications**:

| Field | Description | Format | Example |
|-------|-------------|--------|---------|
| TIMESTAMP | ISO 8601 timestamp in UTC | `YYYY-MM-DDTHH:mm:ss.sssZ` | `2026-01-28T10:30:45.123Z` |
| ACTION | High-level action category | Enum: `START`, `PROGRESS`, `COMPLETE`, `BLOCKED`, `ERROR`, `QUERY`, `RESPONSE` | `COMPLETE` |
| DETAILS | Human-readable description | Free text (max 200 chars) | `Database schema migration completed` |
| STATUS | Current state of the action | Enum: `SUCCESS`, `FAILED`, `PENDING`, `IN_PROGRESS` | `SUCCESS` |
| JSON Metadata | Optional structured data | Valid JSON object | `{"files": ["schema.sql"], "duration_ms": 1234}` |

**Example Log Entries**:

```
[2026-01-28T10:30:45.123Z] [START] [Initializing Rust workspace] [IN_PROGRESS]
[2026-01-28T10:31:12.456Z] [PROGRESS] [Created Cargo.toml and module structure] [SUCCESS] {"modules": ["api", "core", "services", "db"]}
[2026-01-28T10:32:30.789Z] [COMPLETE] [Docker Compose configuration written] [SUCCESS] {"file": "docker-compose.yml", "services": 4}
[2026-01-28T10:33:15.012Z] [BLOCKED] [Waiting for database schema from Agent 1] [PENDING] {"required_artifact": "schema.sql"}
[2026-01-28T10:35:00.345Z] [ERROR] [Failed to connect to Redis] [FAILED] {"error": "ECONNREFUSED", "retry_count": 3}
```

### Log Rotation and Retention

Memory logs are append-only and do not implement automatic rotation during active development. Once a project milestone is reached, logs can be archived manually by moving them to a timestamped archive directory. The retention policy recommends keeping logs for at least 90 days after project completion to support debugging and auditing.

---

## Agent Coordination Protocol

### Event Bus Architecture

The coordination system uses **Redis Streams** for guaranteed delivery, ensuring at-least-once delivery semantics. This replaces the previous Pub/Sub model, which did not guarantee message delivery. Redis Streams provide a lightweight, high-performance messaging layer that supports asynchronous communication without introducing tight coupling between agents. Redis is chosen for its low latency, built-in persistence options, and native support for publish-subscribe patterns.

### Event Types and Semantics

The protocol defines seven core event types that cover all coordination scenarios:

| Event Type | Purpose | Publisher | Subscribers | Delivery Guarantee |
|------------|---------|-----------|-------------|-------------------|
| `task_completion` | Notify completion of a deliverable | Any agent | All agents | At-least-once |
| `resource_ready` | Signal availability of a shared resource | Infrastructure agents | Dependent agents | At-least-once |
| `error_event` | Report blocking errors requiring intervention | Any agent | Coordinator, Self-Correction Engine | At-least-once |
| `query_event` | Request information from another agent's domain | Any agent | Specific agent | Exactly-once (with timeout) |
| `response_event` | Reply to a query event | Queried agent | Querying agent | Exactly-once |
| `heartbeat` | Periodic liveness signal | All agents | Monitoring system | Best-effort |
| `shutdown` | Graceful shutdown notification | Any agent | All agents | Best-effort |

### Event Message Format

All events are serialized as JSON objects with a standardized schema. The schema includes mandatory fields for routing and correlation, plus an extensible payload for event-specific data.

**Schema**:
```json
{
  "event_id": "uuid-v4",
  "agent_id": "agent1",
  "event_type": "task_completion",
  "timestamp": "2026-01-28T10:30:00Z",
  "correlation_id": "optional-uuid-for-request-response",
  "payload": {
    "task": "database_schema_created",
    "status": "completed",
    "artifacts": ["/path/to/schema.sql"],
    "metadata": {}
  }
}
```

**Field Descriptions**:

- **event_id**: Unique identifier for the event (UUID v4). Used for deduplication and tracing.
- **agent_id**: Identifier of the agent publishing the event (e.g., `agent1`, `agent2`).
- **event_type**: One of the seven defined event types.
- **timestamp**: ISO 8601 timestamp when the event was created.
- **correlation_id**: Optional field for linking request-response pairs (used with `query_event` and `response_event`).
- **payload**: Event-specific data structure. Contents vary by event type.

### Channel Naming Convention

Redis Streams are organized hierarchically to enable selective subscription and efficient routing. The naming convention follows the pattern: `mr_darkpromth:{scope}:{event_type}`.

**Stream Examples**:

- `mr_darkpromth:global:task_completion` - All task completion events
- `mr_darkpromth:global:error_event` - All error events
- `mr_darkpromth:agent1:query_event` - Queries directed to Agent 1
- `mr_darkpromth:agent1:response_event` - Responses from Agent 1
- `mr_darkpromth:system:heartbeat` - Heartbeat signals from all agents

### Subscription Patterns

Each agent subscribes to a minimal set of streams based on its role and dependencies. This reduces message processing overhead and prevents unnecessary wake-ups.

**Recommended Subscriptions by Agent**:

| Agent | Subscribed Streams | Rationale |
|-------|---------------------|-----------|
| Agent 1 (Infrastructure) | `global:error_event`, `agent1:query_event` | Needs to respond to infrastructure queries and monitor system errors |
| Agent 2 (API Gateway) | `global:task_completion`, `agent2:query_event` | Waits for service readiness from other agents |
| Agent 3 (Cerebras Integration) | `global:error_event`, `agent3:query_event` | Responds to AI model queries and monitors API failures |
| Agent 4 (Jailbreak) | `agent3:resource_ready`, `agent4:query_event` | Depends on Cerebras client being ready |
| Agent 5 (User Management) | `agent1:resource_ready`, `agent5:query_event` | Depends on database being ready |
| Agent 6 (Tool Executor) | `global:task_completion`, `agent6:query_event` | Waits for tools to be registered by other agents |
| Agent 7 (Multi-Agent System) | `global:*` | Coordinator needs visibility into all events |
| Agent 8 (Self-Correction) | `global:error_event`, `global:task_completion` | Monitors errors and validates completions |
| Agent 9 (Frontend Web) | `agent2:resource_ready`, `agent9:query_event` | Depends on API gateway being ready |
| Agent 10 (VS Code Extension) | `agent2:resource_ready`, `agent10:query_event` | Depends on API gateway being ready |

### Non-Blocking Communication Patterns

The protocol enforces strict non-blocking semantics through three key patterns:

**Pattern 1: Fire-and-Forget Notifications**

Agents publish completion events without waiting for acknowledgments. Subscribers process events asynchronously in their own event loops.

```rust
// Publisher (Agent 1)
redis_client.xadd("mr_darkpromth:global:task_completion", "*", json!({
    "event_id": uuid::Uuid::new_v4(),
    "agent_id": "agent1",
    "event_type": "task_completion",
    "timestamp": Utc::now(),
    "payload": {
        "task": "database_schema_created",
        "artifacts": ["/path/to/schema.sql"]
    }
}));
// No waiting for response
```

**Pattern 2: Query with Timeout**

When an agent needs information from another agent, it publishes a query event and subscribes to a response stream with a timeout. If no response arrives within the timeout period, the agent proceeds with alternative logic or retries later.

```rust
// Querier (Agent 2)
let correlation_id = uuid::Uuid::new_v4();
redis_client.xadd("mr_darkpromth:agent1:query_event", "*", json!({
    "event_id": uuid::Uuid::new_v4(),
    "agent_id": "agent2",
    "event_type": "query_event",
    "timestamp": Utc::now(),
    "correlation_id": correlation_id,
    "payload": {
        "query": "get_database_connection_string"
    }
}));

// Subscribe to response with 5-second timeout
match redis_client.xreadgroup("mr_darkpromth:agent2:response_event", "my_group", correlation_id, 5000).await {
    Ok(response) => { /* Use response */ },
    Err(_timeout) => { /* Proceed with fallback logic */ }
}
```

**Pattern 3: Event-Driven Task Queue**

Agents maintain internal task queues that are populated by incoming events. When a dependency becomes available (e.g., a `resource_ready` event), the agent checks its queue for tasks that can now proceed.

```rust
// Subscriber (Agent 5)
loop {
    let event = redis_client.xread("mr_darkpromth:agent1:resource_ready", "0-0").await;
    match event.event_type {
        "resource_ready" if event.payload.resource == "database" => {
            // Check task queue for database-dependent tasks
            for task in task_queue.iter_mut() {
                if task.requires("database") && task.status == "blocked" {
                    task.status = "ready";
                    executor.schedule(task);
                }
            }
        },
        _ => {}
    }
}
```

### Error Handling and Retry Logic

The coordination protocol includes built-in error handling mechanisms to ensure system resilience:

**Connection Failures**: If an agent loses connection to Redis, it enters a reconnection loop with exponential backoff (starting at 1 second, max 30 seconds). During reconnection, the agent continues local work and queues outgoing events in memory.

**Message Delivery Failures**: Since Redis Streams provide at-least-once delivery, agents must implement idempotency checks using the `event_id` field. Duplicate events are detected and discarded.

**Timeout Handling**: Query events that timeout are logged to the agent's memory log with status `FAILED`. The querying agent retries up to three times with increasing timeouts (5s, 10s, 20s) before escalating to an error event.

**Poison Messages**: Events that fail to parse or contain invalid data are logged to a dead-letter stream (`mr_darkpromth:system:dead_letter`) for manual inspection.

---

## Coordination Workflow Examples

### Example 1: Agent 5 Waiting for Database Schema

**Scenario**: Agent 5 (User Management) needs the database schema to be ready before it can create user-related tables.

**Workflow**:

1. Agent 5 starts and checks if the database schema exists.
2. Schema does not exist, so Agent 5 logs a `BLOCKED` entry and subscribes to `mr_darkpromth:agent1:resource_ready`.
3. Agent 5 proceeds with other tasks (e.g., designing the authentication logic).
4. Agent 1 completes the schema creation and publishes a `resource_ready` event.
5. Agent 5 receives the event, updates its task queue, and proceeds with table creation.

**Log Entries**:

```
# Agent 5 log
[2026-01-28T10:30:00.000Z] [START] [Checking database schema availability] [IN_PROGRESS]
[2026-01-28T10:30:01.000Z] [BLOCKED] [Database schema not ready, subscribing to resource_ready] [PENDING] {"dependency": "agent1:database_schema"}
[2026-01-28T10:30:02.000Z] [PROGRESS] [Designing authentication logic while waiting] [IN_PROGRESS]
[2026-01-28T10:35:00.000Z] [PROGRESS] [Received resource_ready event from Agent 1] [SUCCESS] {"event_id": "abc123"}
[2026-01-28T10:35:01.000Z] [PROGRESS] [Creating user management tables] [IN_PROGRESS]
```

### Example 2: Agent 8 Detecting and Correcting an Error

**Scenario**: Agent 7 (Multi-Agent System) encounters a runtime error while implementing the Coordinator agent. Agent 8 (Self-Correction Engine) detects the error and generates a fix.

**Workflow**:

1. Agent 7 encounters an error and publishes an `error_event` to `mr_darkpromth:global:error_event`.
2. Agent 8 is subscribed to the error stream and receives the event.
3. Agent 8 analyzes the error, generates a fix using the Cerebras.ai API, and applies the fix.
4. Agent 8 publishes a `task_completion` event indicating the correction was successful.
5. Agent 7 receives the completion event and resumes its work.

**Event Flow**:

```json
// Agent 7 publishes error
{
  "event_id": "err-001",
  "agent_id": "agent7",
  "event_type": "error_event",
  "timestamp": "2026-01-28T11:00:00Z",
  "payload": {
    "error_type": "runtime_error",
    "message": "Undefined variable 'context' in coordinator.rs:45",
    "file": "server/agents/coordinator.rs",
    "line": 45
  }
}

// Agent 8 publishes correction
{
  "event_id": "fix-001",
  "agent_id": "agent8",
  "event_type": "task_completion",
  "timestamp": "2026-01-28T11:02:30Z",
  "correlation_id": "err-001",
  "payload": {
    "task": "error_correction",
    "status": "completed",
    "fix_applied": true,
    "file": "server/agents/coordinator.rs",
    "changes": "Added 'let context = build_context();' at line 44"
  }
}
```

---

## Memory Log Query Interface

To facilitate coordination, agents can query other agents' memory logs programmatically. A shared utility library provides functions for parsing and searching logs.

### Query Functions

**Function**: `get_latest_status(agent_id: &str) -> Option<LogEntry>`

Returns the most recent log entry for the specified agent.

**Function**: `find_completion_events(agent_id: &str, task: &str) -> Vec<LogEntry>`

Returns all `COMPLETE` entries for a specific task from the specified agent's log.

**Function**: `check_blocked_status(agent_id: &str) -> Vec<LogEntry>`

Returns all `BLOCKED` entries from the specified agent's log, indicating pending dependencies.

**Function**: `get_artifacts(agent_id: &str) -> Vec<String>`

Extracts all artifact paths from `COMPLETE` entries in the specified agent's log.

### Example Usage

```rust
use memory_log::*;

// Check if Agent 1 has completed the database schema
let completions = find_completion_events("agent1", "database_schema_created");
if !completions.is_empty() {
    let artifacts = get_artifacts("agent1");
    println!("Database schema available at: {:?}", artifacts);
} else {
    println!("Database schema not ready yet");
}
```

---

## Monitoring and Observability

### Heartbeat Mechanism

Each agent publishes a heartbeat event every 30 seconds to the `mr_darkpromth:system:heartbeat` stream. A monitoring service subscribes to this stream and tracks agent liveness. If an agent fails to send a heartbeat for 2 minutes, it is marked as unhealthy.

**Heartbeat Event Format**:

```json
{
  "event_id": "hb-001",
  "agent_id": "agent1",
  "event_type": "heartbeat",
  "timestamp": "2026-01-28T12:00:00Z",
  "payload": {
    "status": "healthy",
    "tasks_in_progress": 2,
    "tasks_completed": 15,
    "memory_usage_mb": 256
  }
}
```

### Centralized Logging

In addition to individual memory logs, all events published to the Redis event bus are also streamed to a centralized logging service (e.g., a file sink or a log aggregation tool). This provides a global view of system activity and simplifies debugging.

### Metrics and Dashboards

The monitoring system exposes the following metrics:

- **Agent Liveness**: Number of healthy agents vs. total agents
- **Event Throughput**: Events published per second, by type
- **Task Completion Rate**: Tasks completed per hour, by agent
- **Error Rate**: Error events per hour, by agent
- **Average Response Time**: Time between query and response events

These metrics are visualized in a real-time dashboard built with the web interface.

---

## Best Practices for Agents

### Writing to Memory Logs

Agents should write to their memory logs at key decision points, not for every minor operation. Recommended logging points include starting a new phase, completing a deliverable, encountering a blocker, and resolving an error.

### Subscribing to Events

Agents should subscribe only to the streams they need. Over-subscription leads to unnecessary message processing and can degrade performance.

### Handling Timeouts

When a query times out, agents should not block indefinitely. Instead, they should log the timeout, proceed with alternative logic, and optionally retry later.

### Idempotency

All event handlers must be idempotent. Since Redis Streams provide at-least-once delivery, duplicate events are possible. Agents should use the `event_id` field to detect and discard duplicates.

### Graceful Shutdown

When an agent completes its work or needs to shut down, it should publish a `shutdown` event to notify other agents. This allows dependent agents to adjust their expectations and avoid waiting for events that will never arrive.

---

## Conclusion

The memory tracking and coordination system provides a robust foundation for the MR.DarkPromth multi-agent architecture. By combining append-only memory logs with an event-driven coordination protocol, the system enables ten independent agents to collaborate effectively without introducing blocking dependencies or tight coupling. This design ensures scalability, resilience, and maintainability as the project evolves.
