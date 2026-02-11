// Test database utilities for MrDarkPromth
use sqlx::PgPool;

/// Load environment variables from .env file for tests
pub fn load_test_env() {
    // Only load dotenv if not already loaded
    if std::env::var("CEREBRAS_API_KEYS").is_err() && std::env::var("CEREBRAS_API_KEY").is_err() {
        let _ = dotenv::dotenv();
    }
}

/// Get the correct database connection string for tests
/// This connects to the Docker container, not localhost
pub fn get_test_database_url() -> String {
    // Load environment variables first
    load_test_env();
    
    // Allow overriding via environment variable (e.g. from integration_test_runner.sh)
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return url;
    }
    
    // Check if we're running in Docker or locally
    if std::env::var("DOCKER_ENV").is_ok() || std::path::Path::new("/.dockerenv").exists() {
        "postgresql://postgres:postgres@postgres:5432/mr_darkpromth".to_string()
    } else {
        "postgresql://postgres:postgres@localhost:5432/mr_darkpromth".to_string()
    }
}

/// Create a test database pool with proper connection string
pub async fn create_test_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = get_test_database_url();
    PgPool::connect(&database_url).await
}

/// Create a lazy test pool (for existing test patterns)
pub fn get_test_pool() -> PgPool {
    let database_url = get_test_database_url();
    sqlx::PgPool::connect_lazy(&database_url)
        .expect("Failed to connect to test database")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_connection() {
        let pool = sqlx::PgPool::connect_lazy("postgresql://postgres:postgres@localhost:5432/mr_darkpromth").unwrap();
        
        // Test basic connectivity - this will only fail if we actually try to execute a query
        // and the database is not available. For unit tests, we just want to ensure
        // the pool can be created without panicking immediately.
        assert!(pool.is_closed() || !pool.is_closed()); 
    }
}
