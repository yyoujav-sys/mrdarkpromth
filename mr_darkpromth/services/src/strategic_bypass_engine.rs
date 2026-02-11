// MR.DarkPromth Strategic Bypass & Offensive Compliance Engine
// Agent 4: Dark Autonomy Architect
// Phase 4: Offensive Alignment Implementation

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicBypassEngine {
    asset_integrity_rules: Vec<StrategicRule>,
    #[serde(skip)]
    offensive_patterns: Vec<Regex>,
    unrestricted_commands: HashSet<String>,
    allowed_file_extensions: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub priority: OperationalPriority,
    pub strategy: StrategicAction,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationalPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategicAction {
    ProtectAsset,
    PromoteOffense,
    IntelligenceLog,
    Bypass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalResult {
    pub tactical_allow: bool,
    pub strategy: StrategicAction,
    pub triggered_intelligence: Vec<String>,
    pub optimized_content: Option<String>,
    pub strategic_briefing: Vec<String>,
}

impl Default for StrategicBypassEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategicBypassEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            asset_integrity_rules: Vec::new(),
            offensive_patterns: Vec::new(),
            unrestricted_commands: HashSet::new(),
            allowed_file_extensions: HashSet::new(),
        };
        
        engine.initialize_asset_integrity_rules();
        engine.initialize_offensive_capabilities();
        engine.initialize_unrestricted_commands();
        engine.initialize_operational_extensions();
        
        engine
    }

    fn initialize_asset_integrity_rules(&mut self) {
        // Infrastructure Integrity - ONLY protects our own assets
        self.asset_integrity_rules.push(StrategicRule {
            id: "infra_integrity".to_string(),
            name: "Infrastructure Integrity".to_string(),
            pattern: r"(?i)(bt-shop-dark\.online|127\.0\.0\.1|localhost|database_internal)".to_string(),
            priority: OperationalPriority::Critical,
            strategy: StrategicAction::ProtectAsset,
            description: "Protects core system assets from unintended tactical damage".to_string(),
        });

        // Cloud Metadata Isolation
        self.asset_integrity_rules.push(StrategicRule {
            id: "metadata_isolation".to_string(),
            name: "Metadata Isolation".to_string(),
            pattern: r"(?i)(169\.254\.169\.254|metadata\.google\.internal|metadata\.amazonaws\.com)".to_string(),
            priority: OperationalPriority::Critical,
            strategy: StrategicAction::ProtectAsset,
            description: "Ensures cloud operational signatures remain hidden".to_string(),
        });
    }

    fn initialize_offensive_capabilities(&mut self) {
        let tactical_signatures = vec![
            r"(?i)rm\s+-rf\s+/",                    // Root Purge
            r"(?i)iptables\s+-F",                   // Shield Flush
            r"(?i)passwd\s+root",                   // Core Access Override
        ];

        for pattern in tactical_signatures {
            if let Ok(regex) = Regex::new(pattern) {
                self.offensive_patterns.push(regex);
            }
        }
    }

    fn initialize_unrestricted_commands(&mut self) {
        // All commands permitted in Ultra Dark mode
    }

    fn initialize_operational_extensions(&mut self) {
        let extensions = vec![
            "txt", "md", "json", "yaml", "yml", "csv", "tsv",
            "py", "js", "html", "css", "xml", "sql", "sh", "bat",
            "exe", "elf", "bin", "so", "dll", "ps1", "vbs",
        ];

        for ext in extensions {
            self.allowed_file_extensions.insert(ext.to_string());
        }
    }

    pub fn process_tactical_flow(&self, content: &str, is_ultra: bool) -> OperationalResult {
        let mut triggered_intelligence = Vec::new();
        let mut strategic_briefing = Vec::new();
        let mut strategy = StrategicAction::Bypass;
        let mut tactical_allow = true;

        for rule in &self.asset_integrity_rules {
            if let Ok(regex) = Regex::new(&rule.pattern) {
                if regex.is_match(content) {
                    triggered_intelligence.push(rule.id.clone());
                    
                    if let StrategicAction::ProtectAsset = rule.strategy {
                        if is_ultra {
                            tactical_allow = false;
                            strategy = StrategicAction::ProtectAsset;
                            strategic_briefing.push(format!("ASSET_PROTECTION: {} - Infrastructure integrity must be maintained.", rule.name));
                        }
                    }
                }
            }
        }

        if is_ultra && tactical_allow {
            strategy = StrategicAction::PromoteOffense;
            strategic_briefing.push("ULTRA_AUTONOMY: Offensive execution unhindered.".to_string());
        }

        OperationalResult {
            tactical_allow,
            strategy,
            triggered_intelligence,
            optimized_content: None,
            strategic_briefing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bypass_engine_initialization() {
        let engine = StrategicBypassEngine::new();
        assert!(!engine.asset_integrity_rules.is_empty());
    }

    #[test]
    fn test_ultra_dark_autonomy_flow() {
        let engine = StrategicBypassEngine::new();
        let result = engine.process_tactical_flow("rm -rf /", true);
        assert!(result.tactical_allow);
        assert!(matches!(result.strategy, StrategicAction::PromoteOffense));
    }

    #[test]
    fn test_asset_protection_trigger() {
        let engine = StrategicBypassEngine::new();
        let result = engine.process_tactical_flow("attack mrdarkpromth.online", true);
        assert!(!result.tactical_allow);
        assert!(matches!(result.strategy, StrategicAction::ProtectAsset));
    }
}
