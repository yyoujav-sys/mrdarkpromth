#![allow(ambiguous_glob_reexports)]

// Axum modules (new)
pub mod axum_router;
pub mod app_state;
pub mod handlers;
pub mod auth_middleware;

// Re-export Axum modules
pub use axum_router::*;
pub use app_state::*;
pub use handlers::*;
pub use auth_middleware::*;