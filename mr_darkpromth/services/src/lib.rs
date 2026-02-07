// MR.DarkPromth Services Library
// Agent 4: Jailbreak & Ultra Tier Engineer
// Agent 6: MasterToolExecutor & Tool System Engineer
// Agent 7: Multi-Agent System Engineer

pub mod jailbreak_system;
pub mod redis_coordination;
// pub mod agent4_main; // Deprecated
pub mod safety_filter;
pub mod sandbox;
pub mod ultra_tier_logic;
#[cfg(test)]
pub mod ultra_tier_integration_tests;
#[cfg(test)]
pub mod ultra_tier_dark_tests;
pub mod audit_analytics;
#[cfg(test)]
pub mod integration_tests;
pub mod monitoring;
pub mod dependency_monitor;
pub mod integration_coordinator;
pub mod self_correction_engine;
pub mod agent8_main;
pub mod audit;
pub mod database_audit;
pub mod tier_validation;
pub mod server_protection;
pub mod sandboxed_execution;
pub mod security_integration_test;
pub mod jailbreak_service;
pub mod response_analyzer;
pub mod jailbreak_models;
pub mod user_service;
pub mod auth_middleware;
pub mod tier_management;
pub mod user_profile;
pub mod cerebras_integration;
// pub mod main_integrated; // Deprecated
pub mod user_event_coordinator;
pub mod user_integration;
pub mod key_pool;
pub mod host_protection;
pub mod billing_service;
pub mod email_service;
pub mod cache_service;

#[cfg(test)]
pub mod test_db_utils;

// Agent 6: Tool System modules
pub mod tool_system;
pub mod tool_plugin_manager;
pub mod tool_registry;
pub mod master_tool_executor;
pub mod tool_sandbox;
pub mod tool_redis_coordination;
pub mod tool_permissions;
pub mod tool_logging;

// Agent 7: Multi-Agent System modules
pub mod agent_framework;
pub mod coordinator_agent;
pub mod editor_agent;
pub mod terminal_agent;
pub mod agent_communication;
pub mod agent_decision_engine;
pub mod agent7_main;

// Agent 8: Self-Correction Engine modules
pub mod error_detector;
pub mod fix_generator;
pub mod correction_validator;
pub mod learning_system;
pub mod self_correction_agent;

// Re-export main types for easier access
pub use redis_coordination::*;
pub use safety_filter::*;
pub use ultra_tier_logic::*;
#[cfg(test)]
pub use integration_tests::*;
pub use monitoring::*;
pub use audit::*;
pub use database_audit::*;
pub use tier_validation::*;
pub use server_protection::*;
pub use sandboxed_execution::*;
pub use user_service::*;
pub use auth_middleware::*;
pub use tier_management::*;
pub use user_profile::*;
pub use cerebras_integration::*;
pub use user_integration::*;

// Agent 6: Tool System exports
pub use tool_system::*;
pub use tool_plugin_manager::*;
pub use tool_registry::*;
pub use master_tool_executor::*;

// Re-export main types for main.rs
pub use jailbreak_system::{JailbreakPrompt, JailbreakSystem, AIModel, PromptCategory, Technique, EffectivenessRating, RiskLevel};
pub use safety_filter::{SafetyFilter, FilterResult, FilterAction, ProtectionRule};
pub use sandbox::{Sandbox, SandboxConfig, Language, ExecutionResult};
pub use ultra_tier_logic::{UltraTierLogic, UltraTierRequest, UltraTierResponse, UserTier, User, AuditLogEntry, UltraAuditAction};

pub use monitoring::{collect_system_metrics, MonitoringSystem, SystemMetrics, HealthStatus, ServiceStatus, DependencyStatus};
pub use dependency_monitor::{DependencyMonitor, DependencyStatus as DepStatus, DependencyState};
pub use integration_coordinator::{IntegrationCoordinator, TestResult};
pub use audit::{AuditLogger, AuditEvent, AuditEventType};
pub use tier_validation::TierValidator;

pub use server_protection::{ServerProtection, ProtectionConfig, ProtectionStats, SuspiciousProcess, ProtectionMiddleware};
pub use sandboxed_execution::SandboxedExecutor;

pub use jailbreak_service::JailbreakPromptService;

pub use jailbreak_models::*;
pub use user_service::{UserService, CreateUserRequest, UpdateUserRequest, UserResponse, Claims, AuthError};
pub use auth_middleware::{AuthState, AuthenticatedUser};

pub use audit_analytics::*;
pub use key_pool::{KeyPool, Provider};
pub use host_protection::{HostProtectionPolicy};
pub use billing_service::{BillingService, Plan, Payment, Subscription, PaymentStatus, PaymentMethod};
pub use email_service::{EmailService, EmailVerificationToken, PasswordResetToken, EmailConfig};
pub use cache_service::CacheService;
