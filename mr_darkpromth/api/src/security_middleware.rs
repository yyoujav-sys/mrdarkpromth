use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};

pub struct SecurityHeaders;

impl<S, B> Transform<S, ServiceRequest> for SecurityHeaders
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = SecurityHeadersMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SecurityHeadersMiddleware { service }))
    }
}

pub struct SecurityHeadersMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for SecurityHeadersMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        Box::pin(async move {
            let mut res = fut.await?;

            let headers = res.headers_mut();
            headers.insert(
                actix_web::http::header::STRICT_TRANSPORT_SECURITY,
                actix_web::http::header::HeaderValue::from_static("max-age=31536000; includeSubDomains"),
            );
            headers.insert(
                actix_web::http::header::X_CONTENT_TYPE_OPTIONS,
                actix_web::http::header::HeaderValue::from_static("nosniff"),
            );
            headers.insert(
                actix_web::http::header::X_FRAME_OPTIONS,
                actix_web::http::header::HeaderValue::from_static("SAMEORIGIN"),
            );
            headers.insert(
                actix_web::http::header::X_XSS_PROTECTION,
                actix_web::http::header::HeaderValue::from_static("1; mode=block"),
            );
            headers.insert(
                actix_web::http::header::CONTENT_SECURITY_POLICY,
                actix_web::http::header::HeaderValue::from_static("default-src 'self'; script-src 'self'; object-src 'none';"),
            );
            headers.insert(
                actix_web::http::header::REFERRER_POLICY,
                actix_web::http::header::HeaderValue::from_static("strict-origin-when-cross-origin"),
            );

            Ok(res)
        })
    }
}
