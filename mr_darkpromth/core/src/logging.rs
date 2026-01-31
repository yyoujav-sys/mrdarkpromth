use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging(level: &str, format: &str) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));

    let subscriber = tracing_subscriber::registry()
        .with(env_filter);

    if format == "json" {
        subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        subscriber
            .with(tracing_subscriber::fmt::layer().pretty())
            .init();
    }
}

pub fn init_file_logging(level: &str, format: &str, file_path: &str) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));

    let file_appender = tracing_appender::rolling::daily(file_path, "app.log");
    
    let subscriber = tracing_subscriber::registry()
        .with(env_filter);

    if format == "json" {
        subscriber
            .with(tracing_subscriber::fmt::layer().json().with_writer(file_appender))
            .init();
    } else {
        subscriber
            .with(tracing_subscriber::fmt::layer().pretty().with_writer(file_appender))
            .init();
    }
}

#[macro_export]
macro_rules! log_agent_action {
    ($agent_id:expr, $action:expr, $details:expr, $status:expr) => {
        tracing::info!(
            agent_id = %$agent_id,
            action = %$action,
            details = %$details,
            status = %$status,
            "Agent action logged"
        );
    };
}

#[macro_export]
macro_rules! log_redis_event {
    ($event_type:expr, $agent_id:expr, $payload:expr) => {
        tracing::info!(
            event_type = %$event_type,
            agent_id = %$agent_id,
            payload = %$payload,
            "Redis event published"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($context:expr, $error:expr) => {
        tracing::error!(
            context = %$context,
            error = %$error,
            "Error occurred"
        );
    };
}

#[macro_export]
macro_rules! log_warning {
    ($context:expr, $message:expr) => {
        tracing::warn!(
            context = %$context,
            message = %$message,
            "Warning logged"
        );
    };
}
