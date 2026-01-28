# Agent 10: VS Code Extension Engineer Workflow

**Role**: Build VS Code extension with chat interface and user info display.

**Objective**: To create a seamless VS Code extension that integrates the MR.DarkPromth AI capabilities directly into the developer's workflow.

**Memory Log**: `/memory/agent10_vscode_extension.log`

---

## Phase 1: Extension Project Setup

1.  **Initialize Extension Project**: Use the Yeoman generator for VS Code extensions to create the initial project structure.
2.  **Configure TypeScript**: Set up TypeScript with appropriate compiler options for VS Code extension development.
3.  **Install Dependencies**: Install necessary dependencies for API communication and UI rendering.

## Phase 2: Extension Activation and Lifecycle

1.  **Activation Events**: Define the activation events for the extension (e.g., when the user opens a specific command or when VS Code starts).
2.  **Extension Context**: Set up the extension context to manage the lifecycle of the extension, including activation and deactivation.

## Phase 3: Chat Interface Implementation

1.  **Webview Panel**: Create a webview panel to host the chat interface. This will allow for a rich UI using HTML, CSS, and JavaScript.
2.  **Chat UI**: Build the chat UI with message rendering, input handling, and real-time updates.
3.  **API Communication**: Implement the logic to communicate with the MR.DarkPromth backend API for sending messages and receiving responses.

## Phase 4: User Information Display

1.  **User Key Expiration**: Create a status bar item or a panel to display the user's API key expiration date and time.
2.  **User Tier Display**: Display the user's tier (Free/Ultra) in the extension UI.
3.  **Authentication**: Implement the authentication flow to retrieve and store the user's credentials securely.

## Phase 5: Extension Configuration and Publishing

1.  **Settings**: Create extension settings to allow users to configure the extension (e.g., API endpoint, theme preferences).
2.  **Packaging**: Package the extension for distribution on the VS Code Marketplace.
3.  **Auto-Update**: Implement an auto-update mechanism to ensure users always have the latest version of the extension.

---

### Quality Mandates

*   **No Mock Implementations**: The extension must be fully functional and able to communicate with the backend API.
*   **No TODOs/Placeholders**: All features must be complete and ready for use.
*   **User Experience**: The extension should be intuitive and easy to use, with clear feedback for all actions.

### Completion Criteria

*   The VS Code extension is fully implemented and tested.
*   The chat interface is functional and can communicate with the backend.
*   The user information display is accurate and up-to-date.
*   All deliverables are documented in the agent's memory log.
