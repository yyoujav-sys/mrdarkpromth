#![allow(ambiguous_glob_reexports)]

// Axum modules (new)
pub mod axum_router;
pub mod app_state;
pub mod handlers;
pub mod i18n_utils;
pub mod auth_middleware;
pub mod middleware;
pub mod logging;
pub mod error_handler;
pub mod structured_logging;
pub mod auth_validators;
pub mod database_config;
pub mod rate_limiting_middleware;

// Re-export Axum modules
pub use axum_router::*;
pub use app_state::*;
pub use handlers::*;
pub use i18n_utils::*;
pub use auth_middleware::*;
pub use logging::*;
pub use error_handler::*;
pub use structured_logging::*;
pub use auth_validators::*;
pub use database_config::*;
pub use rate_limiting_middleware::*;