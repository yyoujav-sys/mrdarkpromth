# Agent 9: Frontend Web Application Engineer Workflow

**Role**: Build React SPA with all required pages and components.

**Objective**: To create a modern, responsive, and user-friendly web interface for the MR.DarkPromth platform.

**Memory Log**: `/memory/agent9_frontend_web.log`

---

## Phase 1: Project Setup and Design System

1.  **Initialize React Project**: Set up a React project with TypeScript using Vite as the build tool.
2.  **Install Dependencies**: Install necessary dependencies including React Router (or Wouter), Tailwind CSS, and any UI component libraries (e.g., shadcn/ui).
3.  **Configure Tailwind**: Set up Tailwind CSS with a custom configuration that includes the project's color palette and design tokens.
4.  **Design System**: Create a design system document that defines the color palette, typography, spacing, and component styles. This will ensure consistency across the application.

## Phase 2: Routing and Layout

1.  **Set Up Routing**: Configure the routing system to handle all required pages (`/chat`, `/`, `/sandbox`, `/tools`, `/admin`).
2.  **Create Layout Components**: Build reusable layout components such as headers, footers, and navigation bars.
3.  **Responsive Design**: Ensure that all layout components are responsive and work well on different screen sizes.

## Phase 3: Page Implementation

1.  **AI Chat Interface**: Build the AI Chat page with real-time messaging, message history, and markdown rendering for AI responses.
2.  **Command Center**: Create the Command Center dashboard with an overview of the user's projects, recent activity, and quick actions.
3.  **Sandbox**: Implement the Sandbox page to visualize the AI workspace and display outputs to the user.
4.  **Tool Explorer**: Build the Tool Explorer page with a searchable and filterable list of available tools.
5.  **Admin Dashboard**: Create the Admin Dashboard with user management, analytics, and system settings.

## Phase 4: State Management and API Integration

1.  **State Management**: Set up a state management solution (e.g., Redux, Zustand, or React Context) to manage global application state.
2.  **API Client**: Create an API client to communicate with the backend. This should handle authentication, request/response formatting, and error handling.
3.  **WebSocket Client**: Implement a WebSocket client for real-time updates in the chat interface.

## Phase 5: Authentication and User Experience

1.  **Authentication Flow**: Implement the login and registration flow, including JWT token storage and refresh logic.
2.  **Protected Routes**: Set up route guards to protect pages that require authentication.
3.  **User Profile**: Create a user profile page where users can view and update their information.

---

### Quality Mandates

*   **No Mock Implementations**: All components and pages must be fully functional and connected to the backend API.
*   **No TODOs/Placeholders**: Do not leave any `TODO` comments or placeholder content in the UI.
*   **Accessibility**: Ensure that the application is accessible to users with disabilities by following WCAG guidelines.

### Completion Criteria

*   All pages and components are implemented and functional.
*   The application is responsive and works well on different devices.
*   The state management and API integration are complete.
*   All deliverables are documented in the agent's memory log.
