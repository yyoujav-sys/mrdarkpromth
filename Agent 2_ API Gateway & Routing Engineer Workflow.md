# Agent 2: API Gateway & Routing Engineer Workflow

**Role**: Build the API gateway, routing system, and HTTP request handling.

**Objective**: Create a secure, high-performance API gateway that serves as the single entry point for all client requests.

**Memory Log**: `/memory/agent2_api_gateway.log`

---

## Phase 1: Web Server Implementation

1.  **Choose Framework**: Select and add the chosen web framework (e.g., `actix-web` or `axum`) as a dependency in the `api` module's `Cargo.toml`.
2.  **Implement HTTP Server**: Write the main server logic to start the HTTP server and bind it to the configured port.
3.  **Basic Routing**: Create a basic routing structure with a health check endpoint (e.g., `/health`) that returns a `200 OK` response.

## Phase 2: API Endpoint and Middleware Development

1.  **RESTful Endpoints**: Implement the RESTful API endpoints for all client operations as defined in the project analysis. This includes **stubbed interfaces** with clear definitions and no placeholder logic.
2.  **Authentication Middleware**: Create a middleware component to handle JWT validation. It should extract the token from the `Authorization` header and verify its signature.
3.  **Rate Limiting**: Implement a rate-limiting middleware to prevent abuse. Use an in-memory store (like a `HashMap`) or Redis for tracking request counts per IP address.
4.  **CORS Configuration**: Configure Cross-Origin Resource Sharing (CORS) to allow requests from the React SPA's domain.

## Phase 3: Real-time Communication with WebSockets

1.  **WebSocket Server**: Implement a WebSocket endpoint (e.g., `/ws/chat`) to handle real-time communication for the AI chat.
2.  **Connection Management**: Create a system to manage active WebSocket connections, including handling new connections and disconnections.

## Phase 4: API Documentation

1.  **OpenAPI Specification**: Generate an OpenAPI 3.0 specification for the API. This can be done manually or using a library that automatically generates it from the code.
2.  **Swagger UI**: Integrate Swagger UI to serve the interactive API documentation from an endpoint (e.g., `/api-docs`).

---

### Quality Mandates

*   **No Mock Implementations**: All endpoints and middleware must be fully functional. Business logic can be stubbed out with clear interfaces for other agents to implement.
*   **No TODOs/Placeholders**: Do not leave any `TODO` comments. All routing and request handling logic must be complete, except for defined stubs.
*   **Comprehensive Testing**: Write unit and integration tests for all endpoints and middleware.

### Completion Criteria

*   All API endpoints are implemented and documented.
*   Authentication and rate-limiting middleware are functional.
*   The WebSocket server is running and can accept connections.
*   The API documentation is accessible and accurate.
*   All deliverables are documented in the agent's memory log.
