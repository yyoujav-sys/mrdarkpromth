use actix_web::{web, HttpResponse, Responder, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use mr_darkpromth_services::{UserService, RegisterRequest, LoginRequest, AuthResponse, Claims};
use crate::auth_middleware::AuthenticatedUser;

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequestDto {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequestDto {
    pub email: String,
    pub password: String,
}

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

#[derive(Debug, Serialize, ToSchema)]
pub struct RegisterResponse {
    pub user: UserResponseDto,
    pub token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub user: UserResponseDto,
    pub token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LogoutResponse {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequestDto,
    responses(
        (status = 201, description = "User registered successfully", body = RegisterResponse),
        (status = 400, description = "Registration failed", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn register(
    user_service: web::Data<Arc<UserService>>,
    request: web::Json<RegisterRequestDto>,
) -> impl Responder {
    let register_req = RegisterRequest {
        username: request.username.clone(),
        email: request.email.clone(),
        password: request.password.clone(),
    };

    match user_service.register(register_req).await {
        Ok(auth_response) => {
            let user_dto = UserResponseDto::from(auth_response.user);
            HttpResponse::Created().json(RegisterResponse {
                user: user_dto,
                token: auth_response.token,
                expires_in: auth_response.expires_in,
            })
        }
        Err(e) => {
            let error_msg = match e {
                mr_darkpromth_services::AuthError::UserAlreadyExists => "User already exists",
                mr_darkpromth_services::AuthError::InvalidCredentials => "Invalid credentials",
                _ => "Registration failed",
            };
            HttpResponse::BadRequest().json(ErrorResponse {
                error: "Registration Error".to_string(),
                message: error_msg.to_string(),
            })
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequestDto,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn login(
    user_service: web::Data<Arc<UserService>>,
    request: web::Json<LoginRequestDto>,
) -> impl Responder {
    let login_req = LoginRequest {
        email: request.email.clone(),
        password: request.password.clone(),
    };

    match user_service.login(login_req).await {
        Ok(auth_response) => {
            let user_dto = UserResponseDto::from(auth_response.user);
            HttpResponse::Ok().json(LoginResponse {
                user: user_dto,
                token: auth_response.token,
                expires_in: auth_response.expires_in,
            })
        }
        Err(e) => {
            let status = match e {
                mr_darkpromth_services::AuthError::InvalidCredentials => actix_web::http::StatusCode::UNAUTHORIZED,
                mr_darkpromth_services::AuthError::UserNotFound => actix_web::http::StatusCode::NOT_FOUND,
                _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            };
            HttpResponse::build(status).json(ErrorResponse {
                error: "Login Error".to_string(),
                message: "Invalid credentials".to_string(),
            })
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/logout",
    responses(
        (status = 200, description = "Logout successful", body = LogoutResponse)
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn logout(
    _user: AuthenticatedUser,
) -> impl Responder {
    // In a real implementation, you might want to invalidate the token
    // For now, we'll just return success
    HttpResponse::Ok().json(LogoutResponse {
        message: "Logged out successfully".to_string(),
    })
}

#[utoipa::path(
    get,
    path = "/api/auth/me",
    responses(
        (status = 200, description = "Current user info", body = UserResponseDto),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "auth",
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_me(
    user_service: web::Data<Arc<UserService>>,
    auth_user: AuthenticatedUser,
) -> impl Responder {
    match user_service.get_user_by_id(auth_user.user_id).await {
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
