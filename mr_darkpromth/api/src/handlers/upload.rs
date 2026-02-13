use axum::{
    extract::{State, Multipart},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::extract_token;
use crate::error_handler::{ApiError, ApiSuccess};
use mr_darkpromth_core::tier::UserTier;

/// Upload a file to Cloudflare R2
pub async fn upload_file_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Authenticate user
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return ApiError::new("INVALID_TOKEN", "Invalid or expired token").into_response();
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new("INVALID_USER_ID", "Invalid user ID in token").into_response();
        }
    };

    // Extract file from multipart form
    let mut file_data: Option<(String, Vec<u8>, String)> = None; // (filename, content, content_type)

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("upload").to_string();
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            
            match field.bytes().await {
                Ok(bytes) => {
                    // Get user tier for tier-based file size limits
                    let user_tier = match claims.tier.as_str() {
                        "free" => UserTier::Free,
                        "premium" => UserTier::Premium,
                        "ultra" => UserTier::Ultra,
                        "admin" => UserTier::Admin,
                        _ => UserTier::Free,
                    };
                    
                    // Tier-based file size limits
                    let max_size = match user_tier {
                        UserTier::Free => 5 * 1024 * 1024,      // 5MB
                        UserTier::Premium => 50 * 1024 * 1024,  // 50MB
                        UserTier::Ultra => 1024 * 1024 * 1024,  // 1GB
                        UserTier::Admin => usize::MAX,           // Unlimited
                    };
                    
                    if bytes.len() > max_size {
                        let size_str = if max_size == usize::MAX {
                            "unlimited".to_string()
                        } else {
                            format!("{}MB", max_size / (1024 * 1024))
                        };
                        return ApiError::new("FILE_TOO_LARGE", 
                            &format!("File size exceeds {} limit for {} tier", size_str, user_tier.as_str())).into_response();
                    }
                    file_data = Some((filename, bytes.to_vec(), content_type));
                }
                Err(e) => {
                    log::error!("Failed to read multipart field: {}", e);
                    return ApiError::new("UPLOAD_ERROR", "Failed to read file data").into_response();
                }
            }
            break;
        }
    }

    let (filename, content, _content_type) = match file_data {
        Some(data) => data,
        None => {
            return ApiError::new("MISSING_FILE", "No file field found in multipart form").into_response();
        }
    };

    // Generate unique key for R2 storage
    let file_uuid = Uuid::new_v4();
    let extension = filename.rsplit('.').next().unwrap_or("bin");
    let key = format!("uploads/{}/{}.{}", user_id, file_uuid, extension);

    // Upload to Cloudflare R2
    match state.cloudflare_service.upload_to_r2(&key, &content).await {
        Ok(url) => {
            log::info!("File uploaded successfully: {} -> {}", filename, url);
            ApiSuccess::new(serde_json::json!({
                "url": url,
                "key": key,
                "filename": filename,
                "size": content.len(),
            })).into_response()
        }
        Err(e) => {
            log::error!("Failed to upload file to R2: {}", e);
            ApiError::new("UPLOAD_FAILED", format!("File upload failed: {}", e)).into_response()
        }
    }
}
