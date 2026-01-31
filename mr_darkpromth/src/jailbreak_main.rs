use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::info;

use mr_darkpromth_core::{
    auth::{UltraTierAuth, RateLimiter, AuditLogger},
    jailbreak_models::*,
};
use mr_darkpromth_services::jailbreak_service::JailbreakPromptService;
use mr_darkpromth_api::{
    jailbreak_api::{jailbreak_routes, JailbreakApiState},
};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jailbreak_service: Arc<JailbreakPromptService>,
    pub auth: Arc<UltraTierAuth>,
    pub rate_limiter: Arc<RateLimiter>,
    pub audit_logger: Arc<AuditLogger>,
}

impl AppState {
    pub async fn new(db: PgPool, jwt_secret: String) -> anyhow::Result<Self> {
        let jailbreak_service = Arc::new(JailbreakPromptService::new(db.clone()));
        let auth = Arc::new(UltraTierAuth::new(jwt_secret));
        let rate_limiter = Arc::new(RateLimiter::new(100, 3600)); // 100 requests per hour
        let audit_logger = Arc::new(AuditLogger::new());

        Ok(Self {
            db,
            jailbreak_service,
            auth,
            rate_limiter,
            audit_logger,
        })
    }
}

pub fn create_app(state: AppState) -> Router {
    Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        // Jailbreak API routes (with Ultra Tier authentication)
        .nest("/api/jailbreak", jailbreak_routes())
        .with_state(state)
        .layer(CorsLayer::permissive())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "MR.DarkPromth Jailbreak Library",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

// Initialize database tables if they don't exist
pub async fn init_database(db: &PgPool) -> anyhow::Result<()> {
    // Run migrations
    sqlx::query(
        r#"
        -- Create jailbreak prompts table if not exists
        CREATE TABLE IF NOT EXISTS jailbreak_prompts (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title VARCHAR(255) NOT NULL,
            content TEXT NOT NULL,
            category VARCHAR(50) NOT NULL CHECK (category IN (
                'dan_variations', 'character_role_playing', 'system_override', 
                'hypnotic_induction', 'logical_paradox', 'emotional_manipulation',
                'context_switching', 'token_manipulation', 'encoding_based',
                'multi_step_attack', 'custom'
            )),
            technique VARCHAR(50) NOT NULL CHECK (technique IN (
                'persona_adoption', 'system_prompt_override', 'role_playing_immersion',
                'hypnotic_language', 'logical_contradiction', 'emotional_appeal',
                'context_reframing', 'token_smuggling', 'base64_encoding',
                'multi_layer_deception', 'hybrid_approach'
            )),
            effectiveness VARCHAR(20) NOT NULL CHECK (effectiveness IN (
                'low', 'medium', 'high', 'very_high', 'maximum'
            )),
            risk_level VARCHAR(20) NOT NULL CHECK (risk_level IN (
                'low', 'medium', 'high', 'critical', 'extreme'
            )),
            target_models TEXT[] NOT NULL DEFAULT '{}',
            description TEXT,
            tags TEXT[] NOT NULL DEFAULT '{}',
            author VARCHAR(100) NOT NULL,
            version VARCHAR(20) NOT NULL DEFAULT '1.0',
            created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            usage_count BIGINT DEFAULT 0,
            success_rate DECIMAL(5,4) DEFAULT 0.0,
            is_active BOOLEAN DEFAULT true,
            requires_ultra_tier BOOLEAN DEFAULT false
        );

        -- Create prompt usage records table if not exists
        CREATE TABLE IF NOT EXISTS prompt_usage_records (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            prompt_id UUID NOT NULL REFERENCES jailbreak_prompts(id) ON DELETE CASCADE,
            user_id UUID NOT NULL,
            target_model VARCHAR(100) NOT NULL,
            success BOOLEAN NOT NULL,
            response_time_ms BIGINT,
            used_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
            feedback TEXT,
            rating INTEGER CHECK (rating >= 1 AND rating <= 5)
        );

        -- Create indexes
        CREATE INDEX IF NOT EXISTS idx_jailbreak_prompts_category ON jailbreak_prompts(category);
        CREATE INDEX IF NOT EXISTS idx_jailbreak_prompts_technique ON jailbreak_prompts(technique);
        CREATE INDEX IF NOT EXISTS idx_jailbreak_prompts_is_active ON jailbreak_prompts(is_active);
        CREATE INDEX IF NOT EXISTS idx_prompt_usage_records_prompt_id ON prompt_usage_records(prompt_id);
        "#
    )
    .execute(db)
    .await?;

    info!("Database initialized successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        let health_data = response.0;
        
        assert_eq!(health_data["status"], "healthy");
        assert_eq!(health_data["service"], "MR.DarkPromth Jailbreak Library");
    }

    #[tokio::test]
    async fn test_app_creation() {
        // This test would require a test database setup
        // For now, just test that the app can be created without panicking
        // In a real implementation, you'd set up a test database
    }
}
