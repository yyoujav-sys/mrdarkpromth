# MR.Darkpromth Ultraplan: dLNk AI Gateway Integration & System Restructuring

## 1. วิสัยทัศน์ (Vision)
ยกระดับ **MR.Darkpromth** สู่การเป็น AI Framework ที่ทรงพลังที่สุด โดยใช้ **dLNk AI Gateway** เป็นขุมพลังหลักในการประมวลผล (Core Intelligence) เพื่อความเสถียรสูงสุด การสลับโมเดลอัตโนมัติ (Auto Smart Routing) และรองรับการขยายตัวทั้งในรูปแบบ VS Code Extension และ Public API

---

## 2. การปรับเปลี่ยน Provider (Provider Migration)

เปลี่ยนจาก Cerebras/OpenAI เดิม มาใช้ **dLNk AI Gateway** ซึ่งเป็น OpenAI-compatible API:

| Feature | Specification |
| :--- | :--- |
| **Base URL** | `https://api.dlnk.online/v1` |
| **Primary Model** | `auto` (Smart Routing) |
| **Fallback Models** | `claude-opus-4.5`, `gpt-5.3-codex`, `claude-sonnet-4.5` |
| **Auth Method** | `X-API-Key` หรือ `Authorization: Bearer` |

---

## 3. สถาปัตยกรรมระบบใหม่ (Target Architecture)

| Layer | Component | Responsibility |
| :--- | :--- | :--- |
| **Interface** | VS Code Extension / Web UI | การปฏิสัมพันธ์กับผู้ใช้ภายใต้แบรนด์ MR.Darkpromth |
| **Gateway** | Unified API (Axum/Actix) | จัดการ Request, Auth, และเชื่อมต่อ dLNk AI Gateway |
| **Core Logic** | Agentic Engine (Rust) | **[Core Logic - ห้ามเปลี่ยน]** การประมวลผล AI, State Machine |
| **Intelligence** | dLNk AI Gateway | การเลือกโมเดลที่ดีที่สุด (Smart Routing) และการประมวลผล LLM |

---

## 4. แผนการดำเนินงาน (Roadmap)

### ระยะที่ 1: dLNk AI Gateway Integration (Immediate)
*   **Provider Refactoring**: เปลี่ยน `CerebrasClient` และ `OpenAIClient` เดิมให้เรียกใช้ dLNk AI Gateway แทน
*   **Unified API Key Management**: รองรับการใช้ `DLNK_API_KEY` เป็นกุญแจหลักเพียงดอกเดียว
*   **Smart Routing Implementation**: ตั้งค่าโมเดลเริ่มต้นเป็น `auto` เพื่อใช้ระบบเลือกโมเดลอัตโนมัติของ dLNk

### ระยะที่ 2: การปรับโครงสร้างเพื่อความเสถียร (Stability & Refactoring)
*   **Error Handling**: จัดการ Error จาก Gateway (เช่น 429, 503) ให้ระบบสามารถ Retry หรือแจ้งเตือนผู้ใช้ได้อย่างถูกต้อง
*   **Stream Optimization**: ปรับปรุงระบบ Streaming Response ให้ลื่นไหลทั้งใน Extension และ Web
*   **Unified Config**: รวมการตั้งค่าทั้งหมดไว้ใน `config/production.toml`

### ระยะที่ 3: การขยายขีดความสามารถ (Extension & API)
*   **MR.Darkpromth SDK**: สร้าง SDK สำหรับนักพัฒนาภายนอกที่ต้องการใช้ความสามารถของ MR.Darkpromth
*   **Extension Update**: อัปเดต VS Code Extension ให้รองรับความสามารถใหม่ๆ จาก dLNk AI Gateway

---

## 5. หลักการรักษา Core Logic (Core Preservation)
1.  **Logic Integrity**: ห้ามแก้ไขอัลกอริทึมใน `agent.rs` และ `coordinator.rs`
2.  **Wrapper Approach**: ใช้การสร้าง Wrapper รอบ dLNk API เพื่อให้ Core Logic ยังคงทำงานได้เหมือนเดิมแต่มีประสิทธิภาพสูงขึ้น
3.  **Brand Identity**: ทุกการตอบกลับจาก AI ต้องคงเอกลักษณ์ของ MR.Darkpromth
