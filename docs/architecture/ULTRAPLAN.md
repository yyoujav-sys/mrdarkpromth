# MR.Darkpromth Ultraplan: System Restructuring & Scalability Roadmap

## 1. วิสัยทัศน์ (Vision)
ยกระดับ **MR.Darkpromth** จากระบบ Agentic AI สู่การเป็น **Enterprise-Grade AI Framework** ที่มีความเสถียรสูง (High Availability), ปลอดภัย (Security-First), และรองรับการขยายตัว (Scalability) ทั้งในรูปแบบ VS Code Extension และ Public API ภายใต้แบรนด์เดียว

---

## 2. สถาปัตยกรรมระบบใหม่ (Target Architecture)

ระบบจะถูกปรับเปลี่ยนเป็นสถาปัตยกรรมแบบ **Modular Monolith** เพื่อลดความซ้ำซ้อนแต่ยังคงความง่ายในการจัดการ:

| Layer | Component | Responsibility |
| :--- | :--- | :--- |
| **Interface** | VS Code Extension / Web UI | การปฏิสัมพันธ์กับผู้ใช้ (User Experience) |
| **Gateway** | Unified API (Axum/Actix) | การจัดการ Request, Auth, และ Rate Limiting |
| **Core Logic** | Agentic Engine (Rust) | **[Core Logic - ห้ามเปลี่ยน]** การประมวลผล AI, State Machine |
| **Services** | Integration Services | การเชื่อมต่อภายนอก (GitHub, Cerebras, Redis) |
| **Data** | PostgreSQL / Redis | การจัดเก็บข้อมูลถาวรและ Cache |

---

## 3. แผนการดำเนินงาน (Roadmap)

### ระยะที่ 1: การปรับโครงสร้างพื้นฐาน (Foundation Refactoring)
*   **Unified Configuration**: รวมระบบ Config จากหลายไฟล์ (toml, env) ให้เป็นระบบเดียวที่จัดการผ่าน `mr_darkpromth/core/config`
*   **Error Handling Standardization**: ใช้ `thiserror` และ `anyhow` ทั่วทั้งโปรเจกต์เพื่อการ Debug ที่แม่นยำ
*   **Logging & Observability**: ติดตั้ง `tracing` พร้อมระบบหมุนเวียน Log (Log Rotation) ที่เสถียร

### ระยะที่ 2: การเสริมความแกร่งของ API & Security
*   **API Versioning**: รองรับ `/v1/` เพื่อความเสถียรของ Extension และ API ภายนอก
*   **Enhanced Auth**: ปรับปรุงระบบ JWT และ GitHub OAuth ให้รองรับ Session Persistence ที่ดีขึ้น
*   **Rate Limiting**: ย้าย Logic การจำกัดการใช้งานไปไว้ที่ระดับ Middleware เพื่อลดภาระของ Core Engine

### ระยะที่ 3: การขยายขีดความสามารถ (Extension & API Integration)
*   **Shared SDK**: สร้าง Library กลาง (TypeScript) ที่ใช้ร่วมกันระหว่าง Frontend และ VS Code Extension
*   **Documentation**: สร้าง Swagger/OpenAPI Spec อัตโนมัติผ่าน `utoipa`
*   **CI/CD Pipeline**: ระบบทดสอบอัตโนมัติก่อนการ Deploy เพื่อป้องกัน Regression

---

## 4. หลักการรักษา Core Logic (Core Preservation Principles)
1.  **No Logic Modification**: ห้ามแก้ไขอัลกอริทึมใน `agent.rs`, `state_machine.rs` และ `coordinator.rs`
2.  **Interface Only**: การปรับปรุงจะทำเฉพาะส่วน Input/Output และการจัดการ Error รอบนอกเท่านั้น
3.  **Regression Testing**: ทุกการเปลี่ยนแปลงต้องผ่านการทดสอบว่าผลลัพธ์จาก Core Engine ยังคงเดิม

---

## 5. ดัชนีชี้วัดความสำเร็จ (KPIs)
*   **Stability**: อัตราการเกิด Error (5xx) ลดลง 80%
*   **Performance**: Response Time ของ API เฉลี่ยลดลง 30% ผ่านการทำ Redis Caching
*   **Usability**: นักพัฒนาภายนอกสามารถเชื่อมต่อ API ได้ภายใน 5 นาทีผ่าน Documentation ใหม่
