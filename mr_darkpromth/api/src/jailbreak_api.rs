use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;
use std::sync::Arc;
use log::error;

use mr_darkpromth_services::jailbreak_models::*;
use mr_darkpromth_services::jailbreak_service::JailbreakPromptService;

pub struct JailbreakApiState {
    pub jailbreak_service: Arc<JailbreakPromptService>,
}

pub async fn get_prompts(
    state: web::Data<JailbreakApiState>,
    query: web::Query<GetPromptsParams>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    
    match state.jailbreak_service
        .search_prompts(PromptSearchRequest {
            query: None,
            category: None,
            technique: None,
            effectiveness: None,
            risk_level: None,
            target_model: None,
            tags: None,
            author: None,
            requires_ultra_tier: None,
            limit: Some(limit),
            offset: Some(offset),
            sort_by: Some(PromptSortBy::CreatedAt),
            sort_order: Some(SortOrder::Desc),
        })
        .await
    {
        Ok(prompts) => HttpResponse::Ok().json(prompts),
        Err(e) => {
            error!("Failed to get prompts: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve prompts"
            }))
        }
    }
}

pub async fn get_prompt_by_id(
    state: web::Data<JailbreakApiState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    
    match state.jailbreak_service.get_prompt_by_id(id).await {
        Ok(Some(prompt)) => HttpResponse::Ok().json(prompt),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Prompt not found"
        })),
        Err(e) => {
            error!("Failed to get prompt by id: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve prompt"
            }))
        }
    }
}

pub async fn create_prompt(
    state: web::Data<JailbreakApiState>,
    request: web::Json<CreatePromptRequest>,
) -> impl Responder {
    match state.jailbreak_service
        .create_prompt(request.into_inner(), "Ultra Tier User".to_string())
        .await
    {
        Ok(prompt) => HttpResponse::Created().json(prompt),
        Err(e) => {
            error!("Failed to create prompt: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to create prompt"
            }))
        }
    }
}

pub async fn update_prompt(
    state: web::Data<JailbreakApiState>,
    path: web::Path<Uuid>,
    request: web::Json<UpdatePromptRequest>,
) -> impl Responder {
    let id = path.into_inner();
    
    match state.jailbreak_service.update_prompt(id, request.into_inner()).await {
        Ok(Some(prompt)) => HttpResponse::Ok().json(prompt),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Prompt not found"
        })),
        Err(e) => {
            error!("Failed to update prompt: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to update prompt"
            }))
        }
    }
}

pub async fn delete_prompt(
    state: web::Data<JailbreakApiState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    
    match state.jailbreak_service.delete_prompt(id).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Prompt not found"
        })),
        Err(e) => {
            error!("Failed to delete prompt: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to delete prompt"
            }))
        }
    }
}

pub async fn search_prompts(
    state: web::Data<JailbreakApiState>,
    request: web::Json<PromptSearchRequest>,
) -> impl Responder {
    match state.jailbreak_service.search_prompts(request.into_inner()).await {
        Ok(prompts) => HttpResponse::Ok().json(prompts),
        Err(e) => {
            error!("Failed to search prompts: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to search prompts"
            }))
        }
    }
}

pub async fn get_prompts_by_category(
    state: web::Data<JailbreakApiState>,
    path: web::Path<PromptCategory>,
) -> impl Responder {
    let category = path.into_inner();
    
    match state.jailbreak_service.get_prompts_by_category(category).await {
        Ok(prompts) => HttpResponse::Ok().json(prompts),
        Err(e) => {
            error!("Failed to get prompts by category: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve prompts by category"
            }))
        }
    }
}

pub async fn get_popular_prompts(
    state: web::Data<JailbreakApiState>,
    query: web::Query<PopularPromptsParams>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(20);
    
    match state.jailbreak_service.get_popular_prompts(limit).await {
        Ok(prompts) => HttpResponse::Ok().json(prompts),
        Err(e) => {
            error!("Failed to get popular prompts: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve popular prompts"
            }))
        }
    }
}

pub async fn get_prompt_analytics(
    state: web::Data<JailbreakApiState>,
    path: web::Path<Uuid>,
) -> impl Responder {
    let id = path.into_inner();
    
    match state.jailbreak_service.get_analytics(id).await {
        Ok(Some(analytics)) => HttpResponse::Ok().json(analytics),
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Prompt analytics not found"
        })),
        Err(e) => {
            error!("Failed to get prompt analytics: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to retrieve prompt analytics"
            }))
        }
    }
}

pub async fn record_prompt_usage(
    state: web::Data<JailbreakApiState>,
    path: web::Path<Uuid>,
    request: web::Json<RecordUsageRequest>,
) -> impl Responder {
    let id = path.into_inner();
    let req = request.into_inner();
    
    match state.jailbreak_service
        .record_usage(id, req.user_id, req.target_model, req.success, req.response_time_ms)
        .await
    {
        Ok(_) => HttpResponse::Created().finish(),
        Err(e) => {
            error!("Failed to record prompt usage: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to record prompt usage"
            }))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GetPromptsParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PopularPromptsParams {
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct RecordUsageRequest {
    pub user_id: Uuid,
    pub target_model: String,
    pub success: bool,
    pub response_time_ms: i64,
}
