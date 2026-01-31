# MR.DarkPromth VS Code Extension

A VS Code extension that integrates MR.DarkPromth AI capabilities directly into your development workflow with chat interface and user information display.

## Features

- **Chat Interface**: Rich webview-based chat interface for interacting with MR.DarkPromth AI
- **User Authentication**: Secure authentication flow with JWT token management
- **User Tier Display**: Visual indication of user tier (Free/Premium/Ultra) in status bar
- **API Key Expiration**: Real-time display of API key expiration status
- **Jailbreak Support**: Ultra tier users can apply jailbreak prompts to enhance AI responses
- **Chat History**: Persistent chat history with configurable maximum length
- **Auto-Update**: Automatic update checking for latest extension version
- **Configurable Settings**: Customizable API endpoint, theme preferences, and more
- **Agent Coordination**: Redis event bus integration for coordinating with other MR.DarkPromth agents

## Installation

### From VS Code Marketplace
1. Open VS Code
2. Search for "MR.DarkPromth" in Extensions
3. Click Install

### From Source
1. Clone this repository
2. Navigate to the extension directory
3. Run `npm install`
4. Run `npm run compile`
5. Press F5 to launch a new VS Code window with the extension loaded

## Configuration

The extension can be configured via VS Code settings:

- `mr-darkpromth.apiEndpoint`: The API endpoint for MR.DarkPromth backend (default: http://127.0.0.1:8080)
- `mr-darkpromth.theme`: Theme for the chat interface (dark/light/auto)
- `mr-darkpromth.autoUpdate`: Automatically check for updates (default: true)
- `mr-darkpromth.maxHistoryLength`: Maximum number of messages in chat history (default: 50)

## Usage

### Authentication
1. Open command palette (Ctrl+Shift+P)
2. Run "MR.DarkPromth: Authenticate with MR.DarkPromth"
3. Enter your email and password

### Opening Chat
- Run command "MR.DarkPromth: Open MR.DarkPromth Chat"
- Or click on the MR.DarkPromth icon in the activity bar

### User Information
The extension displays:
- User tier in the status bar
- API key expiration status
- Real-time updates every minute

### Chat Features
- Send messages to the AI
- View chat history
- Clear chat history
- Ultra tier users can apply jailbreak prompts

## Development

### Project Structure
```
vscode-extension/
├── src/
│   ├── extension.ts          # Main extension entry point
│   ├── authManager.ts        # Authentication management
│   ├── apiClient.ts          # API communication
│   ├── chatPanelProvider.ts  # Chat interface webview
│   ├── statusBarManager.ts   # Status bar user info display
│   └── eventBusCoordinator.ts # Redis event bus coordination
├── resources/
│   └── icon.svg              # Extension icon
├── package.json              # Extension manifest
├── tsconfig.json             # TypeScript configuration
└── README.md                 # This file
```

### Building
```bash
npm install
npm run compile
```

### Packaging
```bash
npm run package
```

This creates a `.vsix` file that can be installed manually.

## Requirements

- VS Code 1.74.0 or higher
- Node.js 18.0.0 or higher
- MR.DarkPromth backend API running on configured endpoint
- Redis server (for agent coordination, optional but recommended)

## Agent Coordination

This extension includes Redis event bus coordination for seamless integration with other MR.DarkPromth agents:

- **Event Publishing**: Publishes task completion and heartbeat events
- **Resource Monitoring**: Listens for API gateway readiness events
- **Error Handling**: Subscribes to system-wide error events
- **Query Response**: Responds to status queries from other agents

### Redis Configuration

The extension attempts to connect to Redis at `redis://localhost:6379` by default. If Redis is not available, the extension will function normally but without agent coordination features.

## License

MIT

## Support

For issues and feature requests, please visit the project repository.
