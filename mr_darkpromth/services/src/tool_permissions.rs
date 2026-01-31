// MR.DarkPromth Tool Permissions System - Agent 6: MasterToolExecutor & Tool System Engineer
// Phase 4: Tool Permissions Implementation

use crate::tool_system::{Tool, ToolError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub name: String,
    pub description: String,
    pub category: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionRule {
    pub permission: String,
    pub allowed_tiers: Vec<UserTier>,
    pub allowed_roles: Vec<String>,
    pub requires_approval: bool,
    pub max_executions_per_hour: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserTier {
    Free,
    Premium,
    Ultra,
}

impl std::fmt::Display for UserTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserTier::Free => write!(f, "free"),
            UserTier::Premium => write!(f, "premium"),
            UserTier::Ultra => write!(f, "ultra"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub reason: Option<String>,
    pub requires_approval: bool,
    pub tier: UserTier,
}

pub struct ToolPermissionManager {
    permissions: HashMap<String, Permission>,
    rules: HashMap<String, PermissionRule>,
    user_permissions: HashMap<String, HashSet<String>>,
    role_permissions: HashMap<String, HashSet<String>>,
    approval_queue: Arc<RwLock<Vec<ApprovalRequest>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub request_id: Uuid,
    pub user_id: String,
    pub tool_name: String,
    pub permission: String,
    pub requested_at: chrono::DateTime<chrono::Utc>,
    pub status: ApprovalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

impl ToolPermissionManager {
    pub fn new() -> Self {
        let mut manager = Self {
            permissions: HashMap::new(),
            rules: HashMap::new(),
            user_permissions: HashMap::new(),
            role_permissions: HashMap::new(),
            approval_queue: Arc::new(RwLock::new(Vec::new())),
        };

        // Initialize default permissions
        manager.initialize_default_permissions();
        manager
    }

    fn initialize_default_permissions(&mut self) {
        // File operations
        self.register_permission(Permission {
            name: "file.read".to_string(),
            description: "Read files from the filesystem".to_string(),
            category: "file_operations".to_string(),
            risk_level: RiskLevel::Low,
        });

        self.register_permission(Permission {
            name: "file.write".to_string(),
            description: "Write files to the filesystem".to_string(),
            category: "file_operations".to_string(),
            risk_level: RiskLevel::Medium,
        });

        // Network operations
        self.register_permission(Permission {
            name: "network.read".to_string(),
            description: "Read data from network resources".to_string(),
            category: "network".to_string(),
            risk_level: RiskLevel::Medium,
        });

        // Code execution
        self.register_permission(Permission {
            name: "code.execute".to_string(),
            description: "Execute code in sandboxed environment".to_string(),
            category: "execution".to_string(),
            risk_level: RiskLevel::High,
        });

        // Database operations
        self.register_permission(Permission {
            name: "database.read".to_string(),
            description: "Read data from database".to_string(),
            category: "database".to_string(),
            risk_level: RiskLevel::Medium,
        });

        // Admin operations
        self.register_permission(Permission {
            name: "admin.manage".to_string(),
            description: "Administrative management operations".to_string(),
            category: "admin".to_string(),
            risk_level: RiskLevel::Critical,
        });

        // Set up default permission rules
        self.register_rule(PermissionRule {
            permission: "file.read".to_string(),
            allowed_tiers: vec![UserTier::Free, UserTier::Premium, UserTier::Ultra],
            allowed_roles: vec!["user".to_string(), "admin".to_string()],
            requires_approval: false,
            max_executions_per_hour: None,
        });

        self.register_rule(PermissionRule {
            permission: "file.write".to_string(),
            allowed_tiers: vec![UserTier::Premium, UserTier::Ultra],
            allowed_roles: vec!["user".to_string(), "admin".to_string()],
            requires_approval: false,
            max_executions_per_hour: Some(100),
        });

        self.register_rule(PermissionRule {
            permission: "network.read".to_string(),
            allowed_tiers: vec![UserTier::Premium, UserTier::Ultra],
            allowed_roles: vec!["user".to_string(), "admin".to_string()],
            requires_approval: false,
            max_executions_per_hour: Some(50),
        });

        self.register_rule(PermissionRule {
            permission: "code.execute".to_string(),
            allowed_tiers: vec![UserTier::Ultra],
            allowed_roles: vec!["admin".to_string()],
            requires_approval: true,
            max_executions_per_hour: Some(20),
        });

        self.register_rule(PermissionRule {
            permission: "database.read".to_string(),
            allowed_tiers: vec![UserTier::Premium, UserTier::Ultra],
            allowed_roles: vec!["user".to_string(), "admin".to_string()],
            requires_approval: false,
            max_executions_per_hour: Some(100),
        });

        self.register_rule(PermissionRule {
            permission: "admin.manage".to_string(),
            allowed_tiers: vec![UserTier::Ultra],
            allowed_roles: vec!["admin".to_string()],
            requires_approval: true,
            max_executions_per_hour: Some(10),
        });
    }

    pub fn register_permission(&mut self, permission: Permission) {
        self.permissions.insert(permission.name.clone(), permission);
    }

    pub fn register_rule(&mut self, rule: PermissionRule) {
        self.rules.insert(rule.permission.clone(), rule);
    }

    pub fn check_permission(
        &self,
        user_id: &str,
        user_tier: &str,
        tool: &dyn Tool,
    ) -> PermissionCheckResult {
        let required_permissions = tool.requires_permissions();
        
        if required_permissions.is_empty() {
            return PermissionCheckResult {
                allowed: true,
                reason: None,
                requires_approval: false,
                tier: Self::parse_tier(user_tier),
            };
        }

        let tier = Self::parse_tier(user_tier);
        let mut all_allowed = true;
        let mut requires_approval = false;
        let mut denial_reasons = Vec::new();

        for permission in &required_permissions {
            if let Some(rule) = self.rules.get(permission) {
                let tier_allowed = rule.allowed_tiers.iter().any(|t| t == &tier);
                
                if !tier_allowed {
                    all_allowed = false;
                    denial_reasons.push(format!(
                        "Permission '{}' requires {} tier or higher",
                        permission,
                        Self::get_minimum_tier(&rule.allowed_tiers)
                    ));
                    continue;
                }

                if rule.requires_approval {
                    requires_approval = true;
                }

                // Check execution limits
                if let Some(max_executions) = rule.max_executions_per_hour {
                    if self.get_execution_count(user_id, permission) >= max_executions {
                        all_allowed = false;
                        denial_reasons.push(format!(
                            "Execution limit reached for permission '{}'",
                            permission
                        ));
                    }
                }
            } else {
                // No rule defined, deny by default
                all_allowed = false;
                denial_reasons.push(format!("No rule defined for permission '{}'", permission));
            }
        }

        PermissionCheckResult {
            allowed: all_allowed,
            reason: if all_allowed {
                if requires_approval {
                    Some("Requires approval".to_string())
                } else {
                    None
                }
            } else {
                Some(denial_reasons.join("; "))
            },
            requires_approval,
            tier,
        }
    }

    pub fn request_approval(
        &mut self,
        user_id: String,
        tool_name: String,
        permission: String,
    ) -> Uuid {
        let request_id = Uuid::new_v4();
        let request = ApprovalRequest {
            request_id,
            user_id,
            tool_name,
            permission,
            requested_at: chrono::Utc::now(),
            status: ApprovalStatus::Pending,
        };

        self.approval_queue.write().unwrap().push(request);
        request_id
    }

    pub fn approve_request(&mut self, request_id: Uuid) -> Result<(), ToolError> {
        let mut queue = self.approval_queue.write().unwrap();
        if let Some(request) = queue.iter_mut().find(|r| r.request_id == request_id) {
            request.status = ApprovalStatus::Approved;
            Ok(())
        } else {
            Err(ToolError::ToolNotFound("Approval request not found".to_string()))
        }
    }

    pub fn deny_request(&mut self, request_id: Uuid) -> Result<(), ToolError> {
        let mut queue = self.approval_queue.write().unwrap();
        if let Some(request) = queue.iter_mut().find(|r| r.request_id == request_id) {
            request.status = ApprovalStatus::Denied;
            Ok(())
        } else {
            Err(ToolError::ToolNotFound("Approval request not found".to_string()))
        }
    }

    pub fn get_pending_approvals(&self) -> Vec<ApprovalRequest> {
        self.approval_queue
            .read()
            .unwrap()
            .iter()
            .filter(|r| r.status == ApprovalStatus::Pending)
            .cloned()
            .collect()
    }

    pub fn cleanup_expired_requests(&mut self, max_age_hours: i64) {
        let mut queue = self.approval_queue.write().unwrap();
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(max_age_hours);
        queue.retain(|r| r.requested_at > cutoff);
    }

    fn get_execution_count(&self, _user_id: &str, _permission: &str) -> u32 {
        // In a real implementation, this would track actual execution counts
        // For now, return 0
        0
    }

    fn parse_tier(tier_str: &str) -> UserTier {
        match tier_str.to_lowercase().as_str() {
            "free" => UserTier::Free,
            "premium" => UserTier::Premium,
            "ultra" => UserTier::Ultra,
            _ => UserTier::Free,
        }
    }

    fn get_minimum_tier(tiers: &[UserTier]) -> String {
        if tiers.contains(&UserTier::Free) {
            "Free".to_string()
        } else if tiers.contains(&UserTier::Premium) {
            "Premium".to_string()
        } else {
            "Ultra".to_string()
        }
    }

    pub fn list_permissions(&self) -> Vec<Permission> {
        self.permissions.values().cloned().collect()
    }

    pub fn get_permission(&self, name: &str) -> Option<Permission> {
        self.permissions.get(name).cloned()
    }
}

impl Default for ToolPermissionManager {
    fn default() -> Self {
        Self::new()
    }
}
