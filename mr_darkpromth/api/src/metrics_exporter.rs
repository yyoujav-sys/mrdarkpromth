use prometheus::{Encoder, TextEncoder, Registry, Counter, Gauge, Histogram, HistogramOpts, Opts};
use lazy_static::lazy_static;
use axum::{
    http::StatusCode,
    response::IntoResponse,
};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();

    pub static ref HTTP_REQUESTS_TOTAL: Counter = Counter::with_opts(
        Opts::new("http_requests_total", "Total number of HTTP requests")
    ).expect("Can't create counter");

    pub static ref HTTP_REQUEST_DURATION_SECONDS: Histogram = Histogram::with_opts(
        HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds")
    ).expect("Can't create histogram");

    pub static ref ACTIVE_USERS: Gauge = Gauge::with_opts(
        Opts::new("active_users", "Number of active users")
    ).expect("Can't create gauge");

    pub static ref SECURITY_VIOLATIONS_TOTAL: Counter = Counter::with_opts(
        Opts::new("security_violations_total", "Total number of security violations")
    ).expect("Can't create counter");

    pub static ref JAILBREAK_ATTEMPTS_TOTAL: Counter = Counter::with_opts(
        Opts::new("jailbreak_attempts_total", "Total number of jailbreak attempts")
    ).expect("Can't create counter");

    pub static ref JAILBREAK_SUCCESS_TOTAL: Counter = Counter::with_opts(
        Opts::new("jailbreak_success_total", "Total number of successful jailbreaks")
    ).expect("Can't create counter");

    pub static ref DB_CONNECTIONS_ACTIVE: Gauge = Gauge::with_opts(
        Opts::new("db_connections_active", "Number of active database connections")
    ).expect("Can't create gauge");

    pub static ref REDIS_OPERATIONS_TOTAL: Counter = Counter::with_opts(
        Opts::new("redis_operations_total", "Total number of Redis operations")
    ).expect("Can't create counter");
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(HTTP_REQUESTS_TOTAL.clone())).expect("Can't register counter");
    REGISTRY.register(Box::new(HTTP_REQUEST_DURATION_SECONDS.clone())).expect("Can't register histogram");
    REGISTRY.register(Box::new(ACTIVE_USERS.clone())).expect("Can't register gauge");
    REGISTRY.register(Box::new(SECURITY_VIOLATIONS_TOTAL.clone())).expect("Can't register counter");
    REGISTRY.register(Box::new(JAILBREAK_ATTEMPTS_TOTAL.clone())).expect("Can't register counter");
    REGISTRY.register(Box::new(JAILBREAK_SUCCESS_TOTAL.clone())).expect("Can't register counter");
    REGISTRY.register(Box::new(DB_CONNECTIONS_ACTIVE.clone())).expect("Can't register gauge");
    REGISTRY.register(Box::new(REDIS_OPERATIONS_TOTAL.clone())).expect("Can't register counter");
}

pub async fn metrics_handler() -> impl IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = REGISTRY.gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        buffer,
    )
}
