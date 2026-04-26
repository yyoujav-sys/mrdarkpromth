# MR.Darkpromth Ultraplan V2: Multi-Dimensional System Optimization

## 1. บทวิเคราะห์สถานะปัจจุบัน (Current State Audit)
จากการตรวจสอบ Codebase พบปัญหาความซ้ำซ้อนเชิงโครงสร้าง (Structural Redundancy) ดังนี้:
*   **Logic Duplication**: มีการนิยาม Logic เดียวกันซ้ำใน `core` และ `services` (เช่น Audit, Host Protection, Jailbreak) ทำให้การแก้ไขทำได้ยากและเสี่ยงต่อความไม่สอดคล้อง
*   **Dependency Fragmentation**: การใช้ทั้ง `axum` และ `actix-web` ใน Workspace เดียวกันเพิ่มขนาด Binary และความซับซ้อนในการจัดการ Middleware
*   **Interface Inconsistency**: Frontend และ VS Code Extension มีการเขียน API Client แยกกันโดยไม่มี Shared SDK ทำให้ Type Safety ไม่ครอบคลุมทั้งระบบ

---

## 2. ยุทธศาสตร์การปรับปรุง 4 มิติ (4-Dimensional Strategy)

### มิติที่ 1: การรวมศูนย์ Logic (Logic Centralization)
*   **Single Source of Truth**: ย้าย Core Logic ทั้งหมดไปไว้ที่ `mr_darkpromth/core` และให้ `services` ทำหน้าที่เป็นเพียง Wrapper หรือ Implementation เฉพาะทางเท่านั้น
*   **Unified Error System**: สร้าง Global Error Enum ที่ครอบคลุมทั้งระบบ เพื่อให้การจัดการ Error จาก dLNk AI Gateway เป็นไปในทิศทางเดียวกัน

### มิติที่ 2: การเพิ่มประสิทธิภาพเครือข่าย (Network & Intelligence)
*   **dLNk AI Gateway Native**: ปรับปรุง `api_key_manager.rs` ให้รองรับ dLNk เป็น First-class Citizen พร้อมระบบ Smart Routing `auto`
*   **Connection Pooling**: ปรับปรุงการเชื่อมต่อ Redis และ PostgreSQL ให้เสถียรขึ้นด้วยระบบ Reconnection อัตโนมัติ

### มิติที่ 3: การพัฒนา Interface (Unified Interface)
*   **MR.Darkpromth SDK (TypeScript)**: สร้างโฟลเดอร์ `shared/sdk` เพื่อเก็บ API Client และ Types ที่ใช้ร่วมกันระหว่าง Frontend และ Extension
*   **Brand Identity Enforcement**: ปรับปรุง UI/UX ให้สะท้อนความเป็น MR.Darkpromth ที่มีความเป็นมืออาชีพและดุดัน (Dark Theme Optimized)

### มิติที่ 4: ความปลอดภัยและความเสถียร (Security & Stability)
*   **Enhanced Sandbox**: ปรับปรุง `host_protection.rs` ให้รองรับการตรวจสอบ Domain ของ dLNk AI Gateway โดยเฉพาะ
*   **Automated Audit**: ระบบบันทึก Log การใช้งาน AI ที่ละเอียดขึ้นเพื่อการตรวจสอบย้อนหลัง (Compliance)

---

## 3. แผนการดำเนินงาน (Execution Roadmap)

### ระยะที่ 1: การทำความสะอาด (The Great Cleanup - 1-2 สัปดาห์)
*   ลบโค้ดที่ซ้ำซ้อนใน `services` และเปลี่ยนไปเรียกใช้จาก `core`
*   รวมระบบ Configuration ให้เป็นหนึ่งเดียว (Unified Config)
*   อัปเดต `Cargo.toml` เพื่อลด Dependency ที่ไม่จำเป็น

### ระยะที่ 2: การเชื่อมต่ออัจฉริยะ (Smart Integration - 1 สัปดาห์)
*   Implement dLNk AI Gateway Wrapper พร้อมระบบ Fallback
*   สร้างระบบ Monitoring เบื้องต้นเพื่อดูสถานะของ Gateway

### ระยะที่ 3: การสร้าง Ecosystem (Ecosystem Building - 2 สัปดาห์)
*   พัฒนา Shared SDK สำหรับ TypeScript
*   ปรับปรุง VS Code Extension ให้รองรับฟีเจอร์ใหม่จาก dLNk

---

## 4. มาตรฐานการตรวจสอบ (Audit Standards)
*   **Zero Duplication**: ห้ามมี Logic ซ้ำซ้อนเกิน 5% ของ Codebase
*   **Type Coverage**: 100% Type Safety ในส่วนการสื่อสารระหว่าง API และ Client
*   **Performance**: API Response Time (Time to First Token) ต้องไม่เกิน 500ms เมื่อใช้ dLNk `auto`
