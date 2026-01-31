use log::info;
use mr_darkpromth_api::ServerConfig;
use mr_darkpromth_api::Server;
use std::env;
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    info!("🚀 Starting MR.DarkPromth API Gateway");
    info!("🔓 User Management & Authentication System Enabled");
    info!("🔓 Jailbreak & Ultra Tier Integration Enabled");

    // Get database URL from environment
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost/mrdarkpromth".to_string());

    // Get JWT secret from environment
    let jwt_secret = env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string());

    // Create database connection pool with optimized settings
    let pool = PgPoolOptions::new()
        .max_connections(50) // Increased for scalability
        .min_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600))
        .max_lifetime(std::time::Duration::from_secs(1800))
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    info!("✅ Database connection established");

    let config = ServerConfig::default();

    info!("Configuration:");
    info!("  Host: {}", config.host);
    info!("  Port: {}", config.port);
    info!("  Framework: actix-web");
    info!("  Auth API: /api/auth/*");
    info!("  User API: /api/users/*");
    info!("  Jailbreak API: /api/jailbreak/*");

    // Create and start server with database pool and JWT secret
    let server = Server::new(pool, jwt_secret);
    server.start(config).await
}
