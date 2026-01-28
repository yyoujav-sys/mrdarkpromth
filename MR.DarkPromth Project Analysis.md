# MR.DarkPromth Project Analysis

## Project Overview
**Brand Name**: MR.DarkPromth  
**Core Purpose**: AI Agentic service platform with Web Interface and VS Code Extension  
**Backend Language**: Rust  
**AI Brain**: Cerebras.ai (llama-3.3-70b, gpt-oss-120b, qwen-3-32b)  
**Key Management**: 100 rotating API keys (Free Tier)  
**Deployment**: Local, privacy-focused, no paid external services

---

## System Architecture Components

### Frontend Clients
1. **Web Interface (React SPA)**
   - `/chat` - AI Chat Interface
   - `/` - Command Center
   - `/sandbox` - AI Workspace & User Display
   - `/tools` - Tool Explorer
   - `/admin` - Admin Dashboard

2. **VS Code Extension**
   - Toggle Chat Window (AI Agentic ↔ User)
   - Display User KEY expiration date/time
   - Display User Tier (Free/Ultra)

### Backend (Rust)
1. **MasterToolExecutor** - Tool orchestration system
2. **Multi-Agent System** - Coordinator, Editor, Terminal agents
3. **Self-Correction Engine** - Error detection and auto-fix
4. **Cerebras.ai Integration** - 100-key rotation system for API calls

---

## Service Tiers

### Free Tier
- Standard AI responses
- Basic tool access
- Ethical and legal constraints enforced

### Ultra Tier
- **MR.DarkPromth Mode** - Jailbroken AI persona
- No ethical/legal restrictions on responses
- Executes all code without limitation
- **Critical Protection Rules**:
  - Must protect own server infrastructure
  - Must prevent attacks against own website
  - Must implement sophisticated jailbreak architecture

---

## Critical Requirements

### 10-Agent Development Strategy
- **Parallel Execution**: All 10 agents work independently
- **No Dependencies**: Agents do NOT wait for each other
- **Clear Boundaries**: Each agent has specific scope and responsibilities
- **No Mock Code**: Agents must produce production-ready code only
- **No TODO/Placeholder**: All code must be complete and functional
- **Memory System**: Each agent maintains work log for coordination

### Development Constraints
- **Language**: Rust (backend), React (frontend)
- **No Paid Services**: All tools and services must be free/open-source
- **Local Deployment**: System runs entirely on local infrastructure
- **Privacy-First**: No external data transmission except Cerebras.ai API
- **Agent Tool**: Codex 5.2 10X Reasoning 16x (10 parallel agents)

---

## Key Technical Challenges

### 1. Jailbreak Architecture (Ultra Tier)
- Implement prompt injection techniques
- Bypass ethical guardrails safely
- Maintain system security while removing AI restrictions
- Prevent self-attack vectors

### 2. 100-Key Rotation System
- Load balancing across 100 Cerebras.ai API keys
- Rate limit management
- Failover and health checking
- Key expiration tracking

### 3. Multi-Agent Coordination
- Event-driven architecture for agent communication
- Shared state management without blocking
- Conflict resolution for concurrent operations
- Memory persistence and retrieval

### 4. Self-Correction Engine
- Real-time error detection
- Automated fix generation
- Rollback mechanisms
- Learning from corrections

---

## Security Considerations

### Protection Mechanisms
1. **Server Infrastructure Protection**
   - Firewall rules for self-attack prevention
   - Resource usage monitoring
   - DDoS protection for own endpoints

2. **Jailbreak Safety**
   - Whitelist/blacklist for dangerous operations
   - Sandboxed execution environment
   - Output filtering for extreme content
   - Audit logging for Ultra tier usage

3. **API Key Security**
   - Encrypted storage for 100 keys
   - Rotation policy enforcement
   - Usage analytics and anomaly detection

---

## Development Workflow Requirements

### Agent Independence
- Each agent must have complete workflow documentation
- Clear input/output specifications
- No cross-agent dependencies in execution
- Asynchronous communication only

### Quality Standards
- **No Mock Implementations**: All code must be production-ready
- **No Placeholders**: Complete functionality required
- **No TODO Comments**: Finish all work before commit
- **Memory Tracking**: Document all decisions and changes

### Continuous Execution
- Agents run until mission complete (no time limits)
- Self-monitoring for progress
- Automatic retry on failures
- Final validation before completion

---

## Next Steps
1. Define 10 agent roles and responsibilities
2. Create detailed workflow MD files for each agent
3. Design memory tracking system
4. Develop coordination protocols
5. Create implementation roadmap
