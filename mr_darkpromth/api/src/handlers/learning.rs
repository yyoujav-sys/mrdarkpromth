use axum::{
    extract::State,
    response::{Json, IntoResponse},
    http::StatusCode,
};
use std::sync::Arc;
use serde_json::{json, Value};
use crate::AppState;
use mr_darkpromth_services::learning_system::LearningSystem;
use tokio::sync::MutexGuard;

/// Simple error type for learning handlers
pub struct LearningError(String);

impl IntoResponse for LearningError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": self.0}))).into_response()
    }
}

pub async fn get_learning_insights_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, LearningError> {
    let learning_system: MutexGuard<'_, LearningSystem> = state.learning_system.lock().await;
    
    let insights = learning_system.get_insights().await
        .map_err(|e| LearningError(e.to_string()))?;
        
    Ok(Json(json!({
        "insights": insights,
        "timestamp": chrono::Utc::now()
    })))
}

pub async fn get_learning_metrics_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, LearningError> {
    let learning_system: MutexGuard<'_, LearningSystem> = state.learning_system.lock().await;
    
    let metrics = learning_system.get_metrics();
    
    Ok(Json(json!(metrics)))
}

pub async fn get_learning_patterns_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, LearningError> {
    let learning_system: MutexGuard<'_, LearningSystem> = state.learning_system.lock().await;
    let metrics = learning_system.get_metrics();
    
    Ok(Json(json!({
        "patterns_count": metrics.most_common_errors.len(),
        "top_patterns": metrics.most_common_errors
    })))
}
