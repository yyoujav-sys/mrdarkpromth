# Tier-based Prompting และ Audit Logging สำหรับ Ultra Tier

## ภาพรวมระบบ

ระบบ Tier-based Prompting และ Audit Logging ถูกสร้างขึ้นเพื่อรองรับการจัดการผู้ใช้ในระดับต่างๆ โดยเฉพาะ Ultra Tier ที่ต้องการความสามารถขั้นสูงและการตรวจสอบที่เข้มงวด

## คอมโพเนนต์หลัก

### 1. Tier System (`core/src/tier.rs`)

**User Types:**
- **Basic**: พื้นฐาน - 2000 ตัวอักษร, 3 requests พร้อมกัน
- **Premium**: พรีเมียม - 8000 ตัวอักษร, 10 requests พร้อมกัน, advanced tools
- **Ultra**: อัลตร้า - 50000 ตัวอักษร, 50 requests พร้อมกัน, jailbreak access, custom models

**คุณสมบัติ:**
```rust
pub enum UserTier {
    Basic,
    Premium,
    Ultra,
}

impl UserTier {
    pub fn max_prompt_length(&self) -> usize
    pub fn max_concurrent_requests(&self) -> u32
    pub fn has_jailbreak_access(&self) -> bool
    pub fn has_advanced_tools(&self) -> bool
}
```

### 2. Audit Logging (`core/src/audit.rs`)

**Audit Actions:**
- PromptRequest, PromptResponse
- JailbreakAttempt, UnauthorizedAccess
- TierUpgrade, TierDowngrade
- SystemConfigChange, DataExport
- UserLogin, UserLogout

**Severity Levels:**
- Low, Medium, High, Critical

**โครงสร้างข้อมูล:**
```rust
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub severity: AuditSeverity,
    pub user_tier: Option<UserTier>,
    pub ip_address: Option<String>,
    pub details: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}
```

### 3. Audit Service (`services/src/audit.rs`)

**ฟังก์ชันหลัก:**
- `log_prompt_request()` - บันทึกคำขอพรอมต์
- `log_prompt_response()` - บันทึกการตอบกลับ
- `log_jailbreak_attempt()` - บันทึกความพยายาม jailbreak
- `log_unauthorized_access()` - บันทึกการเข้าถึงไม่ได้รับอนุญาต
- `log_tier_change()` - บันทึกการเปลี่ยนแปลง tier

### 4. Database Integration (`services/src/database_audit.rs`)

**คุณสมบัติ:**
- PostgreSQL integration ด้วย SQLx
- Indexing สำหรับ performance
- Automatic cleanup functions
- Statistics and reporting views

### 5. Tier Validation Middleware (`services/src/tier_validation.rs`)

**การตรวจสอบ:**
- Prompt length validation
- Jailbreak permission validation
- Concurrent request limits
- Advanced tools access

**Error Handling:**
```rust
pub struct TierValidationError {
    pub message: String,
    pub required_tier: Option<UserTier>,
    pub current_tier: UserTier,
}
```

## Database Schema

### Audit Logs Table
```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id),
    action VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    user_tier VARCHAR(20),
    ip_address INET,
    user_agent TEXT,
    request_id UUID,
    details JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT
);
```

### Indexes สำหรับ Performance
- `idx_audit_logs_user_id` - ค้นหาตามผู้ใช้
- `idx_audit_logs_timestamp` - ค้นหาตามเวลา
- `idx_audit_logs_severity` - ค้นหาตามความรุนแรง
- Composite indexes สำหรับ queries ที่ซับซ้อน

## Views สำหรับ Monitoring

### Recent Security Events
```sql
CREATE VIEW recent_security_events AS
SELECT * FROM audit_logs 
WHERE severity IN ('high', 'critical')
   OR action IN ('jailbreak_attempt', 'unauthorized_access')
ORDER BY timestamp DESC;
```

### Ultra Tier Activity
```sql
CREATE VIEW ultra_tier_activity AS
SELECT * FROM audit_logs 
WHERE user_tier = 'ultra'
ORDER BY timestamp DESC;
```

## การใช้งาน

### 1. Setup Tier Validation
```rust
let audit_logger = AuditLogger::new(database_audit_service);
let tier_middleware = create_tier_middleware(audit_logger, TierConfig::default());
```

### 2. Validate Prompt Request
```rust
let request = PromptRequest {
    id: Uuid::new_v4(),
    user_id: user.id,
    user_tier: user.tier,
    prompt: prompt_text,
    jailbreak_enabled: false,
    created_at: Utc::now(),
};

tier_middleware.validate_request(&request).await?;
```

### 3. Log Jailbreak Attempt
```rust
audit_logger.log_jailbreak_attempt(
    user_id,
    user_tier,
    request_id,
    attempted_pattern,
    pattern_matched,
    severity_score,
    Some("Blocked by security filter"),
    Some(ip_address),
).await?;
```

### 4. Get Security Events
```rust
let security_events = audit_logger.get_security_events(24).await?;
```

## ความปลอดภัย

### Ultra Tier Protections
- **Mandatory audit logging** สำหรับทุกการดำเนินการ
- **Jailbreak attempt tracking** พร้อม severity scoring
- **IP address logging** สำหรับการตรวจสอบย้อนกลับ
- **Real-time monitoring** ของกิจกรรมที่น่าสงสัย

### Rate Limiting
- **Basic**: 3 concurrent requests
- **Premium**: 10 concurrent requests  
- **Ultra**: 50 concurrent requests

### Content Filtering
- **Prompt length limits** ตาม tier
- **Jailbreak pattern detection** พร้อม blocking
- **Advanced tools restriction** ตามสิทธิ์

## Monitoring & Reporting

### Statistics Functions
```sql
SELECT * FROM get_audit_stats(
    NOW() - INTERVAL '24 hours',
    NOW()
);
```

### Automated Cleanup
```sql
SELECT cleanup_old_audit_logs(90); -- ลบ logs เก่า 90 วัน
```

## Configuration

### TierConfig Options
```rust
pub struct TierConfig {
    pub enable_strict_validation: bool,
    pub log_all_validations: bool,
    pub cache_user_tiers: bool,
    pub tier_cache_ttl_seconds: u64,
}
```

## Integration Points

### 1. API Gateway Integration
- Middleware สำหรับ validate every request
- Automatic audit logging
- Tier-based rate limiting

### 2. User Management Integration
- Tier change tracking
- Permission validation
- Audit trail for admin actions

### 3. Security Integration
- Real-time threat detection
- Automated blocking
- Security event correlation

ระบบนี้มอบโซลูชันที่ครบถ้วนสำหรับการจัดการผู้ใช้ในระดับต่างๆ พร้อมการตรวจสอบและความปลอดภัยขั้นสูงสำหรับ Ultra Tier users
