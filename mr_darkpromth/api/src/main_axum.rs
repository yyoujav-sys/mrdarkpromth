use axum::{
    Router, serve,
    routing::get,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use mr_darkpromth_api::axum_router::create_router;
use mr_darkpromth_api::AppState;

#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Starting MR.DarkPromth Axum API");

    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/mrdarkpromth".to_string());
    
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(50)
        .connect(&db_url).await.expect("DB connect failed");

    let state = Arc::new(AppState::new(pool).await);
    let app = create_router(state);
    
    let addr = SocketAddr::from(([0,0,0,0], 8080));
    log::info!("Listening on {}", addr);
    
    let listener = TcpListener::bind(addr).await.unwrap();
    serve(listener, app).await.unwrap();
}
