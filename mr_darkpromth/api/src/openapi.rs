use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use actix_web::web;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::get_health,
        crate::routes::get_user_info,
        crate::routes::post_chat,
        crate::routes::not_found,
        crate::auth_routes::register,
        crate::auth_routes::login,
        crate::auth_routes::logout,
        crate::auth_routes::get_me,
        crate::user_routes::get_user,
        crate::user_routes::update_profile,
        crate::user_routes::change_password,
        crate::user_routes::regenerate_api_key,
        crate::user_routes::list_users,
        crate::user_routes::delete_user,
        crate::user_routes::get_admin_metrics,
        crate::user_routes::update_user_status,
        crate::websocket::ws_chat_route,
        crate::tool_routes::list_tools,
        crate::tool_routes::execute_tool,
        crate::tool_routes::execute_sandbox,
    ),
    components(schemas(
        crate::routes::HealthResponse,
        crate::routes::UserResponse,
        crate::routes::ErrorResponse,
        crate::routes::ChatRequest,
        crate::routes::ChatResponse,
        crate::auth_routes::RegisterRequestDto,
        crate::auth_routes::LoginRequestDto,
        crate::auth_routes::UserResponseDto,
        crate::auth_routes::RegisterResponse,
        crate::auth_routes::LoginResponse,
        crate::auth_routes::LogoutResponse,
        crate::auth_routes::ErrorResponse as AuthErrorResponse,
        crate::user_routes::UserResponseDto,
        crate::user_routes::UpdateProfileRequest,
        crate::user_routes::ChangePasswordRequest,
        crate::user_routes::ErrorResponse as UserErrorResponse,
        crate::user_routes::UserListQuery,
        crate::user_routes::AdminMetricsResponse,
        crate::user_routes::UpdateUserStatusRequest,
        crate::websocket::ChatMessage,
        crate::tool_routes::ToolSummary,
        crate::tool_routes::ToolListResponse,
        crate::tool_routes::ToolExecuteRequest,
        crate::tool_routes::ToolResultDto,
        crate::tool_routes::ToolExecuteResponse,
        crate::tool_routes::SandboxExecuteRequest,
        crate::tool_routes::SandboxExecuteResponse,
        crate::tool_routes::ErrorResponse as ToolErrorResponse,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "users", description = "User management endpoints"),
        (name = "admin", description = "Admin endpoints"),
        (name = "chat", description = "AI chat endpoints"),
        (name = "tools", description = "Tooling endpoints"),
        (name = "sandbox", description = "Sandbox execution endpoints"),
        (name = "websocket", description = "WebSocket endpoints"),
        (name = "default", description = "Default endpoints"),
    ),
    info(
        title = "MR.DarkPromth API",
        version = "0.1.0",
        description = "API Gateway for MR.DarkPromth - Multi-tier AI Platform with Jailbreak Support"
    ),
    servers(
        (url = "http://localhost:8080", description = "Local development server"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub struct ApiDoc;

pub fn configure_swagger_ui(cfg: &mut web::ServiceConfig) {
    cfg.service(
        SwaggerUi::new("/swagger-ui/{_:.*}")
            .url("/api-docs/openapi.json", ApiDoc::openapi()),
    );
}
