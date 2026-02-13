pub mod cloudflare_service;
pub mod slack_service;
pub mod openrouter_client;
// MR.DarkPromth Services Library
// Agent 4: Jailbreak & Ultra Tier Engineer
// Agent 6: MasterToolExecutor & Tool System Engineer
// Agent 7: Multi-Agent System Engineer
// Agent 8: Self-Correction Engine modules

pub mod jailbreak_system;
pub mod redis_coordination;
pub mod strategic_bypass_engine;
pub mod ultra_tier_logic;
pub mod audit_analytics;
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
pub mod user_event_coordinator;
pub mod user_integration;
pub mod key_pool;
pub mod host_protection;
pub mod billing_service;
pub mod email_service;
pub mod cache_service;
pub mod auto_docs;
pub mod telemetry;
pub mod i18n;
pub mod test_db_utils;
pub mod sandbox;

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

// Explicit Re-exports (Strictly NO Glob Exports to avoid shadowing)
pub use jailbreak_models::{
    JailbreakPrompt, CreatePromptRequest, UpdatePromptRequest, PromptResponse,
    PromptSearchRequest, PromptSearchResponse, PromptCategory, BypassTechnique,
    EffectivenessRating, RiskLevel,
};

pub use jailbreak_service::JailbreakPromptService;

pub use user_service::{UserService, CreateUserRequest, UpdateUserRequest, UserResponse, Claims, AuthError};

pub use auth_middleware::{
    AuthState, AuthenticatedUser, api_key_middleware, auth_middleware,
    check_user_tier, extract_auth_user, optional_auth_middleware,
    require_premium_tier, require_ultra_tier,
};

pub use mr_darkpromth_core::tier::UserTier;
pub use ultra_tier_logic::{UltraTierLogic, UltraTierRequest, UltraTierResponse};

pub use strategic_bypass_engine::{StrategicBypassEngine, StrategicAction, StrategicRule, OperationalResult, OperationalPriority};

pub use sandboxed_execution::{SandboxedExecutor, SandboxConfig as SandboxedExecutorConfig, SandboxError, SandboxManager, SandboxSession};
pub use sandbox::{Sandbox, SandboxConfig, Language, ExecutionResult as SandboxExecutionResult};

pub use monitoring::{collect_system_metrics, MonitoringSystem, HealthStatus, ServiceStatus, DependencyStatus};

pub use audit::{AuditLogger, AuditEvent, AuditEventType, AuditLog, AuditFilter, AuditStats, AuditAction, AuditSeverity};

pub use tier_management::{TierManagementService, TierUpgradeRequest, TierUpgradeResponse, TierStats};

pub use billing_service::{BillingService, Plan, Subscription, PaymentStatus};

pub use key_pool::{KeyPool, Provider};

pub use cache_service::CacheService;

pub use email_service::EmailService;

pub use redis_coordination::{RedisCoordinator, CoordinationEvent};

pub use database_audit::DatabaseAuditService;

pub use tool_registry::ToolRegistry;
pub use master_tool_executor::MasterToolExecutor;
pub use cerebras_integration::CerebrasClient;
pub use openrouter_client::OpenRouterClient;
pub use tier_validation::TierValidator;
pub use auto_docs::{AutoDocService, DocModule, DocFunction, DocStruct, DocEnum};
pub use telemetry::TelemetryService;
pub use i18n::{Language as BackendLanguage, Translator};
pub use learning_system::LearningSystem;
pub use slack_service::SlackService;
pub use cloudflare_service::CloudflareService;
