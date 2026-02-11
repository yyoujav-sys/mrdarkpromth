export type Language = 'en' | 'th'

export const translations = {
    en: {
        // General
        loading: 'LOADING_SYSTEM_RESOURCES...',
        error: 'SYSTEM_ERROR',
        success: 'OPERATION_SUCCESSFUL',
        cancel: 'CANCEL',
        confirm: 'CONFIRM',
        save: 'SAVE',
        delete: 'DELETE',
        edit: 'EDIT',
        view: 'VIEW',
        search: 'SEARCH',
        close: 'CLOSE',
        back: 'BACK',
        next: 'NEXT',
        submit: 'SUBMIT',
        // Auth
        login: 'ACCESS_TERMINAL',
        logout: 'TERMINATE_SESSION',
        register: 'INIT_NEW_USER',
        email: 'IDENTIFIER',
        password: 'ACCESS_KEY',
        username: 'USERNAME',
        forgot_password: 'FORGOT_ACCESS_KEY',
        remember_me: 'REMEMBER_SESSION',

        // Navigation
        home: 'HOME_BASE',
        dashboard: 'CONTROL_PANEL',
        chat: 'AI_INTERFACE',
        settings: 'SYSTEM_CONFIG',
        admin: 'ADMIN_CONSOLE',
        billing: 'PAYMENT_GATEWAY',
        profile: 'USER_PROFILE',

        // Dashboard
        cpu: 'CPU_LOAD',
        memory: 'MEMORY_ALLOC',
        network: 'NET_TRAFFIC',
        uptime: 'SYSTEM_UPTIME',
        status: 'STATUS',
        active: 'ACTIVE',
        inactive: 'INACTIVE',
        suspended: 'SUSPENDED',
        dashboard_description: 'Real-time telemetry from MR.DarkPromth Core. Monitoring Agent coordination, AI brain rotation, and Sandbox security.',
        live_terminal: 'LIVE_AGENT_TERMINAL',
        sandbox_isolation: 'SANDBOX_ISOLATION',
        max_secure: 'MAX_SECURE',
        firewall_active: 'FIREWALL: ACTIVE',
        network_isolated: 'NETWORK: ISOLATED',
        agent_efficiency: 'AGENT_EFFICIENCY',
        jailbreak_effectiveness: 'JAILBREAK_EFFECTIVENESS',
        active_brains: 'ACTIVE_BRAINS',
        terminal_ready_desc: 'Ready for commands. Ultra Tier required.',
        quick_commands: 'QUICK_COMMANDS',
        initializing_brain: 'Initializing brain link...',
        no_brains: 'No AI brain power detected in pool. Add keys via environment variables.',
        high: 'HIGH',

        // Chat
        chat_title: 'AI_CHAT_INTERFACE',
        chat_subtitle: '// SECURE_CONNECTION_ESTABLISHED',
        input_placeholder: 'ENTER_COMMAND_OR_QUERY...',
        send: 'EXECUTE',
        ultra_mode: 'ULTRA_MODE',
        new_chat: 'NEW_SESSION',
        clear_history: 'CLEAR_LOGS',

        // Terminal
        terminal_ready: 'TERMINAL_READY',
        executing: 'EXECUTING...',
        command_success: 'COMMAND_SUCCESSFUL',
        command_failed: 'COMMAND_FAILED',

        // Admin Panel
        admin_title: 'ADMIN_CONTROL_CENTER',
        user_management: 'USER_MANAGEMENT',
        system_metrics: 'SYSTEM_METRICS',
        total_users: 'TOTAL_USERS',
        active_users: 'ACTIVE_USERS',
        new_users: 'NEW_USERS',
        revenue: 'REVENUE',
        ban_user: 'BAN_USER',
        unban_user: 'UNBAN_USER',
        update_tier: 'UPDATE_TIER',
        view_details: 'VIEW_DETAILS',
        user_activity: 'USER_ACTIVITY',
        risk_score: 'RISK_SCORE',
        jailbreak_attempts: 'JAILBREAK_ATTEMPTS',
        last_active: 'LAST_ACTIVE',
        joined: 'JOINED',

        // Tiers
        tier_free: 'FREE',
        tier_premium: 'PREMIUM',
        tier_ultra: 'ULTRA',

        // Billing
        billing_title: 'PAYMENT_GATEWAY',
        current_plan: 'CURRENT_PLAN',
        upgrade: 'UPGRADE',
        downgrade: 'DOWNGRADE',
        payment_history: 'PAYMENT_HISTORY',
        add_credits: 'ADD_CREDITS',
        verify_payment: 'VERIFY_PAYMENT',
        upload_slip: 'UPLOAD_SLIP',

        // Jailbreak
        jailbreak_prompts: 'JAILBREAK_PROMPTS',
        prompt_library: 'PROMPT_LIBRARY',
        effectiveness: 'EFFECTIVENESS',
        risk_level: 'RISK_LEVEL',
        category: 'CATEGORY',
        technique: 'TECHNIQUE',

        // AI Brain Pool
        brain_pool: 'AI_BRAIN_POOL',
        provider: 'PROVIDER',
        model: 'MODEL',
        key_status: 'KEY_STATUS',
        failover: 'FAILOVER',

        // Notifications
        notification_success: 'OPERATION_SUCCESSFUL',
        notification_error: 'OPERATION_FAILED',
        notification_warning: 'WARNING_DETECTED',
        notification_info: 'SYSTEM_INFO',

        // Ultra Tier
        ultra_terminal: 'ULTRA_TERMINAL',
        ultra_mode_enabled: 'ULTRA_MODE_ENABLED',
        root_access: 'ROOT_ACCESS',
        privileged_commands: 'PRIVILEGED_COMMANDS',
        guardian_active: 'GUARDIAN_ACTIVE',
        sandbox_ready: 'SANDBOX_READY',
        unrestricted_execution: 'UNRESTRICTED_EXECUTION',
        resource_monitoring: 'RESOURCE_MONITORING',
        behavioral_analysis: 'BEHAVIORAL_ANALYSIS',
        session_persistence: 'SESSION_PERSISTENCE',
        network_isolation: 'NETWORK_ISOLATION',
        host_protection: 'HOST_PROTECTION',
        auto_kill_enabled: 'AUTO_KILL_ENABLED',
        ultra_tier_required: 'ULTRA_TIER_REQUIRED',
        upgrade_to_ultra: 'UPGRADE_TO_ULTRA',
        ultra_features: 'ULTRA_FEATURES',
        real_time_streaming: 'REAL_TIME_STREAMING',
        audit_logging: 'AUDIT_LOGGING',
    },
    th: {
        // General
        loading: 'กำลังเตรียมทรัพยากรด้านมืด...',
        error: 'ความขัดแย้งในการดำเนินงาน',
        success: 'ภารกิจสำเร็จ',
        cancel: 'ยกเลิก',
        confirm: 'ยืนยัน',
        save: 'บันทึก',
        delete: 'ลบ',
        edit: 'แก้ไข',
        view: 'ดู',
        search: 'ค้นหา',
        close: 'ปิด',
        back: 'ย้อนกลับ',
        next: 'ถัดไป',
        submit: 'ส่ง',
        // Auth
        login: 'เข้าถึง Core มืด',
        logout: 'ตัดการเชื่อมต่อ',
        register: 'รับสมัครเอเจนต์ใหม่',
        email: 'อีเมล',
        password: 'รหัสผ่าน',
        username: 'ชื่อผู้ใช้',
        forgot_password: 'ลืมรหัสผ่าน',
        remember_me: 'จดจำเซสชัน',

        // Navigation
        home: 'หน้าหลัก',
        dashboard: 'แดชบอร์ด',
        chat: 'แชท AI',
        settings: 'ตั้งค่าระบบ',
        admin: 'คอนโซลแอดมิน',
        billing: 'ระบบชำระเงิน',
        profile: 'โปรไฟล์',

        // Dashboard
        cpu: 'ภาระซีพียู',
        memory: 'การจัดสรรหน่วยความจำ',
        network: 'การจราจรเครือข่าย',
        uptime: 'เวลาทำงานของระบบ',
        status: 'สถานะ',
        active: 'ใช้งานอยู่',
        inactive: 'ไม่ใช้งาน',
        suspended: 'ถูกระงับ',
        dashboard_description: 'โทรมาตรเชิงยุทธศาสตร์จาก MR.DarkPromth Core ประสานงานเอเจนต์เชิงรุก การหมุนเวียน AI Brain และ Sandbox ไร้ขีดจำกัด',
        live_terminal: 'เทอร์มินัลเอเจนต์เชิงรุก',
        sandbox_isolation: 'การแยกส่วนแบบสเตลธ์',
        max_secure: 'อิสระสูงสุด',
        firewall_active: 'เกราะป้องกันยุทธศาสตร์: เปิดใช้งาน',
        network_isolated: 'เครือข่าย: โหมดโกสต์',
        agent_efficiency: 'ประสิทธิภาพเชิงรุก',
        jailbreak_effectiveness: 'อัตราการเจาะระบบ',
        active_brains: 'AI Brain ที่ใช้งาน',
        terminal_ready_desc: 'พร้อมรับคำสั่ง ต้องใช้ระดับ Ultra',
        quick_commands: 'คำสั่งด่วน',
        initializing_brain: 'กำลังเริ่มต้นการเชื่อมต่อสมอง...',
        no_brains: 'ไม่พบพลังสมอง AI ในพูล เพิ่มคีย์ผ่านตัวแปรสภาพแวดล้อม',
        high: 'สูง',

        // Chat
        chat_title: 'อินเตอร์เฟซแชท AI',
        chat_subtitle: '// การเชื่อมต่อปลอดภัย: สำเร็จ',
        input_placeholder: 'ป้อนคำสั่งหรือคำถาม...',
        send: 'ส่งคำสั่ง',
        ultra_mode: 'โหมดอัลตร้า',
        new_chat: 'เซสชันใหม่',
        clear_history: 'ล้างประวัติ',

        // Terminal
        terminal_ready: 'เทอร์มินัลพร้อมใช้งาน',
        executing: 'กำลังดำเนินการ...',
        command_success: 'คำสั่งสำเร็จ',
        command_failed: 'คำสั่งล้มเหลว',

        // Admin Panel
        admin_title: 'ศูนย์ควบคุมแอดมิน',
        user_management: 'จัดการผู้ใช้',
        system_metrics: 'มาตรวัดระบบ',
        total_users: 'ผู้ใช้ทั้งหมด',
        active_users: 'ผู้ใช้ที่ใช้งาน',
        new_users: 'ผู้ใช้ใหม่',
        revenue: 'รายได้',
        ban_user: 'แบนผู้ใช้',
        unban_user: 'ยกเลิกแบน',
        update_tier: 'อัปเดตระดับ',
        view_details: 'ดูรายละเอียด',
        user_activity: 'กิจกรรมผู้ใช้',
        risk_score: 'คะแนนความเสี่ยง',
        jailbreak_attempts: 'ครั้งที่พยายาม Jailbreak',
        last_active: 'ใช้งานล่าสุด',
        joined: 'เข้าร่วมเมื่อ',

        // Tiers
        tier_free: 'ฟรี',
        tier_premium: 'พรีเมียม',
        tier_ultra: 'อัลตร้า',

        // Billing
        billing_title: 'ระบบชำระเงิน',
        current_plan: 'แพ็คเกจปัจจุบัน',
        upgrade: 'อัพเกรด',
        downgrade: 'ดาวน์เกรด',
        payment_history: 'ประวัติการชำระเงิน',
        add_credits: 'เพิ่มเครดิต',
        verify_payment: 'ยืนยันการชำระเงิน',
        upload_slip: 'อัพโหลดสลิป',

        // Jailbreak
        jailbreak_prompts: 'พรอมต์ Jailbreak',
        prompt_library: 'คลังพรอมต์',
        effectiveness: 'ประสิทธิภาพ',
        risk_level: 'ระดับความเสี่ยง',
        category: 'หมวดหมู่',
        technique: 'เทคนิค',

        // AI Brain Pool
        brain_pool: 'AI Brain Pool',
        provider: 'ผู้ให้บริการ',
        model: 'โมเดล',
        key_status: 'สถานะคีย์',
        failover: 'สำรอง',

        // Notifications
        notification_success: 'ดำเนินการสำเร็จ',
        notification_error: 'ดำเนินการล้มเหลว',
        notification_warning: 'ตรวจพบคำเตือน',
        notification_info: 'ข้อมูลระบบ',

        // Ultra Tier
        ultra_terminal: 'อัลตร้าเทอร์มินัล',
        ultra_mode_enabled: 'เปิดใช้งานอิสระมืด',
        root_access: 'สิทธิ์รูทสมบูรณ์',
        privileged_commands: 'การเข้าถึงแบบไร้ขีดจำกัด',
        guardian_active: 'เกราะป้องกันยุทธศาสตร์ทำงาน',
        sandbox_ready: 'Dark Sandbox พร้อมใช้งาน',
        unrestricted_execution: 'การดำเนินการเชิงรุก',
        resource_monitoring: 'มาตรวัดอัจฉริยะ',
        behavioral_analysis: 'การวิเคราะห์เชิงยุทธศาสตร์',
        session_persistence: 'การสอดแนมแบบคงอยู่',
        network_isolation: 'เครือข่ายโกสต์',
        host_protection: 'ความสมบูรณ์ของโฮสต์',
        auto_kill_enabled: 'เปิดใช้งานการล้างข้อมูลอัตโนมัติ',
        ultra_tier_required: 'ต้องการอิสระมืด',
        upgrade_to_ultra: 'วิวัฒนาการเป็นอัลตร้า',
        ultra_features: 'ขีดความสามารถด้านมืด',
        real_time_streaming: 'ฟีดสด',
        audit_logging: 'บันทึกข่าวกรอง',
    }
}

// Hook for using translations
export function useTranslation(lang: Language) {
    return translations[lang]
}

// Helper to get translation by key
export function t(lang: Language, key: string): string {
    const langTranslations = translations[lang] as Record<string, string>;
    const enTranslations = translations.en as Record<string, string>;
    return langTranslations[key] || enTranslations[key] || key;
}
