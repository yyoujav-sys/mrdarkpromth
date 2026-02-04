use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

use mr_darkpromth_api::axum_router::create_router;
use mr_darkpromth_api::AppState;

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("🚀 Starting MR.DarkPromth Axum API Gateway");

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/mrdarkpromth".to_string());
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&db_url).await.expect("DB connect failed");

    log::info!("✅ Database connection established");

    let state = Arc::new(AppState::new(pool).await);
    let app = create_router(state);
    
    let addr = SocketAddr::from(([0,0,0,0], 8080));
    log::info!("Listening on {}", addr);
    
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
