use actix_web::{body::{BoxBody, MessageBody}, dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpResponse};
use futures::future::{ready, Ready};
use redis::Commands;
use redis::Client;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use log::{info, warn};

#[derive(Clone)]
pub struct RateLimiter {
    redis_client: Option<Arc<Client>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            redis_client: None,
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn with_redis(redis_url: &str, max_requests: usize, window_secs: u64) -> Result<Self, redis::RedisError> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            redis_client: Some(Arc::new(client)),
            max_requests,
            window: Duration::from_secs(window_secs),
        })
    }

    pub async fn check_rate_limit(&self, ip: IpAddr) -> bool {
        if let Some(redis_client) = &self.redis_client {
            self.check_rate_limit_redis(redis_client, ip).await
        } else {
            warn!("Redis not configured, rate limiting disabled");
            true
        }
    }

    async fn check_rate_limit_redis(&self, client: &Client, ip: IpAddr) -> bool {
        let key = format!("rate_limit:{}", ip);
        let window_secs = self.window.as_secs() as usize;
        let max_requests = self.max_requests as i64;

        let mut con = match client.get_connection() {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to connect to Redis: {}, allowing request", e);
                return true;
            }
        };

        let result: Result<i64, _> = con.incr(&key, 1);

        let count = match result {
            Ok(c) => c,
            Err(e) => {
                warn!("Redis INCR failed: {}, allowing request", e);
                return true;
            }
        };

        if count == 1 {
            let _: Result<(), _> = con.expire(&key, window_secs);
        }

        if count > max_requests {
            info!("Rate limit exceeded for IP: {} (count: {})", ip, count);
            false
        } else {
            true
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = RateLimiterMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service: Rc::new(service),
            limiter: self.clone(),
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: Rc<S>,
    limiter: RateLimiter,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
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
        let peer_addr = req.connection_info().peer_addr().map(|addr| addr.to_string());
        let ip = match peer_addr {
            Some(addr_str) => {
                match addr_str.parse::<std::net::SocketAddr>() {
                    Ok(addr) => addr.ip(),
                    Err(_) => {
                        warn!("Unable to parse client IP address: {}", addr_str);
                        return Box::pin(async move {
                            Ok(req.into_response(
                                HttpResponse::InternalServerError().json(serde_json::json!({
                                    "error": "Unable to determine client IP"
                                }))
                            ))
                        });
                    }
                }
            }
            None => {
                warn!("Unable to determine client IP address");
                return Box::pin(async move {
                    Ok(req.into_response(
                        HttpResponse::InternalServerError().json(serde_json::json!({
                            "error": "Unable to determine client IP"
                        }))
                    ))
                });
            }
        };

        let limiter = self.limiter.clone();
        let service = self.service.clone();
        Box::pin(async move {
            if limiter.check_rate_limit(ip).await {
                let fut = service.call(req);
                let res = fut.await?.map_into_boxed_body();
                Ok(res)
            } else {
                info!("Rate limit applied to IP: {}", ip);
                Ok(req.into_response(
                    HttpResponse::TooManyRequests().json(serde_json::json!({
                        "error": "Rate limit exceeded",
                        "message": "Too many requests. Please try again later."
                    }))
                ))
            }
        })
    }
}
