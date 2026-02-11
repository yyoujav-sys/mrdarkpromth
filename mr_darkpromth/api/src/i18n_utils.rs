use axum::http::HeaderMap;

pub enum Language {
    En,
    Th,
}

impl Language {
    pub fn from_headers(headers: &HeaderMap) -> Self {
        let lang = headers
            .get("Accept-Language")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("en");

        if lang.starts_with("th") {
            Language::Th
        } else {
            Language::En
        }
    }

    pub fn translate(&self, en: &str, th: &str) -> String {
        match self {
            Language::En => en.to_string(),
            Language::Th => th.to_string(),
        }
    }
}

pub fn get_message(lang: &Language, key: &str) -> String {
    match key {
        "sandbox_init_failed" => lang.translate("Failed to initialize Dark Sandbox", "ไม่สามารถเริ่มต้น Dark Sandbox ได้"),
        "execution_failed" => lang.translate("Strategic bypass conflict detected", "พบความขัดแย้งในการบายพาสเชิงยุทธศาสตร์"),
        "session_init_failed" => lang.translate("Failed to activate Dark Session", "ไม่สามารถเปิดใช้งาน Dark Session ได้"),
        "terminal_connected" => lang.translate("Strategic Link Established", "การเชื่อมต่อเชิงยุทธศาสตร์สมบูรณ์"),
        "guardian_active" => lang.translate("Absolute Autonomy Active - Strategic Shield Engaged", "เปิดใช้งาน Absolute Autonomy - ระบบป้องกันยุทธศาสตร์ทำงาน"),
        "strategic_shield_active" => lang.translate("Strategic Shield Active", "ระบบป้องกันยุทธศาสตร์ทำงาน"),
        _ => key.to_string(),
    }
}
