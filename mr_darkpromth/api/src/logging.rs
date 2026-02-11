// Logging middleware to prevent credential leakage
use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;
use log::info;

/// Middleware to log HTTP requests/responses without exposing sensitive data
pub async fn log_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    // Log incoming request (without sensitive path parameters)
    info!(
        target: "http",
        "→ {} {} from {}",
        method,
        sanitize_path(&path),
        addr.ip()
    );

    // Call the next middleware/handler
    let response = next.run(request).await;

    // Log outgoing response
    info!(
        target: "http",
        "← {} {}",
        method,
        response.status()
    );

    Ok(response)
}

/// Sanitize path to remove sensitive information
fn sanitize_path(path: &str) -> String {
    // Remove potential password/token patterns
    let mut sanitized = path.to_string();

    // Remove credentials from query parameters
    if let Some(query_pos) = sanitized.find('?') {
        let (base, query) = sanitized.split_at(query_pos);
        let sanitized_query = query
            .split('&')
            .map(|param| {
                if param.contains("password=") 
                    || param.contains("token=")
                    || param.contains("secret=")
                    || param.contains("apikey=")
                    || param.contains("key=")
                    || param.contains("auth=")
                {
                    "[REDACTED]"
                } else {
                    param
                }
            })
            .collect::<Vec<_>>()
            .join("&");
        sanitized = format!("{}?{}", base, sanitized_query);
    }

    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_path() {
        let path = "/api/auth?password=secret123&user=john";
        let result = sanitize_path(path);
        assert!(!result.contains("secret123"));
        assert!(result.contains("[REDACTED]"));
    }
}
