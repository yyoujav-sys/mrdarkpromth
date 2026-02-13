use dotenv::dotenv;
use std::net::SocketAddr;
use std::sync::Arc;
use std::io::Write;
use tokio::net::TcpListener;
use sqlx::Executor;

use mr_darkpromth_api::axum_router::create_router;
use mr_darkpromth_api::{AppState, DatabaseConfig};

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Structured logging: JSON format when LOG_FORMAT=json
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_default();
    if log_format == "json" {
        env_logger::Builder::from_default_env()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{{\"timestamp\":\"{}\",\"level\":\"{}\",\"target\":\"{}\",\"message\":\"{}\"}}",
                    chrono::Utc::now().to_rfc3339(),
                    record.level(),
                    record.target(),
                    record.args().to_string().replace('\"', "\\\"")
                )
            })
            .init();
    } else {
        env_logger::init();
    }

    log::info!("🚀 Starting MR.DarkPromth Axum API Gateway");

    // Use database configuration from environment or defaults
    let db_config = DatabaseConfig::from_env();
    let pool = db_config
        .create_pool()
        .await
        .expect("Failed to create database pool");

    log::info!("✅ Database connection established");

    let state = Arc::new(AppState::new(pool).await);

    // Spawn self-correction engine (Agent 8) — lightweight DB health monitor
    {
        let slack = state.slack_service.clone();
        let health_pool = state.pool.clone();
        tokio::spawn(async move {
            log::info!("🤖 Agent 8 (Self-Correction Engine) started — monitoring every 300s");
            // Send startup notification
            let _ = slack.notify_deployment(env!("CARGO_PKG_VERSION"), "success").await;
            
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300));
            loop {
                interval.tick().await;
                // Check DB connectivity
                match sqlx::query("SELECT 1").execute(&health_pool).await {
                    Ok(_) => {
                        log::debug!("Agent 8: DB health check OK");
                    }
                    Err(e) => {
                        log::error!("Agent 8: DB health check FAILED: {}", e);
                        let _ = slack.notify_error("Agent8/DB", &format!("DB health check failed: {}", e)).await;
                    }
                }
            }
        });
    }

    let app = create_router(state);
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("Invalid PORT number");
    let addr = SocketAddr::from(([0,0,0,0], port));
    log::info!("Listening on {}", addr);
    
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.expect("Server error");
}
