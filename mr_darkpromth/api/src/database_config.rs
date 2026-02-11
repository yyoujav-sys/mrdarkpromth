// Database configuration and connection pool management
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Database configuration with sensible defaults for production
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    
    /// Minimum number of connections to maintain
    pub min_connections: u32,
    
    /// Timeout for acquiring a connection from the pool
    pub acquire_timeout_secs: u64,
    
    /// Maximum lifetime of a connection
    pub max_lifetime_secs: u64,
    
    /// Idle timeout before a connection is closed
    pub idle_timeout_secs: u64,
    
    /// Enable SSL mode
    pub ssl_mode: String,
    
    /// Connection string
    pub url: String,
}

impl DatabaseConfig {
    /// Create a new database configuration from environment or defaults
    pub fn from_env() -> Self {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/mr_darkpromth".to_string());

        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(50);

        let min_connections = std::env::var("DB_MIN_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let acquire_timeout_secs = std::env::var("DB_ACQUIRE_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let max_lifetime_secs = std::env::var("DB_MAX_LIFETIME")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600); // 10 minutes

        let idle_timeout_secs = std::env::var("DB_IDLE_TIMEOUT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(300); // 5 minutes

        let ssl_mode = std::env::var("DB_SSL_MODE")
            .unwrap_or_else(|_| "prefer".to_string());

        Self {
            max_connections,
            min_connections,
            acquire_timeout_secs,
            max_lifetime_secs,
            idle_timeout_secs,
            ssl_mode,
            url,
        }
    }

    /// Create a database configuration for development
    pub fn development() -> Self {
        Self {
            max_connections: 20,
            min_connections: 5,
            acquire_timeout_secs: 30,
            max_lifetime_secs: 600,
            idle_timeout_secs: 300,
            ssl_mode: "disable".to_string(),
            url: "postgresql://postgres:postgres@localhost:5432/mr_darkpromth".to_string(),
        }
    }

    /// Create a database configuration for production
    pub fn production() -> Self {
        Self {
            max_connections: 100,
            min_connections: 20,
            acquire_timeout_secs: 60,
            max_lifetime_secs: 900,
            idle_timeout_secs: 600,
            ssl_mode: "require".to_string(),
            url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/mr_darkpromth".to_string()),
        }
    }

    /// Create a connection pool with these settings
    pub async fn create_pool(&self) -> Result<PgPool, sqlx::Error> {
        log::info!(
            "Creating database pool: max={}, min={}, acquire_timeout={}s, max_lifetime={}s, idle_timeout={}s",
            self.max_connections,
            self.min_connections,
            self.acquire_timeout_secs,
            self.max_lifetime_secs,
            self.idle_timeout_secs
        );

        let pool = PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(Duration::from_secs(self.acquire_timeout_secs))
            .max_lifetime(Duration::from_secs(self.max_lifetime_secs))
            .idle_timeout(Duration::from_secs(self.idle_timeout_secs))
            .connect(&self.url)
            .await?;

        log::info!("✅ Database pool created successfully");
        Ok(pool)
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Health check for database connectivity
pub async fn check_database_health(pool: &PgPool) -> Result<DatabaseHealthStatus, String> {
    let start = std::time::Instant::now();
    
    let result = sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await;

    let duration_ms = start.elapsed().as_millis() as u64;

    match result {
        Ok(_) => {
            Ok(DatabaseHealthStatus {
                is_healthy: true,
                response_time_ms: duration_ms,
                total_connections: 0,
                idle_connections: 0,
                error: None,
            })
        }
        Err(e) => {
            Ok(DatabaseHealthStatus {
                is_healthy: false,
                response_time_ms: duration_ms,
                total_connections: 0,
                idle_connections: 0,
                error: Some(e.to_string()),
            })
        }
    }
}

/// Database health status information
#[derive(Debug, Clone, serde::Serialize)]
pub struct DatabaseHealthStatus {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub total_connections: u32,
    pub idle_connections: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_development_config() {
        let config = DatabaseConfig::development();
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.ssl_mode, "disable");
    }

    #[test]
    fn test_production_config() {
        let config = DatabaseConfig::production();
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.ssl_mode, "require");
        assert!(config.max_lifetime_secs >= 900);
    }
}
