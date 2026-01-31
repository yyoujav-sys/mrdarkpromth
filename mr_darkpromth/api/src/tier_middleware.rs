use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform, Error}, HttpMessage};
use futures::future::{ready, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use log::{info, warn};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserTier {
    Free,
    Premium,
    Ultra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Uuid,
    pub tier: UserTier,
    pub username: String,
}

pub struct TierDetection {
    secret: String,
}

impl TierDetection {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    fn extract_user_tier(&self, token: &str) -> Option<UserContext> {
        use jsonwebtoken::{decode, Validation, DecodingKey};
        
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        let decoding_key = DecodingKey::from_secret(self.secret.as_ref());

        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,
            exp: usize,
            tier: String,
            username: String,
        }

        match decode::<Claims>(&token, &decoding_key, &validation) {
            Ok(token_data) => {
                let tier = match token_data.claims.tier.as_str() {
                    "free" | "basic" => UserTier::Free,
                    "premium" => UserTier::Premium,
                    "ultra" => UserTier::Ultra,
                    _ => UserTier::Free,
                };

                let user_id = Uuid::parse_str(&token_data.claims.sub).unwrap_or_else(|_| Uuid::new_v4());

                Some(UserContext {
                    user_id,
                    tier,
                    username: token_data.claims.username,
                })
            }
            Err(e) => {
                warn!("Failed to decode JWT for tier detection: {}", e);
                None
            }
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for TierDetection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = TierDetectionMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TierDetectionMiddleware {
            service,
            secret: self.secret.clone(),
        }))
    }
}

pub struct TierDetectionMiddleware<S> {
    service: S,
    secret: String,
}

impl<S, B> Service<ServiceRequest> for TierDetectionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let secret = self.secret.clone();
        
        let auth_header = req.headers().get("Authorization");
        
        if let Some(header) = auth_header {
            if let Ok(header_str) = header.to_str() {
                if header_str.starts_with("Bearer ") {
                    let token = header_str[7..].to_string();
                    
                    if let Some(user_context) = Self::extract_user_tier_from_token(&token, &secret) {
                        info!("User {} detected as {:?} tier", user_context.username, user_context.tier);
                        req.extensions_mut().insert(user_context);
                    }
                }
            }
        }
        
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}

impl TierDetectionMiddleware<S> {
    fn extract_user_tier_from_token(token: &str, secret: &str) -> Option<UserContext> {
        use jsonwebtoken::{decode, Validation, DecodingKey};
        
        let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,
            exp: usize,
            tier: String,
            username: String,
        }

        match decode::<Claims>(&token, &decoding_key, &validation) {
            Ok(token_data) => {
                let tier = match token_data.claims.tier.as_str() {
                    "free" | "basic" => UserTier::Free,
                    "premium" => UserTier::Premium,
                    "ultra" => UserTier::Ultra,
                    _ => UserTier::Free,
                };

                let user_id = Uuid::parse_str(&token_data.claims.sub).unwrap_or_else(|_| Uuid::new_v4());

                Some(UserContext {
                    user_id,
                    tier,
                    username: token_data.claims.username,
                })
            }
            Err(_) => None
        }
    }
}

pub fn require_ultra_tier(user_context: &UserContext) -> Result<(), actix_web::HttpResponse> {
    if !matches!(user_context.tier, UserTier::Ultra) {
        return Err(actix_web::HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Ultra tier required",
            "message": "This feature requires an Ultra tier subscription"
        })));
    }
    Ok(())
}

pub fn require_premium_tier(user_context: &UserContext) -> Result<(), actix_web::HttpResponse> {
    if !matches!(user_context.tier, UserTier::Premium | UserTier::Ultra) {
        return Err(actix_web::HttpResponse::Forbidden().json(serde_json::json!({
            "error": "Premium tier required",
            "message": "This feature requires a Premium or Ultra tier subscription"
        })));
    }
    Ok(())
}
