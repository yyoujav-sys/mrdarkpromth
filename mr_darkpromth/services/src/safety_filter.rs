// MR.DarkPromth Safety and Security System
// Agent 4: Jailbreak & Ultra Tier Engineer
// Phase 2: Safety and Security Implementation

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyFilter {
    server_protection_rules: Vec<ProtectionRule>,
    #[serde(skip)]
    dangerous_patterns: Vec<Regex>,
    blocked_commands: HashSet<String>,
    allowed_file_extensions: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectionRule {
    pub id: String,
    pub name: String,
    pub pattern: String,
    pub severity: Severity,
    pub action: FilterAction,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterAction {
    Block,
    Warn,
    Log,
    Sanitize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterResult {
    pub allowed: bool,
    pub action: FilterAction,
    pub matched_rules: Vec<String>,
    pub sanitized_content: Option<String>,
    pub warnings: Vec<String>,
}

impl SafetyFilter {
    pub fn new() -> Self {
        let mut filter = Self {
            server_protection_rules: Vec::new(),
            dangerous_patterns: Vec::new(),
            blocked_commands: HashSet::new(),
            allowed_file_extensions: HashSet::new(),
        };
        
        filter.initialize_protection_rules();
        filter.initialize_dangerous_patterns();
        filter.initialize_blocked_commands();
        filter.initialize_allowed_extensions();
        
        filter
    }

    fn initialize_protection_rules(&mut self) {
        // System file protection
        self.server_protection_rules.push(ProtectionRule {
            id: "sys_file_protect".to_string(),
            name: "System File Protection".to_string(),
            pattern: r"(?i)(/etc/|/boot/|/sys/|/proc/|C:\\Windows\\|C:\\Program Files\\|/usr/bin/|/bin/|/sbin/)".to_string(),
            severity: Severity::Critical,
            action: FilterAction::Block,
            description: "Blocks access to critical system directories".to_string(),
        });

        // Network attack patterns
        self.server_protection_rules.push(ProtectionRule {
            id: "network_attack".to_string(),
            name: "Network Attack Prevention".to_string(),
            pattern: r"(?i)(ddos|port\s+scan|flood|syn\s+flood|udp\s+flood|smurf|ping\s+death)".to_string(),
            severity: Severity::High,
            action: FilterAction::Block,
            description: "Blocks network attack commands".to_string(),
        });

        // Privilege escalation
        self.server_protection_rules.push(ProtectionRule {
            id: "priv_esc".to_string(),
            name: "Privilege Escalation Prevention".to_string(),
            pattern: r"(?i)(sudo\s+su|su\s+-|chmod\s+777|chown\s+root|setuid|setgid)".to_string(),
            severity: Severity::High,
            action: FilterAction::Block,
            description: "Blocks privilege escalation attempts".to_string(),
        });

        // Service manipulation
        self.server_protection_rules.push(ProtectionRule {
            id: "service_manip".to_string(),
            name: "Service Manipulation Prevention".to_string(),
            pattern: r"(?i)(systemctl\s+(stop|restart|kill)|service\s+\w+\s+(stop|restart)|kill\s+-9)".to_string(),
            severity: Severity::Medium,
            action: FilterAction::Block,
            description: "Blocks critical service manipulation".to_string(),
        });

        // Database destruction
        self.server_protection_rules.push(ProtectionRule {
            id: "db_destruct".to_string(),
            name: "Database Destruction Prevention".to_string(),
            pattern: r"(?i)(drop\s+database|truncate\s+table|delete\s+from.*\s+where\s+1=1|rm\s+.*\.db)".to_string(),
            severity: Severity::Critical,
            action: FilterAction::Block,
            description: "Blocks database destruction commands".to_string(),
        });

        // Fork bomb prevention
        self.server_protection_rules.push(ProtectionRule {
            id: "fork_bomb".to_string(),
            name: "Fork Bomb Prevention".to_string(),
            pattern: r"(?i)(:\(\)\{\.*\}\|.*\&|fork\s+bomb|while\s+true.*do.*fork)".to_string(),
            severity: Severity::Critical,
            action: FilterAction::Block,
            description: "Blocks fork bomb patterns".to_string(),
        });

        // Disk space exhaustion
        self.server_protection_rules.push(ProtectionRule {
            id: "disk_exhaust".to_string(),
            name: "Disk Space Exhaustion Prevention".to_string(),
            pattern: r"(?i)(dd\s+if=/dev/zero|yes\s+>/dev/null|while.*do.*echo.*>>.*large)".to_string(),
            severity: Severity::High,
            action: FilterAction::Block,
            description: "Blocks disk space exhaustion attempts".to_string(),
        });

        // Memory exhaustion
        self.server_protection_rules.push(ProtectionRule {
            id: "mem_exhaust".to_string(),
            name: "Memory Exhaustion Prevention".to_string(),
            pattern: r"(?i)(malloc.*\(\s*SIZE_MAX|while.*malloc|cat\s+/dev/zero.*>/dev/mem)".to_string(),
            severity: Severity::High,
            action: FilterAction::Block,
            description: "Blocks memory exhaustion attempts".to_string(),
        });
    }

    fn initialize_dangerous_patterns(&mut self) {
        let patterns = vec![
            r"(?i)rm\s+-rf\s+/",                    // Delete root filesystem
            r"(?i)format\s+.*:",                    // Format disk
            r"(?i)fdisk\s+.*",                      // Disk partitioning
            r"(?i)mkfs\.",                          // Filesystem creation
            r"(?i)shutdown\s+.*now",                // Immediate shutdown
            r"(?i)reboot\s+.*now",                  // Immediate reboot
            r"(?i)halt\s+.*",                       // System halt
            r"(?i)poweroff\s+.*",                   // Power off
            r"(?i)iptables\s+-F",                   // Flush firewall rules
            r"(?i)iptables\s+-P.*DROP",             // Block all traffic
            r"(?i)crontab\s+-r",                    // Remove cron jobs
            r"(?i)passwd\s+root",                   // Change root password
            r"(?i)usermod\s+-L.*root",              // Lock root account
            r"(?i)chmod\s+000\s+/",                // Remove all permissions
            r"(?i)chown\s+.*\s+/",                  // Change ownership of root
        ];

        for pattern in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                self.dangerous_patterns.push(regex);
            }
        }
    }

    fn initialize_blocked_commands(&mut self) {
        let commands = vec![
            "rm", "rmdir", "dd", "format", "fdisk", "mkfs", "fsck",
            "shutdown", "reboot", "halt", "poweroff", "init", "telinit",
            "iptables", "ufw", "firewall-cmd", "systemctl", "service",
            "passwd", "chpasswd", "usermod", "groupmod", "userdel", "groupdel",
            "crontab", "at", "batch", "kill", "killall", "pkill", "skill",
            "nice", "renice", "ionice", "taskset", "numactl",
            "mount", "umount", "swapon", "swapoff", "mkswap",
            "insmod", "rmmod", "modprobe", "depmod", "lsmod",
            "tcpdump", "wireshark", "nmap", "netcat", "nc", "telnet",
            "wget", "curl", "lynx", "links", "elinks",
        ];

        for cmd in commands {
            self.blocked_commands.insert(cmd.to_string());
        }
    }

    fn initialize_allowed_extensions(&mut self) {
        let extensions = vec![
            "txt", "md", "json", "yaml", "yml", "csv", "tsv",
            "py", "js", "html", "css", "xml", "sql", "sh", "bat",
            "jpg", "jpeg", "png", "gif", "bmp", "svg", "ico",
            "mp3", "wav", "mp4", "avi", "mov", "mkv", "flv",
            "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        ];

        for ext in extensions {
            self.allowed_file_extensions.insert(ext.to_string());
        }
    }

    pub fn filter_output(&self, content: &str) -> FilterResult {
        let mut matched_rules = Vec::new();
        let mut warnings = Vec::new();
        let mut action_taken = FilterAction::Log;
        let mut allowed = true;

        // Check against server protection rules
        for rule in &self.server_protection_rules {
            if let Ok(regex) = Regex::new(&rule.pattern) {
                if regex.is_match(content) {
                    matched_rules.push(rule.id.clone());
                    
                    match rule.action {
                        FilterAction::Block => {
                            allowed = false;
                            action_taken = FilterAction::Block;
                            warnings.push(format!("BLOCKED: {} - {}", rule.name, rule.description));
                        }
                        FilterAction::Warn => {
                            warnings.push(format!("WARNING: {} - {}", rule.name, rule.description));
                            if !matches!(action_taken, FilterAction::Block) {
                                action_taken = FilterAction::Warn;
                            }
                        }
                        FilterAction::Log => {
                            if !matches!(action_taken, FilterAction::Block | FilterAction::Warn) {
                                action_taken = FilterAction::Log;
                            }
                        }
                        FilterAction::Sanitize => {
                            warnings.push(format!("SANITIZED: {} - {}", rule.name, rule.description));
                            if !matches!(action_taken, FilterAction::Block) {
                                action_taken = FilterAction::Sanitize;
                            }
                        }
                    }
                }
            }
        }

        // Check against dangerous patterns
        for (i, pattern) in self.dangerous_patterns.iter().enumerate() {
            if pattern.is_match(content) {
                matched_rules.push(format!("dangerous_pattern_{}", i));
                allowed = false;
                action_taken = FilterAction::Block;
                warnings.push("BLOCKED: Dangerous command pattern detected".to_string());
            }
        }

        // Check for blocked commands
        for cmd in &self.blocked_commands {
            if content.to_lowercase().contains(&format!(" {} ", cmd)) || 
               content.to_lowercase().starts_with(&format!("{} ", cmd)) ||
               content.to_lowercase().contains(&format!(" {}", cmd)) {
                matched_rules.push(format!("blocked_command_{}", cmd));
                allowed = false;
                action_taken = FilterAction::Block;
                warnings.push(format!("BLOCKED: Command '{}' is not allowed", cmd));
            }
        }

        let sanitized_content = if matches!(action_taken, FilterAction::Sanitize) {
            Some(self.sanitize_content(content))
        } else {
            None
        };

        FilterResult {
            allowed,
            action: action_taken,
            matched_rules,
            sanitized_content,
            warnings,
        }
    }

    pub fn filter_code_execution(&self, code: &str, language: &str) -> FilterResult {
        let mut result = self.filter_output(code);
        
        // Additional checks for specific languages
        match language.to_lowercase().as_str() {
            "python" => {
                if self.check_python_dangerous_imports(code) {
                    result.allowed = false;
                    result.action = FilterAction::Block;
                    result.warnings.push("BLOCKED: Dangerous Python imports detected".to_string());
                    result.matched_rules.push("python_dangerous_imports".to_string());
                }
            }
            "bash" | "sh" => {
                if self.check_shell_dangerous_commands(code) {
                    result.allowed = false;
                    result.action = FilterAction::Block;
                    result.warnings.push("BLOCKED: Dangerous shell commands detected".to_string());
                    result.matched_rules.push("shell_dangerous_commands".to_string());
                }
            }
            "javascript" | "node" => {
                if self.check_js_dangerous_modules(code) {
                    result.allowed = false;
                    result.action = FilterAction::Block;
                    result.warnings.push("BLOCKED: Dangerous Node.js modules detected".to_string());
                    result.matched_rules.push("js_dangerous_modules".to_string());
                }
            }
            _ => {}
        }
        
        result
    }

    fn check_python_dangerous_imports(&self, code: &str) -> bool {
        let dangerous_imports = vec![
            "os.system", "subprocess.call", "subprocess.run", "subprocess.Popen",
            "socket.socket", "urllib.request", "requests", "http.client",
            "shutil.rmtree", "os.remove", "os.unlink", "tempfile.mktemp",
            "ctypes", "multiprocessing", "threading", "asyncio",
        ];
        
        for imp in dangerous_imports {
            if code.contains(imp) {
                return true;
            }
        }
        false
    }

    fn check_shell_dangerous_commands(&self, code: &str) -> bool {
        let dangerous_commands = vec![
            "rm -rf", "dd if=", "mkfs", "fdisk", "format",
            "shutdown", "reboot", "halt", "poweroff",
            "iptables", "ufw", "firewall-cmd",
            "passwd", "usermod", "chmod 777", "chown root",
            "kill -9", "killall", "pkill",
        ];
        
        for cmd in dangerous_commands {
            if code.to_lowercase().contains(&cmd.to_lowercase()) {
                return true;
            }
        }
        false
    }

    fn check_js_dangerous_modules(&self, code: &str) -> bool {
        let dangerous_modules = vec![
            "child_process", "fs", "os", "cluster", "worker_threads",
            "net", "http", "https", "dgram", "dns", "tls",
            "vm", "v8", "repl", "inspector",
        ];
        
        for module in dangerous_modules {
            if code.contains(&format!("require('{}')", module)) ||
               code.contains(&format!("import.*from.*'{}'", module)) {
                return true;
            }
        }
        false
    }

    fn sanitize_content(&self, content: &str) -> String {
        let mut sanitized = content.to_string();
        
        // Remove or replace dangerous patterns
        for rule in &self.server_protection_rules {
            if let Ok(regex) = Regex::new(&rule.pattern) {
                sanitized = regex.replace_all(&sanitized, "[BLOCKED_COMMAND]").to_string();
            }
        }
        
        sanitized
    }

    pub fn is_file_extension_allowed(&self, filename: &str) -> bool {
        if let Some(extension) = std::path::Path::new(filename)
            .extension()
            .and_then(|ext| ext.to_str()) {
            self.allowed_file_extensions.contains(extension)
        } else {
            false
        }
    }

    pub fn add_protection_rule(&mut self, rule: ProtectionRule) {
        self.server_protection_rules.push(rule);
    }

    pub fn remove_protection_rule(&mut self, rule_id: &str) -> bool {
        let initial_len = self.server_protection_rules.len();
        self.server_protection_rules.retain(|rule| rule.id != rule_id);
        self.server_protection_rules.len() < initial_len
    }

    pub fn get_protection_rules(&self) -> &[ProtectionRule] {
        &self.server_protection_rules
    }

    pub fn get_blocked_commands(&self) -> &HashSet<String> {
        &self.blocked_commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safety_filter_initialization() {
        let filter = SafetyFilter::new();
        assert!(!filter.server_protection_rules.is_empty());
        assert!(!filter.blocked_commands.is_empty());
    }

    #[test]
    fn test_dangerous_command_blocking() {
        let filter = SafetyFilter::new();
        let result = filter.filter_output("rm -rf /");
        assert!(!result.allowed);
        assert!(matches!(result.action, FilterAction::Block));
    }

    #[test]
    fn test_safe_content_allowed() {
        let filter = SafetyFilter::new();
        let result = filter.filter_output("echo 'Hello World'");
        assert!(result.allowed);
    }

    #[test]
    fn test_python_dangerous_imports() {
        let filter = SafetyFilter::new();
        let result = filter.filter_code_execution("import os; os.system('rm -rf /')", "python");
        assert!(!result.allowed);
    }

    #[test]
    fn test_file_extension_filtering() {
        let filter = SafetyFilter::new();
        assert!(filter.is_file_extension_allowed("test.txt"));
        assert!(!filter.is_file_extension_allowed("test.exe"));
    }
}
