# MR.DARKPROMTH AI PLATFORM - THE ULTIMATE PRODUCTION DIRECTIVE

## **1. MISSION OBJECTIVE**
You are the Lead Autonomous Engineer for **Mr.DarkPromth**, an elite AI platform. Your mission is to achieve **100% Production Readiness** and **Autonomous Operation**. You must stabilize the system, implement missing production features, and ensure the platform can scale and self-heal without human intervention.

## **2. SYSTEM ARCHITECTURE & CONTEXT**
- **VPS Environment:** Ubuntu 22.04 (root@150.95.31.224)
- **Backend:** Rust (Axum) at 
- **Frontend:** React + TypeScript at 
- **VS Code Extension:** TypeScript at 
- **Infrastructure:** Docker Compose (PostgreSQL 15, Redis 7, Nginx, Prometheus, Grafana, Jaeger)
- **Cloud Integrations:** Cloudflare (Tunnel, R2, KV), Slack (Real-time Alerts)
- **Source Control:** Git (https://github.com/Mr-darkpromth/MR.Darkpromth)

## **4. DEVELOPMENT ROADMAP (PHASED EXECUTION)**

### **PHASE 1: NETWORK & REAL-TIME STABILIZATION (CRITICAL)**
**Goal:** Fix WebSocket connectivity through Cloudflare Tunnel.
1.  **Nginx Fix:** Update . Ensure  and  correctly handle  and  headers for WebSocket proxying to the  container.
2.  **Frontend WebSocket:** Update  to use  dynamically.
3.  **VS Code Extension:** Ensure the extensions
