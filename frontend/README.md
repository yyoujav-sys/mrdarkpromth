# MR.DarkPromth Frontend

Advanced AI platform frontend with ultra-tier capabilities and jailbreak features.

## Features

- **Modern React SPA** built with TypeScript, Vite, and Tailwind CSS
- **AI Chat Interface** with real-time messaging and markdown rendering
- **Command Center Dashboard** with system overview and quick actions
- **Sandbox Environment** for testing AI capabilities
- **Tool Explorer** with searchable and filterable tools
- **Admin Dashboard** with user management and analytics
- **User Profile Management** with tier-based access control
- **Authentication System** with JWT and protected routes
- **State Management** using Zustand
- **Real-time Updates** with WebSocket integration
- **Responsive Design** with dark theme and neon accents

## Tech Stack

- **Framework**: React 18 with TypeScript
- **Build Tool**: Vite
- **Styling**: Tailwind CSS with custom design system
- **Routing**: React Router DOM
- **State Management**: Zustand
- **HTTP Client**: Axios
- **UI Components**: Custom components with Lucide icons
- **Markdown**: react-markdown with syntax highlighting
- **WebSocket**: Native WebSocket API with fallback

## Getting Started

### Prerequisites

- Node.js 18+ 
- npm or yarn

### Installation

1. Clone the repository
2. Install dependencies:
   ```bash
   npm install
   ```

3. Set up environment variables:
   ```bash
   cp .env.example .env
   ```
   Edit `.env` with your API configuration.

4. Start the development server:
   ```bash
   npm run dev
   ```

5. Open [http://localhost:5173](http://localhost:5173) in your browser.

### Environment Variables

- `VITE_API_BASE_URL`: Backend API URL (default: http://localhost:8080)
- `VITE_WS_URL`: WebSocket URL (default: ws://localhost:8080/ws)
- `VITE_ENABLE_JAILBREAK`: Enable jailbreak features (default: true)
- `VITE_ENABLE_ANALYTICS`: Enable analytics (default: true)
- `VITE_DEV_MODE`: Development mode (default: true)

## Project Structure

```
src/
├── components/
│   ├── layout/          # Layout components
│   ├── ui/              # Reusable UI components
│   └── ProtectedRoute.tsx
├── pages/               # Page components
├── store/               # Zustand stores
├── lib/                 # Utilities and API client
├── App.tsx              # Main app component
└── main.tsx             # Entry point
```

## Available Scripts

- `npm run dev` - Start development server
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm run lint` - Run ESLint

## Features Overview

### Authentication
- Login/logout with JWT tokens
- Protected routes with tier-based access
- Persistent authentication state

### Chat Interface
- Real-time messaging with WebSocket
- Markdown rendering for AI responses
- Jailbreak mode toggle for Ultra tier users
- Message history and chat management

### Dashboard
- System metrics and statistics
- Quick action buttons
- Recent activity feed
- Responsive grid layout

### Admin Panel
- User management with search and filters
- System monitoring and metrics
- Tier-based access control
- Security alerts and actions

### Tool Explorer
- Searchable and filterable tools list
- Category-based organization
- Usage statistics and status tracking
- Configuration and management

## Design System

The application uses a custom dark theme with neon accents:
- **Primary Colors**: Dark backgrounds with neon purple/blue accents
- **Typography**: Inter font with JetBrains Mono for code
- **Components**: Glass morphism effects and neon borders
- **Responsive**: Mobile-first design with breakpoints

## API Integration

The frontend integrates with the MR.DarkPromth backend API:
- Authentication endpoints
- Chat and messaging
- Jailbreak prompt management
- Admin and user management
- Real-time WebSocket updates

## Contributing

1. Follow the existing code style and patterns
2. Use TypeScript for all new code
3. Ensure responsive design
4. Test on different screen sizes
5. Update documentation as needed

## License

This project is part of the MR.DarkPromth ecosystem.
