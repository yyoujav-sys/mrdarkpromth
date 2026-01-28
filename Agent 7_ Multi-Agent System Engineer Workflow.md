# Agent 7: Multi-Agent System Engineer Workflow

**Role**: Implement the three core AI agents: Coordinator, Editor, and Terminal.

**Objective**: To build the core AI agent system that drives the platform's autonomous capabilities.

**Memory Log**: `/memory/agent7_multi_agent_system.log`

---

## Phase 1: Agent Framework

1.  **Agent Trait**: Define a common `Agent` trait that all agents (Coordinator, Editor, Terminal) will implement. This trait should include methods for handling messages, making decisions, and executing actions.
2.  **State Machine**: Implement a state machine for the agents to manage their lifecycle (e.g., `idle`, `thinking`, `acting`).

## Phase 2: Core Agent Implementation

1.  **Coordinator Agent**: Implement the Coordinator agent, which is responsible for high-level task planning and orchestration. It will receive tasks from the API gateway and break them down into smaller steps for the other agents.
2.  **Editor Agent**: Implement the Editor agent, which is responsible for all code generation and file manipulation tasks. It will receive instructions from the Coordinator and use the tool system to perform its tasks.
3.  **Terminal Agent**: Implement the Terminal agent, which is responsible for executing shell commands and other system-level operations. It will also receive instructions from the Coordinator.

## Phase 3: Inter-Agent Communication

1.  **Communication Protocol**: Implement the inter-agent communication protocol using the event bus (Redis Pub/Sub) set up by Agent 1. This will allow the agents to communicate asynchronously.
2.  **Message Handling**: Implement the logic for each agent to handle incoming messages and events.

## Phase 4: Agent Logic and Collaboration

1.  **Decision-Making**: Implement the decision-making logic for each agent. This will involve using the Cerebras.ai API to generate responses and decide on the next course of action.
2.  **Collaboration Patterns**: Design and implement collaboration patterns for the agents. For example, the Coordinator might delegate a coding task to the Editor, which then uses the Terminal to test the code.

---

### Quality Mandates

*   **No Mock Implementations**: All agents and their core logic must be fully functional.
*   **No TODOs/Placeholders**: All communication and collaboration patterns must be complete.
*   **Autonomy**: The agents should be able to operate autonomously with minimal human intervention.

### Completion Criteria

*   The Coordinator, Editor, and Terminal agents are fully implemented and tested.
*   The inter-agent communication system is functional and reliable.
*   The agents are able to collaborate to complete complex tasks.
*   All deliverables are documented in the agent's memory log.
