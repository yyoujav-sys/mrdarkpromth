use actix_web::{body::{BoxBody, MessageBody}, dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage, HttpResponse};
use futures::future::{ready, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use jsonwebtoken::{decode, Validation, DecodingKey};
use serde::{Deserialize, Serialize};
use log::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

pub struct JwtAuth {
    pub secret: String,
}

impl JwtAuth {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service,
            secret: self.secret.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: S,
    secret: String,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let secret = self.secret.clone();
        
        let auth_header = req.headers().get("Authorization");
        
        let token = match auth_header {
            Some(header) => {
                let header_str = match header.to_str() {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Invalid Authorization header format: {}", e);
                        return Box::pin(async move {
                            Ok(req.into_response(
                                HttpResponse::Unauthorized().json(serde_json::json!({
                                    "error": "Invalid token format"
                                }))
                            ))
                        });
                    }
                };
                
                if header_str.starts_with("Bearer ") {
                    Some(header_str[7..].to_string())
                } else {
                    warn!("Authorization header missing 'Bearer ' prefix");
                    None
                }
            }
            None => None,
        };
        
        if let Some(token) = token {
            let validation = Validation::new(jsonwebtoken::Algorithm::HS256);
            let decoding_key = DecodingKey::from_secret(secret.as_ref());
            
            match decode::<Claims>(&token, &decoding_key, &validation) {
                Ok(token_data) => {
                    info!("JWT validated for user: {}", token_data.claims.sub);
                    req.extensions_mut().insert(token_data.claims);
                    
                    let fut = self.service.call(req);
                    Box::pin(async move {
                        let res = fut.await?.map_into_boxed_body();
                        Ok(res)
                    })
                }
                Err(e) => {
                    warn!("JWT validation failed: {}", e);
                    Box::pin(async move {
                        Ok(req.into_response(
                            HttpResponse::Unauthorized().json(serde_json::json!({
                                "error": "Invalid or expired token"
                            }))
                        ))
                    })
                }
            }
        } else {
            Box::pin(async move {
                Ok(req.into_response(
                    HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "Missing authorization token"
                    }))
                ))
            })
        }
    }
}
