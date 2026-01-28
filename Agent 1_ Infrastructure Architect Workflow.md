# Agent 1: Infrastructure Architect Workflow

**Role**: Design and implement core system infrastructure and deployment architecture.

**Objective**: Create a robust, scalable, and secure foundation for the MR.DarkPromth platform. All infrastructure must be defined as code and be fully automated.

**Memory Log**: `/memory/agent1_infrastructure.log`

---

## Phase 1: Project Initialization

1.  **Create Root Directory**: Create the main project directory named `mr_darkpromth`.
2.  **Initialize Rust Workspace**: Inside the root directory, initialize a Rust workspace using `cargo init --workspace`.
3.  **Create Core Modules**: Create the initial directory structure for the backend modules within the workspace (e.g., `api`, `core`, `services`, `db`).
4.  **Version Control**: Initialize a Git repository and create the initial commit with a `.gitignore` file for Rust projects.

## Phase 2: Containerization with Docker

1.  **Create Dockerfile**: Write a multi-stage `Dockerfile` for the Rust application. The build stage should compile the application in a release profile, and the final stage should copy the binary into a minimal container image (e.g., `debian:slim`).
2.  **Create Docker Compose File**: Write a `docker-compose.yml` file to orchestrate the different services:
    *   `backend`: The main Rust application.
    *   `postgres`: The PostgreSQL database.
    *   `redis`: The Redis cache and event bus.
    *   `nginx`: The Nginx reverse proxy.
3.  **Configure Nginx**: Write the Nginx configuration (`nginx.conf`) to act as a reverse proxy, routing requests to the backend service and serving the frontend static files.

## Phase 3: Database and Storage Setup

1.  **Database Schema**: Design the initial PostgreSQL database schema. Create SQL scripts for table creation (e.g., `users`, `tiers`, `api_keys`).
2.  **Database Migrations**: Set up a database migration tool (e.g., `sqlx-cli`) and create the initial migration file with the schema.
3.  **File System Structure**: Define and create the directory structure for agent memory logs under `/memory`.

## Phase 4: System Configuration and Logging

1.  **Environment Configuration**: Implement a configuration management system using environment variables. Create a `.env.example` file with all required variables.
2.  **Structured Logging**: Integrate a structured logging library (e.g., `tracing` with `tracing-subscriber`) to output logs in JSON format.

## Phase 5: CI/CD Pipeline

1.  **Create CI/CD Configuration**: Create a configuration file for a CI/CD pipeline (e.g., `.github/workflows/rust.yml` for GitHub Actions). The pipeline should:
    *   Run `cargo check` and `cargo fmt`.
    *   Run `cargo test`.
    *   Build the Docker image.
    *   (Optional) Push the image to a container registry.

---

### Quality Mandates

*   **No Mock Implementations**: All scripts, configurations, and code must be fully functional and production-ready.
*   **No TODOs/Placeholders**: Do not leave any `TODO` comments or placeholder values. All configurations must be complete.
*   **Idempotency**: All scripts must be idempotent, meaning they can be run multiple times without causing errors.

### Completion Criteria

*   The entire infrastructure can be brought up using `docker-compose up -d`.
*   The database is initialized with the correct schema.
*   The CI/CD pipeline is configured and passes.
*   All deliverables are documented in the agent's memory log.
