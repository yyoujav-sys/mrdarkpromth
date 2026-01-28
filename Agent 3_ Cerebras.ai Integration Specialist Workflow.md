# Agent 3: Cerebras.ai Integration Specialist Workflow

**Role**: Implement AI brain integration with Cerebras.ai and 100-key rotation system.

**Objective**: Create a reliable and efficient client for the Cerebras.ai API, ensuring high availability and optimal use of the 100 free-tier keys.

**Memory Log**: `/memory/agent3_cerebras_integration.log`

---

## Phase 1: API Client Development

1.  **Create Client Module**: Create a new Rust module (e.g., `cerebras_client`) for the API client.
2.  **Implement API Client**: Write an asynchronous client using a library like `reqwest`. Implement methods for making requests to the Cerebras.ai API endpoints.
3.  **Model Selection**: Implement logic to select the appropriate AI model (llama-3.3-70b, gpt-oss-120b, qwen-3-32b) based on the request parameters.

## Phase 2: Key Management and Rotation

1.  **Key Storage**: Implement a secure way to store and access the 100 API keys. For local deployment, this can be an encrypted file or environment variables.
2.  **Key Rotation Algorithm**: Design and implement an algorithm for rotating the API keys. This could be a simple round-robin or a more sophisticated load-balancing approach.
3.  **Health Monitoring**: Create a system to monitor the health of each API key. If a key fails, it should be temporarily removed from the rotation.

## Phase 3: Request Handling and Prompt Engineering

1.  **Request Queue**: Implement a request queue to manage outgoing requests to the Cerebras.ai API, ensuring that the rate limits are not exceeded.
2.  **Retry Logic**: Implement a retry mechanism with exponential backoff for failed requests.
3.  **Prompt Engineering**: Create a system for constructing prompts based on templates. This will allow for easy modification and experimentation with different prompt strategies.
4.  **Response Streaming**: Implement response streaming to handle real-time responses from the AI, particularly for the chat feature.

## Phase 4: Usage Tracking

1.  **Token Usage**: Implement a system to track the token usage for each API key to monitor costs and stay within the free tier limits.
2.  **Analytics**: Create a mechanism to log and analyze API usage patterns.

---

### Quality Mandates

*   **No Mock Implementations**: The client must be fully functional and able to communicate with the Cerebras.ai API.
*   **No TODOs/Placeholders**: All key management and request handling logic must be complete.
*   **Resilience**: The client must be resilient to network errors and API failures.

### Completion Criteria

*   The Cerebras.ai client is fully implemented and tested.
*   The key rotation system is functional and robust.
*   The prompt engineering system is in place.
*   Usage tracking and analytics are operational.
*   All deliverables are documented in the agent's memory log.
