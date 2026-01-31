use actix_web::{
    dev::{forward, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpRequest,
};
use futures_util::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use mr_darkpromth_services::{UserService, Claims};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub email: String,
    pub tier: String,
}

impl FromRequest for AuthenticatedUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // Try to get user from request extensions (set by middleware)
        if let Some(user) = req.extensions().get::<AuthenticatedUser>() {
            ok(user.clone())
        } else {
            ok(Err(actix_web::error::ErrorUnauthorized("Unauthorized")))
        }
    }
}

pub struct AuthMiddleware {
    user_service: Arc<UserService>,
    jwt_secret: String,
}

impl AuthMiddleware {
    pub fn new(user_service: Arc<UserService>, jwt_secret: String) -> Self {
        Self {
            user_service,
            jwt_secret,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service,
            user_service: self.user_service.clone(),
            jwt_secret: self.jwt_secret.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    user_service: Arc<UserService>,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let user_service = self.user_service.clone();
        let jwt_secret = self.jwt_secret.clone();

        Box::pin(async move {
            // Extract Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            if let Some(auth_header) = auth_header {
                if auth_header.starts_with("Bearer ") {
                    let token = &auth_header[7..]; // Remove "Bearer " prefix

                    // Validate token
                    match user_service.validate_token(token).await {
                        Ok(claims) => {
                            // Create authenticated user
                            let user_id = uuid::Uuid::parse_str(&claims.sub).unwrap_or_default();
                            let auth_user = AuthenticatedUser {
                                user_id,
                                username: claims.username,
                                email: claims.email,
                                tier: claims.tier,
                            };

                            // Add user to request extensions
                            req.extensions_mut().insert(auth_user);

                            // Continue with request
                            let res = self.service.call(req).await?;
                            return Ok(res);
                        }
                        Err(_) => {
                            return Err(actix_web::error::ErrorUnauthorized(
                                "Invalid or expired token",
                            ));
                        }
                    }
                }
            }

            // No valid token found
            Err(actix_web::error::ErrorUnauthorized("Missing or invalid authorization header"))
        })
    }
}

pub struct OptionalAuthMiddleware {
    user_service: Arc<UserService>,
    jwt_secret: String,
}

impl OptionalAuthMiddleware {
    pub fn new(user_service: Arc<UserService>, jwt_secret: String) -> Self {
        Self {
            user_service,
            jwt_secret,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for OptionalAuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = OptionalAuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(OptionalAuthMiddlewareService {
            service,
            user_service: self.user_service.clone(),
            jwt_secret: self.jwt_secret.clone(),
        })
    }
}

pub struct OptionalAuthMiddlewareService<S> {
    service: S,
    user_service: Arc<UserService>,
    jwt_secret: String,
}

impl<S, B> Service<ServiceRequest> for OptionalAuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let user_service = self.user_service.clone();
        let jwt_secret = self.jwt_secret.clone();

        Box::pin(async move {
            // Try to extract and validate token, but don't fail if missing
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok());

            if let Some(auth_header) = auth_header {
                if auth_header.starts_with("Bearer ") {
                    let token = &auth_header[7..];

                    if let Ok(claims) = user_service.validate_token(token).await {
                        let user_id = uuid::Uuid::parse_str(&claims.sub).unwrap_or_default();
                        let auth_user = AuthenticatedUser {
                            user_id,
                            username: claims.username,
                            email: claims.email,
                            tier: claims.tier,
                        };
                        req.extensions_mut().insert(auth_user);
                    }
                }
            }

            // Continue with request regardless of auth status
            let res = self.service.call(req).await?;
            Ok(res)
        })
    }
}
