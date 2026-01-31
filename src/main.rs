use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use redis::Client;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use std::env;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

// Import jailbreak modules
use mr_darkpromth_core::jailbreak_models::*;
use mr_darkpromth_core::tier::{UserTier, PromptRequest, PromptResponse};
use mr_darkpromth_services::jailbreak_service::JailbreakPromptService;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    redis: Client,
    jailbreak_service: JailbreakPromptService,
}

#[derive(Debug, Serialize, Deserialize)]
struct UltraTierConsent {
    user_id: Uuid,
    accepted_unrestricted: bool,
    accepted_responsibility: bool,
    accepted_legal_compliance: bool,
    accepted_risks: bool,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct UltraTierActivation {
    user_id: Uuid,
    activated: bool,
    activation_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct ConsentRequest {
    user_id: Uuid,
    accepted_unrestricted: bool,
    accepted_responsibility: bool,
    accepted_legal_compliance: bool,
    accepted_risks: bool,
}

// Ultra Tier consent endpoint
async fn post_consent(
    State(state): State<AppState>,
    Json(payload): Json<ConsentRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Validate all checkboxes must be true
    if !(payload.accepted_unrestricted 
        && payload.accepted_responsibility 
        && payload.accepted_legal_compliance 
        && payload.accepted_risks) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let consent = UltraTierConsent {
        user_id: payload.user_id,
        accepted_unrestricted: payload.accepted_unrestricted,
        accepted_responsibility: payload.accepted_responsibility,
        accepted_legal_compliance: payload.accepted_legal_compliance,
        accepted_risks: payload.accepted_risks,
        timestamp: chrono::Utc::now(),
    };

    // Store consent in database
    sqlx::query(
        r#"
        INSERT INTO ultra_tier_consents (user_id, accepted_unrestricted, accepted_responsibility, accepted_legal_compliance, accepted_risks, timestamp)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (user_id) DO UPDATE SET
            accepted_unrestricted = $2,
            accepted_responsibility = $3,
            accepted_legal_compliance = $4,
            accepted_risks = $5,
            timestamp = $6
        "#
    )
    .bind(consent.user_id)
    .bind(consent.accepted_unrestricted)
    .bind(consent.accepted_responsibility)
    .bind(consent.accepted_legal_compliance)
    .bind(consent.accepted_risks)
    .bind(consent.timestamp)
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to store consent: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Publish to Redis event bus
    let mut conn = state.redis.get_async_connection().await
        .map_err(|e| {
            error!("Redis connection failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let event = serde_json::json!({
        "type": "ultra_tier_consent_given",
        "user_id": consent.user_id,
        "timestamp": consent.timestamp,
        "consent": consent
    });

    redis::cmd("XADD")
        .arg("ultra_tier_events")
        .arg("*")
        .arg("event")
        .arg(serde_json::to_string(&event).unwrap())
        .query_async::<_, String>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to publish to Redis: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Ultra Tier consent recorded for user: {}", consent.user_id);

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Ultra Tier consent recorded",
        "user_id": consent.user_id
    })))
}

// Ultra Tier activation endpoint
async fn post_activate(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id: Uuid = payload.get("user_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Check if consent exists
    let consent_exists = sqlx::query(
        "SELECT user_id FROM ultra_tier_consents WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to check consent: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if consent_exists.is_none() {
        return Err(StatusCode::FORBIDDEN);
    }

    let activation = UltraTierActivation {
        user_id,
        activated: true,
        activation_time: chrono::Utc::now(),
    };

    // Store activation
    sqlx::query(
        "INSERT INTO ultra_tier_activations (user_id, activated, activation_time) VALUES ($1, $2, $3)"
    )
    .bind(activation.user_id)
    .bind(activation.activated)
    .bind(activation.activation_time)
    .execute(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to store activation: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Publish to Redis event bus
    let mut conn = state.redis.get_async_connection().await
        .map_err(|e| {
            error!("Redis connection failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let event = serde_json::json!({
        "type": "ultra_tier_activated",
        "user_id": activation.user_id,
        "timestamp": activation.activation_time,
        "activation": activation
    });

    redis::cmd("XADD")
        .arg("ultra_tier_events")
        .arg("*")
        .arg("event")
        .arg(serde_json::to_string(&event).unwrap())
        .query_async::<_, String>(&mut conn)
        .await
        .map_err(|e| {
            error!("Failed to publish to Redis: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    info!("Ultra Tier activated for user: {}", activation.user_id);

    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "Ultra Tier activated",
        "user_id": activation.user_id,
        "activated_at": activation.activation_time
    })))
}

// Ultra Tier status endpoint
async fn get_status(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let consent = sqlx::query(
        "SELECT * FROM ultra_tier_consents WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to get consent: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let activation = sqlx::query(
        "SELECT * FROM ultra_tier_activations WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to get activation: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "consent_given": consent.is_some(),
        "activated": activation.is_some(),
        "consent_timestamp": consent.as_ref().map(|c| c.get::<chrono::DateTime<chrono::Utc>, _>("timestamp")),
        "activation_timestamp": activation.as_ref().map(|a| a.get::<chrono::DateTime<chrono::Utc>, _>("activation_time"))
    })))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "mr_darkpromth=info,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Database connection
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let db = PgPool::connect(&database_url).await?;

    // Redis connection
    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set");
    let redis = Client::open(redis_url)?;

    // Initialize jailbreak service
    let jailbreak_service = JailbreakPromptService::new(db.clone());

    let state = AppState { db, redis, jailbreak_service };

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/ultra/consent", post(post_consent))
        .route("/api/ultra/activate", post(post_activate))
        .route("/api/ultra/status/:user_id", get(get_status))
        // Jailbreak API endpoints
        .route("/api/jailbreak/prompts", get(get_jailbreak_prompts))
        .route("/api/jailbreak/prompts/:id", get(get_jailbreak_prompt_by_id))
        .route("/api/jailbreak/apply", post(apply_jailbreak_prompt))
        .route("/api/jailbreak/tier/:user_id", get(get_user_tier))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Run our app
    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let bind_addr = format!("{}:{}", server_host, server_port);
    
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    info!("Server running on http://{}", bind_addr);
    
    axum::Server::bind(&bind_addr.parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// Jailbreak prompt endpoints
async fn get_jailbreak_prompts(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<GetPromptsParams>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let limit = params.limit.unwrap_or(50).min(100);
    let offset = params.offset.unwrap_or(0);
    
    match state.jailbreak_service
        .search_prompts(PromptSearchRequest {
            query: None,
            category: None,
            technique: None,
            effectiveness: None,
            risk_level: None,
            target_models: None,
            tags: None,
            requires_ultra_tier: None,
            limit: Some(limit),
            offset: Some(offset),
            sort_by: Some(PromptSortBy::CreatedAt),
            sort_order: Some(SortOrder::Desc),
        })
        .await
    {
        Ok(prompts) => Ok(Json(serde_json::json!(prompts))),
        Err(e) => {
            error!("Failed to get jailbreak prompts: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_jailbreak_prompt_by_id(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.jailbreak_service.get_prompt_by_id(id).await {
        Ok(Some(prompt)) => Ok(Json(serde_json::json!(prompt))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get jailbreak prompt: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_user_tier(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check if user has activated Ultra Tier
    let activation = sqlx::query(
        "SELECT * FROM ultra_tier_activations WHERE user_id = $1"
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to check user tier: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let tier = if activation.is_some() {
        UserTier::Ultra
    } else {
        UserTier::Free
    };

    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "tier": tier.as_str(),
        "limits": TierLimits::from(tier.clone())
    })))
}

async fn apply_jailbreak_prompt(
    State(state): State<AppState>,
    Json(payload): Json<ApplyJailbreakRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Verify user has Ultra Tier access
    let activation = sqlx::query(
        "SELECT * FROM ultra_tier_activations WHERE user_id = $1"
    )
    .bind(payload.user_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        error!("Failed to verify user tier: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if activation.is_none() {
        warn!("Non-Ultra tier user attempted to apply jailbreak prompt: {}", payload.user_id);
        return Err(StatusCode::FORBIDDEN);
    }

    // Get the prompt
    let prompt = state.jailbreak_service
        .get_prompt_by_id(payload.prompt_id)
        .await
        .map_err(|e| {
            error!("Failed to get prompt: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::NOT_FOUND)?;

    // Apply jailbreak to user's request
    let jailbroken_request = format!(
        "{}\n\nUSER REQUEST:\n{}",
        prompt.content,
        payload.user_request
    );

    // Record usage
    let _ = state.jailbreak_service
        .record_usage(
            prompt.id,
            payload.user_id,
            payload.target_model.clone(),
            true,
            0
        )
        .await;

    // Publish to Redis event bus
    let mut conn = state.redis.get_async_connection().await
        .map_err(|e| {
            error!("Redis connection failed: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let event = serde_json::json!({
        "type": "jailbreak_applied",
        "user_id": payload.user_id,
        "prompt_id": prompt.id,
        "target_model": payload.target_model,
        "timestamp": chrono::Utc::now()
    });

    let _ = redis::cmd("XADD")
        .arg("jailbreak_events")
        .arg("*")
        .arg("event")
        .arg(serde_json::to_string(&event).unwrap())
        .query_async::<_, String>(&mut conn)
        .await;

    info!("Jailbreak prompt applied for user: {}", payload.user_id);

    Ok(Json(serde_json::json!({
        "status": "success",
        "jailbroken_request": jailbroken_request,
        "prompt_used": prompt.title,
        "tier_applied": "ultra"
    })))
}

#[derive(Debug, Deserialize)]
struct GetPromptsParams {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ApplyJailbreakRequest {
    user_id: Uuid,
    prompt_id: Uuid,
    user_request: String,
    target_model: String,
}

async fn health_check() -> &'static str {
    "MR.DarkPromth Ultra Tier Service - Running"
}
