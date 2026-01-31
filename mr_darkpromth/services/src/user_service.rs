use anyhow::Result;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
pub use mr_darkpromth_db::{CreateUserRequest, UpdateUserRequest, UserResponse};
use mr_darkpromth_db::{User, UserRepository, UserTier};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // user_id
    pub username: String,
    pub email: String,
    pub tier: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: UserResponse,
    pub token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub api_key: String,
    pub expires_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("User not found")]
    UserNotFound,
    #[error("User already exists")]
    UserAlreadyExists,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Insufficient permissions")]
    InsufficientPermissions,
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Hash error: {0}")]
    HashError(String),
    #[error("JWT error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<anyhow::Error> for AuthError {
    fn from(error: anyhow::Error) -> Self {
        AuthError::InternalError(error.to_string())
    }
}

pub struct UserService {
    repository: UserRepository,
    jwt_secret: String,
    argon2: Argon2<'static>,
}

impl UserService {
    pub fn new(repository: UserRepository, jwt_secret: String) -> Self {
        Self {
            repository,
            jwt_secret,
            argon2: Argon2::default(),
        }
    }

    pub async fn register(&self, request: RegisterRequest) -> Result<AuthResponse, AuthError> {
        // Check if user already exists
        if let Some(_) = self.repository.get_user_by_email(&request.email).await? {
            return Err(AuthError::UserAlreadyExists);
        }

        if let Some(_) = self.repository.get_user_by_username(&request.username).await? {
            return Err(AuthError::UserAlreadyExists);
        }

        // Validate input
        self.validate_registration_input(&request)?;

        // Hash password
        let password_hash = self.hash_password(&request.password)?;

        // Generate API key
        let api_key = self.generate_api_key();

        // Create user
        let create_request = CreateUserRequest {
            username: request.username,
            email: request.email,
            password: String::new(), // password is not stored in plain text
            tier: None, // default to Free tier
        };

        let user = self.repository.create_user(create_request, password_hash, api_key.clone()).await?;

        // Generate JWT token
        let token = self.generate_token(&user)?;

        Ok(AuthResponse {
            user: user.into(),
            token,
            expires_in: 24 * 60 * 60, // 24 hours
        })
    }

    pub async fn login(&self, request: LoginRequest) -> Result<AuthResponse, AuthError> {
        // Find user by email
        let user = self.repository
            .get_user_by_email(&request.email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password
        if !self.verify_password(&request.password, &user.password_hash)? {
            return Err(AuthError::InvalidCredentials);
        }

        // Check if user is active
        if !user.is_active {
            return Err(AuthError::InvalidCredentials);
        }

        // Generate JWT token
        let token = self.generate_token(&user)?;

        Ok(AuthResponse {
            user: user.into(),
            token,
            expires_in: 24 * 60 * 60, // 24 hours
        })
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<Option<UserResponse>, AuthError> {
        let user = self.repository.get_user_by_id(user_id).await?;
        Ok(user.map(UserResponse::from))
    }

    pub async fn update_user(&self, user_id: Uuid, request: UpdateUserRequest) -> Result<Option<UserResponse>, AuthError> {
        let user = self.repository.update_user(user_id, request).await?;
        Ok(user.map(UserResponse::from))
    }

    pub async fn regenerate_api_key(&self, user_id: Uuid) -> Result<ApiKeyResponse, AuthError> {
        let api_key = self.generate_api_key();
        let expires_at = Some(Utc::now() + Duration::days(30)); // 30 days expiration

        let user = self.repository.update_api_key(user_id, api_key.clone(), expires_at).await?;
        
        if user.is_none() {
            return Err(AuthError::UserNotFound);
        }

        Ok(ApiKeyResponse {
            api_key,
            expires_at,
        })
    }

    pub async fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    pub async fn validate_api_key(&self, api_key: &str) -> Result<Option<User>, AuthError> {
        let user = self.repository.get_user_by_api_key(api_key).await?;
        
        // Check if API key is expired
        if let Some(user) = &user {
            if let Some(expires_at) = user.api_key_expires_at {
                if expires_at < Utc::now() {
                    return Ok(None);
                }
            }
        }

        Ok(user)
    }

    pub fn check_tier_permission(&self, user_tier: UserTier, required_tier: UserTier) -> bool {
        match (user_tier, required_tier) {
            (UserTier::Ultra, _) => true, // Ultra tier has access to everything
            (UserTier::Premium, UserTier::Free | UserTier::Premium) => true,
            (UserTier::Premium, UserTier::Ultra) => false,
            (UserTier::Free, UserTier::Free) => true, // Free tier can access free features
            (UserTier::Free, UserTier::Premium | UserTier::Ultra) => false,
        }
    }

    fn validate_registration_input(&self, request: &RegisterRequest) -> Result<(), AuthError> {
        if request.username.len() < 3 {
            return Err(AuthError::InvalidCredentials);
        }

        if request.username.len() > 50 {
            return Err(AuthError::InvalidCredentials);
        }

        if !request.email.contains('@') || request.email.len() > 255 {
            return Err(AuthError::InvalidCredentials);
        }

        if request.password.len() < 8 {
            return Err(AuthError::InvalidCredentials);
        }

        Ok(())
    }

    fn hash_password(&self, password: &str) -> Result<String, AuthError> {
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = self.argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        Ok(password_hash.to_string())
    }

    fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AuthError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AuthError::HashError(e.to_string()))?;
        Ok(self.argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    fn generate_token(&self, user: &User) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + Duration::hours(24); // 24 hour expiration

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            email: user.email.clone(),
            tier: user.tier.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_ref()),
        )?;

        Ok(token)
    }

    fn generate_api_key(&self) -> String {
        format!("mr_{}", Uuid::new_v4().to_string().replace("-", ""))
    }

    pub async fn list_users(&self, limit: i64, offset: i64) -> Result<Vec<UserResponse>, AuthError> {
        let users = self.repository.list_users(limit, offset).await?;
        Ok(users.into_iter().map(UserResponse::from).collect())
    }

    pub async fn delete_user(&self, user_id: Uuid) -> Result<bool, AuthError> {
        self.repository.delete_user(user_id).await.map_err(AuthError::from)
    }

    pub async fn count_users(&self) -> Result<i64, AuthError> {
        self.repository.count_users().await.map_err(AuthError::from)
    }

    pub async fn count_active_users(&self) -> Result<i64, AuthError> {
        self.repository
            .count_active_users()
            .await
            .map_err(AuthError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mr_darkpromth_db::UserRepository;

    #[tokio::test]
    async fn test_password_hashing() {
        let pool = sqlx::PgPool::connect("postgresql://test_user:test_pass@localhost/test_db").await.unwrap();
        let service = UserService::new(UserRepository::new(pool), "test_secret".to_string());

        let password = "test_password_123";
        let hash = service.hash_password(password).unwrap();
        
        assert!(service.verify_password(password, &hash).unwrap());
        assert!(!service.verify_password("wrong_password", &hash).unwrap());
    }

    #[tokio::test]
    async fn test_api_key_generation() {
        let pool = sqlx::PgPool::connect("postgresql://test_user:test_pass@localhost/test_db").await.unwrap();
        let service = UserService::new(UserRepository::new(pool), "test_secret".to_string());

        let api_key1 = service.generate_api_key();
        let api_key2 = service.generate_api_key();
        
        assert_ne!(api_key1, api_key2);
        assert!(api_key1.starts_with("mr_"));
        assert!(api_key2.starts_with("mr_"));
    }

    #[test]
    fn test_tier_permissions() {
        let pool = sqlx::PgPool::connect_lazy("postgresql://test_user:test_pass@localhost/test_db").unwrap();
        let service = UserService::new(UserRepository::new(pool), "test_secret".to_string());

        // Ultra tier should have access to everything
        assert!(service.check_tier_permission(UserTier::Ultra, UserTier::Free));
        assert!(service.check_tier_permission(UserTier::Ultra, UserTier::Ultra));

        // Free tier should only have access to free features
        assert!(service.check_tier_permission(UserTier::Free, UserTier::Free));
        assert!(!service.check_tier_permission(UserTier::Free, UserTier::Ultra));
    }
}
