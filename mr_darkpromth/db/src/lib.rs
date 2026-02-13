use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::IpAddr;
use uuid::Uuid;

pub type DbPool = sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub tier: UserTier,
    pub api_key: String,
    pub api_key_expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub language: Option<String>,
    pub github_id: Option<String>,
    pub last_client_type: Option<String>,
    pub is_online: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub use mr_darkpromth_core::tier::UserTier;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub tier: Option<UserTier>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub tier: Option<UserTier>,
    pub is_active: Option<bool>,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub tier: UserTier,
    pub api_key: String,
    pub api_key_expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub language: Option<String>,
    pub github_id: Option<String>,
    pub last_client_type: Option<String>,
    pub is_online: bool,
    pub created_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            tier: user.tier,
            api_key: user.api_key,
            api_key_expires_at: user.api_key_expires_at,
            is_active: user.is_active,
            language: user.language,
            github_id: user.github_id,
            last_client_type: user.last_client_type,
            is_online: user.is_online,
            created_at: user.created_at,
        }
    }
}

pub struct UserRepository {
    pub pool: sqlx::PgPool,
}

impl UserRepository {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_user(&self, request: CreateUserRequest, password_hash: String, api_key: String) -> anyhow::Result<User> {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash, tier, api_key)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(request.username)
        .bind(request.email)
        .bind(password_hash)
        .bind(request.tier.unwrap_or_default())
        .bind(api_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(user)
    }

    pub async fn get_user_by_api_key(&self, api_key: &str) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE api_key = $1 AND is_active = true",
        )
        .bind(api_key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_user(&self, user_id: Uuid, request: UpdateUserRequest) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET username = COALESCE($1, username),
                email = COALESCE($2, email),
                tier = COALESCE($3, tier),
                is_active = COALESCE($4, is_active),
                language = COALESCE($5, language),
                updated_at = NOW()
            WHERE id = $6
            RETURNING *
            "#,
        )
        .bind(request.username)
        .bind(request.email)
        .bind(request.tier)
        .bind(request.is_active)
        .bind(request.language)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_api_key(&self, user_id: Uuid, api_key: String, expires_at: Option<DateTime<Utc>>) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET api_key = $1,
                api_key_expires_at = $2,
                updated_at = NOW()
            WHERE id = $3
            RETURNING *
            "#,
        )
        .bind(api_key)
        .bind(expires_at)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn update_password_hash(&self, user_id: Uuid, password_hash: String) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET password_hash = $1,
                updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(password_hash)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn delete_user(&self, user_id: Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn link_github_account(&self, user_id: Uuid, github_id: String) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET github_id = $1,
                updated_at = NOW()
            WHERE id = $2
            RETURNING *
            "#,
        )
        .bind(github_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn unlink_github_account(&self, user_id: Uuid) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            UPDATE users 
            SET github_id = NULL,
                updated_at = NOW()
            WHERE id = $1
            RETURNING *
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn list_users(&self, limit: i64, offset: i64) -> anyhow::Result<Vec<User>> {
        let users = sqlx::query_as::<_, User>(
            "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(users)
    }

    pub async fn count_users(&self) -> anyhow::Result<i64> {
        let count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }

    pub async fn count_active_users(&self) -> anyhow::Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM users WHERE is_active = true",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    // ==================== Active Session Management ====================

    /// Insert or update an active session for a user
    pub async fn upsert_active_session(
        &self,
        user_id: Uuid,
        client_type: &str,
        client_version: Option<&str>,
        ip_address: Option<IpAddr>,
    ) -> anyhow::Result<Uuid> {
        let ip_str = ip_address.map(|ip| ip.to_string());
        let session_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO user_active_sessions (user_id, client_type, client_version, ip_address)
            VALUES ($1, $2, $3, $4::INET)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(client_type)
        .bind(client_version)
        .bind(ip_str)
        .fetch_one(&self.pool)
        .await?;

        Ok(session_id)
    }

    /// Remove an active session by session ID
    pub async fn delete_active_session(&self, session_id: Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM user_active_sessions WHERE id = $1")
            .bind(session_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Remove all active sessions for a user
    pub async fn delete_all_user_active_sessions(&self, user_id: Uuid) -> anyhow::Result<u64> {
        let result = sqlx::query("DELETE FROM user_active_sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    /// Count active sessions for a user
    pub async fn count_user_active_sessions(&self, user_id: Uuid) -> anyhow::Result<i64> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM user_active_sessions WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    /// Update heartbeat for an active session
    pub async fn heartbeat_active_session(
        &self,
        user_id: Uuid,
        client_type: &str,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE user_active_sessions 
            SET last_activity = NOW()
            WHERE user_id = $1 AND client_type = $2
            "#,
        )
        .bind(user_id)
        .bind(client_type)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    /// Remove active session by user_id and client_type
    pub async fn delete_active_session_by_type(
        &self,
        user_id: Uuid,
        client_type: &str,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM user_active_sessions WHERE user_id = $1 AND client_type = $2",
        )
        .bind(user_id)
        .bind(client_type)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    // ==================== Online Status ====================

    /// Update user online status and last client type
    pub async fn update_user_online_status(
        &self,
        user_id: Uuid,
        is_online: bool,
        last_client_type: Option<&str>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE users 
            SET is_online = $1,
                last_client_type = COALESCE($2, last_client_type),
                updated_at = NOW()
            WHERE id = $3
            "#,
        )
        .bind(is_online)
        .bind(last_client_type)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ==================== Terminal Sessions ====================

    /// Create a new terminal session record
    pub async fn create_terminal_session(
        &self,
        user_id: Uuid,
        session_id: &str,
        client_type: &str,
        command: Option<&str>,
    ) -> anyhow::Result<Uuid> {
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO terminal_sessions (user_id, session_id, client_type, command)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(session_id)
        .bind(client_type)
        .bind(command)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    /// Close a terminal session
    pub async fn close_terminal_session(
        &self,
        session_id: &str,
        exit_code: Option<i32>,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE terminal_sessions 
            SET status = 'closed', 
                exit_code = $1,
                closed_at = NOW()
            WHERE session_id = $2 AND status = 'active'
            "#,
        )
        .bind(exit_code)
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    // ==================== Agent Action Logs ====================

    /// Log an agent action
    pub async fn log_agent_action(
        &self,
        user_id: Uuid,
        session_id: Option<Uuid>,
        action_type: &str,
        content: Option<&str>,
        metadata: Option<serde_json::Value>,
        client_origin: Option<&str>,
        duration_ms: Option<i32>,
    ) -> anyhow::Result<Uuid> {
        let id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO agent_action_logs (user_id, session_id, action_type, content, metadata, client_origin, duration_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(user_id)
        .bind(session_id)
        .bind(action_type)
        .bind(content)
        .bind(metadata.unwrap_or(serde_json::json!({})))
        .bind(client_origin)
        .bind(duration_ms)
        .fetch_one(&self.pool)
        .await?;
        Ok(id)
    }

    /// Get the database pool reference
    pub fn get_pool(&self) -> &sqlx::PgPool {
        &self.pool
    }
}