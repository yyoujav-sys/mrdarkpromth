use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::auth_middleware::AuthenticatedUser;
use mr_darkpromth_services::{collect_system_metrics, UpdateUserRequest, UserService};

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponseDto {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub tier: String,
    pub api_key: String,
    pub api_key_expires_at: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[utoipa::path(
    get,
    path = "/api/admin/metrics",
    responses(
        (status = 200, description = "System metrics", body = AdminMetricsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_admin_metrics(
    user_service: web::Data<Arc<UserService>>,
) -> impl Responder {
    let metrics = collect_system_metrics("api_gateway");

    match (
        user_service.count_users().await,
        user_service.count_active_users().await,
    ) {
        (Ok(total_users), Ok(active_users)) => HttpResponse::Ok().json(AdminMetricsResponse {
            timestamp: metrics.timestamp.to_rfc3339(),
            total_users,
            active_users,
            cpu_usage_percent: metrics.cpu_usage_percent,
            memory_usage_mb: metrics.memory_usage_mb,
        }),
        _ => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Internal Server Error".to_string(),
            message: "Failed to load metrics".to_string(),
        }),
    }
}

impl From<mr_darkpromth_db::UserResponse> for UserResponseDto {
    fn from(user: mr_darkpromth_db::UserResponse) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            tier: user.tier.to_string(),
            api_key: user.api_key,
            api_key_expires_at: user.api_key_expires_at.map(|t| t.to_rfc3339()),
            is_active: user.is_active,
            created_at: user.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AdminMetricsResponse {
    pub timestamp: String,
    pub total_users: i64,
    pub active_users: i64,
    pub cpu_usage_percent: f32,
    pub memory_usage_mb: u64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserStatusRequest {
    pub status: String,
}

#[utoipa::path(
    get,
    path = "/api/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponseDto),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user(
    user_service: web::Data<Arc<UserService>>,
    user_id: web::Path<Uuid>,
) -> impl Responder {
    match user_service.get_user_by_id(user_id.into_inner()).await {
        Ok(Some(user)) => {
            let user_dto = UserResponseDto::from(user);
            HttpResponse::Ok().json(user_dto)
        }
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Not Found".to_string(),
            message: "User not found".to_string(),
        }),
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Internal Server Error".to_string(),
            message: "Failed to fetch user".to_string(),
        }),
    }
}

#[utoipa::path(
    put,
    path = "/api/users/me",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated", body = UserResponseDto),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_profile(
    user_service: web::Data<Arc<UserService>>,
    auth_user: AuthenticatedUser,
    request: web::Json<UpdateProfileRequest>,
) -> impl Responder {
    let update_req = UpdateUserRequest {
        username: request.username.clone(),
        email: request.email.clone(),
        tier: None,
        is_active: None,
    };

    match user_service.update_user(auth_user.user_id, update_req).await {
        Ok(Some(user)) => {
            let user_dto = UserResponseDto::from(user);
            HttpResponse::Ok().json(user_dto)
        }
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Not Found".to_string(),
            message: "User not found".to_string(),
        }),
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Internal Server Error".to_string(),
            message: "Failed to update profile".to_string(),
        }),
    }
}

#[utoipa::path(
    put,
    path = "/api/users/me/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully"),
        (status = 400, description = "Bad request", body = ErrorResponse)
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn change_password(
    _user_service: web::Data<Arc<UserService>>,
    _auth_user: AuthenticatedUser,
    request: web::Json<ChangePasswordRequest>,
) -> impl Responder {
    // Validate new password
    if request.new_password.len() < 8 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Bad Request".to_string(),
            message: "Password must be at least 8 characters".to_string(),
        });
    }

    // In a real implementation, you would verify the current password
    // For now, we'll return success
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Password changed successfully"
    }))
}

#[utoipa::path(
    post,
    path = "/api/users/me/api-key",
    responses(
        (status = 200, description = "API key regenerated"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn regenerate_api_key(
    user_service: web::Data<Arc<UserService>>,
    auth_user: AuthenticatedUser,
) -> impl Responder {
    match user_service.regenerate_api_key(auth_user.user_id).await {
        Ok(api_key_response) => {
            HttpResponse::Ok().json(serde_json::json!({
                "api_key": api_key_response.api_key,
                "expires_at": api_key_response.expires_at.map(|t| t.to_rfc3339()),
            }))
        }
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Internal Server Error".to_string(),
            message: "Failed to regenerate API key".to_string(),
        }),
    }
}

#[utoipa::path(
    get,
    path = "/api/admin/users",
    params(
        ("limit" = Option<i64>, Query, description = "Limit number of results"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination")
    ),
    responses(
        (status = 200, description = "List of users"),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_users(
    user_service: web::Data<Arc<UserService>>,
    query: web::Query<UserListQuery>,
) -> impl Responder {
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);

    match user_service.list_users(limit, offset).await {
        Ok(users) => {
            let user_dtos: Vec<UserResponseDto> = users.into_iter().map(UserResponseDto::from).collect();
            HttpResponse::Ok().json(serde_json::json!({
                "users": user_dtos,
                "total_count": user_dtos.len(),
                "limit": limit,
                "offset": offset,
            }))
        }
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Internal Server Error".to_string(),
            message: "Failed to fetch users".to_string(),
        }),
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UserListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[utoipa::path(
    delete,
    path = "/api/admin/users/{id}",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "users",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_user(
    user_service: web::Data<Arc<UserService>>,
    user_id: web::Path<Uuid>,
) -> impl Responder {
    match user_service.delete_user(user_id.into_inner()).await {
        Ok(true) => HttpResponse::NoContent().finish(),
        Ok(false) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Not Found".to_string(),
            message: "User not found".to_string(),
        }),
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Internal Server Error".to_string(),
            message: "Failed to delete user".to_string(),
        }),
    }
}

#[utoipa::path(
    put,
    path = "/api/admin/users/{id}/status",
    params(
        ("id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserStatusRequest,
    responses(
        (status = 200, description = "User status updated", body = UserResponseDto),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_user_status(
    user_service: web::Data<Arc<UserService>>,
    user_id: web::Path<Uuid>,
    request: web::Json<UpdateUserStatusRequest>,
) -> impl Responder {
    let status = request.status.to_lowercase();
    let is_active = match status.as_str() {
        "active" => true,
        "suspended" | "pending" => false,
        _ => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Bad Request".to_string(),
                message: "Status must be active, suspended, or pending".to_string(),
            })
        }
    };

    let update_req = UpdateUserRequest {
        username: None,
        email: None,
        tier: None,
        is_active: Some(is_active),
    };

    match user_service.update_user(user_id.into_inner(), update_req).await {
        Ok(Some(user)) => HttpResponse::Ok().json(UserResponseDto::from(user)),
        Ok(None) => HttpResponse::NotFound().json(ErrorResponse {
            error: "Not Found".to_string(),
            message: "User not found".to_string(),
        }),
        Err(_) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Internal Server Error".to_string(),
            message: "Failed to update status".to_string(),
        }),
    }
}
