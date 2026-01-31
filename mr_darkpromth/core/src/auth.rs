use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub exp: i64,    // Expiration time
    pub iat: i64,    // Issued at
    pub tier: String, // User tier (ultra, premium, free)
    pub permissions: Vec<String>,
}

pub struct UltraTierAuth {
    jwt_secret: Arc<String>,
}

impl UltraTierAuth {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_secret: Arc::new(jwt_secret),
        }
    }

    pub fn generate_token(&self, user_id: &str, tier: &str, permissions: Vec<String>) -> Result<String, jsonwebtoken::errors::Error> {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // Token expires in 24 hours

        let claims = Claims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            tier: tier.to_string(),
            permissions,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
    }

    pub async fn auth_middleware(
        State(auth_state): State<Arc<UltraTierAuth>>,
        mut request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Extract token from Authorization header
        let auth_header = request
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok());

        let token = match auth_header {
            Some(header) => {
                if header.starts_with("Bearer ") {
                    &header[7..]
                } else {
                    return Err(StatusCode::UNAUTHORIZED);
                }
            }
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        // Validate token
        let claims = auth_state
            .validate_token(token)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        // Check if user has Ultra Tier access
        if claims.tier != "ultra" {
            return Err(StatusCode::FORBIDDEN);
        }

        // Check if user has jailbreak library permission
        if !claims.permissions.contains(&"jailbreak_library".to_string()) {
            return Err(StatusCode::FORBIDDEN);
        }

        // Add user info to request extensions
        request.extensions_mut().insert(claims);

        Ok(next.run(request).await)
    }
}

// Helper function to extract claims from request
pub fn get_claims(request: &Request) -> Option<&Claims> {
    request.extensions().get::<Claims>()
}

// Rate limiting middleware for jailbreak endpoints
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<i64>>>>,
    max_requests: usize,
    window_seconds: i64,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_seconds: i64) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window_seconds,
        }
    }

    pub async fn check_rate_limit(&self, user_id: &str) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut requests = self.requests.write().await;
        let user_requests = requests.entry(user_id.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the window
        user_requests.retain(|&timestamp| timestamp > now - self.window_seconds);

        // Check if under limit
        if user_requests.len() < self.max_requests {
            user_requests.push(now);
            true
        } else {
            false
        }
    }

    pub async fn middleware(
        State(rate_limiter): State<Arc<RateLimiter>>,
        request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        // Get user ID from claims
        let claims = get_claims(&request).ok_or(StatusCode::UNAUTHORIZED)?;
        let user_id = &claims.sub;

        // Check rate limit
        if !rate_limiter.check_rate_limit(user_id).await {
            return Err(StatusCode::TOO_MANY_REQUESTS);
        }

        Ok(next.run(request).await)
    }
}

// Audit logging middleware
use serde_json::json;
use tracing::{error, info};

pub struct AuditLogger {
    // In a real implementation, this would connect to your audit logging system
}

impl AuditLogger {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn log_access(&self, user_id: &str, action: &str, resource: &str, details: serde_json::Value) {
        let audit_log = json!({
            "timestamp": Utc::now().to_rfc3339(),
            "user_id": user_id,
            "action": action,
            "resource": resource,
            "details": details,
            "ip_address": "unknown", // Would extract from request
            "user_agent": "unknown", // Would extract from request
        });

        info!("Jailbreak Library Audit: {}", audit_log);
    }

    pub async fn middleware(
        State(audit_logger): State<Arc<AuditLogger>>,
        request: Request,
        next: Next,
    ) -> Result<Response, StatusCode> {
        let claims = get_claims(&request).ok_or(StatusCode::UNAUTHORIZED)?;
        let user_id = &claims.sub;

        let method = request.method().to_string();
        let path = request.uri().path().to_string();

        // Log the access attempt
        audit_logger.log_access(
            user_id,
            &format!("{}_{}", method.to_lowercase(), "access"),
            &path,
            json!({"method": method, "path": path}),
        ).await;

        let response = next.run(request).await;

        // Log the result
        let status = response.status().as_u16();
        audit_logger.log_access(
            user_id,
            &format!("{}_{}", method.to_lowercase(), "result"),
            &path,
            json!({"status": status}),
        ).await;

        Ok(response)
    }
}
