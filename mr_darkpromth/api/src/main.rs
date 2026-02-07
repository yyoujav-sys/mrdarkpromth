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
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/mr_darkpromth".to_string());
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(50)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&db_url).await.expect("DB connect failed");

    log::info!("✅ Database connection established");

    let state = Arc::new(AppState::new(pool).await);
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
