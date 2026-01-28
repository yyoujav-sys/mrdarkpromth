# Agent 6: MasterToolExecutor & Tool System Engineer Workflow

**Role**: Build the tool execution framework and tool registry.

**Objective**: To create a flexible and extensible tool system that allows the AI agents to perform a wide range of tasks.

**Memory Log**: `/memory/agent6_tool_executor.log`

---

## Phase 1: Tool Framework Design

1.  **Tool Interface**: Define a common `Tool` trait that all tools must implement. This trait should include methods for executing the tool, getting its description, and defining its input/output schema.
2.  **Plugin Architecture**: Design a plugin system that allows new tools to be added to the system without modifying the core code. This could be based on dynamic linking or a similar mechanism.

## Phase 2: MasterToolExecutor Implementation

1.  **Tool Registry**: Implement a tool registry that discovers and loads all available tools at startup.
2.  **Orchestration Engine**: Build the `MasterToolExecutor`, which is responsible for selecting the appropriate tool based on the agent's request, validating the input, executing the tool, and returning the output.
3.  **Sandbox Environment**: Create a sandboxed environment for tool execution to prevent tools from interfering with each other or the main application.

## Phase 3: Built-in Tools

1.  **File Operations**: Implement a set of built-in tools for file operations (e.g., `read_file`, `write_file`, `list_files`).
2.  **Web Scraping**: Implement a tool for web scraping using a library like `scraper`.
3.  **Code Execution**: Implement a tool for executing code in a sandboxed environment. This is a critical and high-risk tool that requires careful implementation.
4.  **Database Queries**: Implement a tool for executing SQL queries against the database.

## Phase 4: Tool Management

1.  **Tool Permissions**: Design and implement a permission system that restricts access to certain tools based on the agent's role or the user's tier.
2.  **Tool Logging**: Implement a logging system that records all tool executions, including the input, output, and any errors.

---

### Quality Mandates

*   **No Mock Implementations**: The tool framework and all built-in tools must be fully functional.
*   **No TODOs/Placeholders**: All tool management and security features must be complete.
*   **Extensibility**: The tool system should be designed to be easily extensible with new tools.

### Completion Criteria

*   The tool framework and `MasterToolExecutor` are fully implemented and tested.
*   A set of built-in tools is available and functional.
*   The tool permission and logging systems are in place.
*   All deliverables are documented in the agent's memory log.
