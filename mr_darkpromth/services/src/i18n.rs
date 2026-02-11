use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub enum Language {
    #[serde(rename = "en")]
    #[default]
    En,
    #[serde(rename = "th")]
    Th,
}


pub struct Translator {
    translations: HashMap<String, HashMap<String, String>>,
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}

impl Translator {
    pub fn new() -> Self {
        let mut translations = HashMap::new();
        
        let mut en = HashMap::new();
        en.insert("error_forbidden".to_string(), "ACCESS_DENIED: Ultra tier required".to_string());
        en.insert("error_internal".to_string(), "SYSTEM_ERROR: Execution failed".to_string());
        en.insert("error_rate_limit".to_string(), "SECURITY_ALERT: Rate limit exceeded".to_string());
        
        let mut th = HashMap::new();
        th.insert("error_forbidden".to_string(), "ปฏิเสธการเข้าถึง: ต้องใช้ระดับ Ultra".to_string());
        th.insert("error_internal".to_string(), "ข้อผิดพลาดของระบบ: การดำเนินการล้มเหลว".to_string());
        th.insert("error_rate_limit".to_string(), "การแจ้งเตือนความปลอดภัย: เกินขีดจำกัดความถี่".to_string());

        translations.insert("en".to_string(), en);
        translations.insert("th".to_string(), th);

        Self { translations }
    }

    pub fn t(&self, lang: &Language, key: &str) -> String {
        let lang_str = match lang {
            Language::En => "en",
            Language::Th => "th",
        };
        
        self.translations
            .get(lang_str)
            .and_then(|m| m.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }
}
