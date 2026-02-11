// MR.DarkPromth Secure Session Management and Token Validation System
// Phase 2: Safety and Security Implementation

use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("Invalid session token: {0}")]
    InvalidToken(String),
    #[error("Session expired: {0}")]
    SessionExpired(String),
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Session revoked: {0}")]
    SessionRevoked(String),
    #[error("Token generation failed: {0}")]
    TokenGenerationFailed(String),
    #[error("Invalid session state: {0}")]
    InvalidSessionState(String),
    #[error("Concurrent session limit exceeded: {0}")]
    ConcurrentSessionLimit(String),
    #[error("Suspicious session activity: {0}")]
    SuspiciousActivity(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub jwt_secret: String,
    pub access_token_duration: Duration,
    pub refresh_token_duration: Duration,
    pub max_concurrent_sessions: u8,
    pub enable_session_rotation: bool,
    pub enable_ip_binding: bool,
    pub enable_user_agent_binding: bool,
    pub enable_device_fingerprinting: bool,
    pub session_timeout_inactive: Duration,
    pub cleanup_interval: Duration,
    pub max_failed_attempts: u8,
    pub lockout_duration: Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "your-super-secret-jwt-key-change-in-production".to_string(),
            access_token_duration: Duration::minutes(15),
            refresh_token_duration: Duration::days(7),
            max_concurrent_sessions: 3,
            enable_session_rotation: true,
            enable_ip_binding: true,
            enable_user_agent_binding: true,
            enable_device_fingerprinting: true,
            session_timeout_inactive: Duration::hours(2),
            cleanup_interval: Duration::minutes(30),
            max_failed_attempts: 5,
            lockout_duration: Duration::minutes(15),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub exp: i64,    // Expiration time
    pub iat: i64,    // Issued at
    pub jti: String, // JWT ID (session ID)
    pub iss: String, // Issuer
    pub aud: String, // Audience
    pub session_type: SessionType,
    pub user_tier: String,
    pub permissions: Vec<String>,
    pub device_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent_hash: Option<String>,
    pub is_refresh_token: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    User,
    API,
    Admin,
    Service,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_type: SessionType,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
    pub is_revoked: bool,
    pub device_id: Option<String>,
    pub device_fingerprint: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub user_agent_hash: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub failed_attempts: u8,
    pub locked_until: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionValidationResult {
    pub is_valid: bool,
    pub session: Option<Session>,
    pub claims: Option<Claims>,
    pub warnings: Vec<String>,
    pub should_rotate: bool,
    pub should_refresh: bool,
}

#[derive(Clone)]
pub struct SessionManager {
    config: SessionConfig,
    sessions: HashMap<String, Session>,
    user_sessions: HashMap<Uuid, HashSet<String>>, // user_id -> session_ids
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl SessionManager {
    pub fn new(config: SessionConfig) -> Result<Self, SessionError> {
        let encoding_key = EncodingKey::from_secret(config.jwt_secret.as_ref());
        let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_ref());
        
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.set_issuer(&["mr-darkpromth"]);
        validation.set_audience(&["mr-darkpromth-users"]);

        Ok(Self {
            config,
            sessions: HashMap::new(),
            user_sessions: HashMap::new(),
            encoding_key,
            decoding_key,
            validation,
        })
    }

    pub fn create_session(
        &mut self,
        user_id: Uuid,
        user_tier: &crate::UserTier,
        permissions: Vec<String>,
        ip_address: String,
        user_agent: String,
        device_id: Option<String>,
    ) -> Result<(String, String, Session), SessionError> {
        // Check concurrent session limit
        self.check_concurrent_sessions(&user_id)?;

        // Generate session ID
        let session_id = Uuid::new_v4();
        let now = Utc::now();

        // Generate device fingerprint if enabled
        let device_fingerprint = if self.config.enable_device_fingerprinting {
            Some(self.generate_device_fingerprint(&ip_address, &user_agent))
        } else {
            None
        };

        // Generate user agent hash
        let user_agent_hash = self.hash_user_agent(&user_agent);

        // Create session record
        let session = Session {
            id: session_id,
            user_id,
            session_type: SessionType::User,
            created_at: now,
            last_accessed: now,
            expires_at: now + self.config.refresh_token_duration,
            is_active: true,
            is_revoked: false,
            device_id,
            device_fingerprint,
            ip_address: ip_address.clone(),
            user_agent: user_agent.clone(),
            user_agent_hash: user_agent_hash.clone(),
            access_token: None,
            refresh_token: None,
            failed_attempts: 0,
            locked_until: None,
            metadata: HashMap::new(),
        };

        // Generate access token
        let access_token = self.generate_token(
            &session,
            user_tier,
            &permissions,
            false,
            now + self.config.access_token_duration,
        )?;

        // Generate refresh token
        let refresh_token = self.generate_token(
            &session,
            user_tier,
            &permissions,
            true,
            now + self.config.refresh_token_duration,
        )?;

        // Store session
        let session_id_str = session_id.to_string();
        self.sessions.insert(session_id_str.clone(), session.clone());
        
        // Update user sessions mapping
        self.user_sessions
            .entry(user_id)
            .or_default()
            .insert(session_id_str.clone());

        Ok((access_token, refresh_token, session))
    }

    pub fn validate_session(&mut self, token: &str, ip_address: &str, user_agent: &str) -> Result<SessionValidationResult, SessionError> {
        let mut result = SessionValidationResult {
            is_valid: false,
            session: None,
            claims: None,
            warnings: Vec::new(),
            should_rotate: false,
            should_refresh: false,
        };

        // Decode token
        let token_data = decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| SessionError::InvalidToken(e.to_string()))?;

        let claims = token_data.claims;
        result.claims = Some(claims.clone());

        // Get session
        let session = self.sessions.get(&claims.jti)
            .ok_or_else(|| SessionError::SessionNotFound(claims.jti.clone()))?
            .clone();

        // Check if session is active
        if !session.is_active {
            return Err(SessionError::SessionRevoked(claims.jti));
        }

        // Check if session is revoked
        if session.is_revoked {
            return Err(SessionError::SessionRevoked(claims.jti));
        }

        // Check expiration
        let now = Utc::now();
        if now > session.expires_at {
            return Err(SessionError::SessionExpired(claims.jti));
        }

        // Check if session is locked
        if let Some(locked_until) = session.locked_until {
            if now < locked_until {
                return Err(SessionError::InvalidSessionState("Session is locked".to_string()));
            }
        }

        // IP binding check
        if self.config.enable_ip_binding
            && session.ip_address.as_str() != ip_address {
                result.warnings.push("IP address mismatch detected".to_string());
                // For security, we might want to invalidate the session
                return Err(SessionError::SuspiciousActivity("IP address mismatch".to_string()));
            }

        // User agent binding check
        if self.config.enable_user_agent_binding {
            let current_user_agent_hash = self.hash_user_agent(user_agent);
            if session.user_agent_hash != current_user_agent_hash {
                result.warnings.push("User agent mismatch detected".to_string());
                return Err(SessionError::SuspiciousActivity("User agent mismatch".to_string()));
            }
        }

        // Device fingerprint check
        if let Some(session_fingerprint) = &session.device_fingerprint {
            let current_fingerprint = self.generate_device_fingerprint(ip_address, user_agent);
            if session_fingerprint != &current_fingerprint {
                result.warnings.push("Device fingerprint mismatch detected".to_string());
                return Err(SessionError::SuspiciousActivity("Device fingerprint mismatch".to_string()));
            }
        }

        // Check if token should be rotated
        if self.config.enable_session_rotation && !claims.is_refresh_token {
            let token_age = now - DateTime::from_timestamp(claims.iat, 0).unwrap();
            if token_age > Duration::minutes(10) {
                result.should_rotate = true;
            }
        }

        // Check if session should be refreshed
        let time_until_expiry = session.expires_at - now;
        if time_until_expiry < Duration::hours(1) {
            result.should_refresh = true;
        }

        // Update last accessed time
        self.update_session_access(&claims.jti);

        result.is_valid = true;
        result.session = Some(session);

        Ok(result)
    }

    pub fn refresh_session(&mut self, refresh_token: &str, ip_address: &str, user_agent: &str) -> Result<(String, Session), SessionError> {
        // Validate refresh token
        let result = self.validate_session(refresh_token, ip_address, user_agent)?;
        
        if !result.claims.as_ref().unwrap().is_refresh_token {
            return Err(SessionError::InvalidToken("Not a refresh token".to_string()));
        }

        let session = result.session.unwrap();
        let claims = result.claims.unwrap();

        // Generate new access token
        let now = Utc::now();
        let new_access_token = self.generate_token(
            &session,
            &crate::UserTier::Ultra, // This should be extracted from claims
            &claims.permissions,
            false,
            now + self.config.access_token_duration,
        )?;

        // Update session
        self.update_session_access(&claims.jti);

        Ok((new_access_token, session))
    }

    pub fn revoke_session(&mut self, session_id: &str) -> Result<(), SessionError> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| SessionError::SessionNotFound(session_id.to_string()))?;

        session.is_revoked = true;
        session.is_active = false;

        // Remove from user sessions mapping
        if let Some(user_sessions) = self.user_sessions.get_mut(&session.user_id) {
            user_sessions.remove(session_id);
        }

        Ok(())
    }

    pub fn revoke_all_user_sessions(&mut self, user_id: &Uuid) -> Result<Vec<String>, SessionError> {
        let session_ids = self.user_sessions.get(user_id).cloned()
            .unwrap_or_default();

        let mut revoked_sessions = Vec::new();

        for session_id in &session_ids {
            if let Some(session) = self.sessions.get_mut(session_id) {
                session.is_revoked = true;
                session.is_active = false;
                revoked_sessions.push(session_id.clone());
            }
        }

        // Clear user sessions mapping
        self.user_sessions.remove(user_id);

        Ok(revoked_sessions)
    }

    pub fn record_failed_attempt(&mut self, session_id: &str) -> Result<(), SessionError> {
        let session = self.sessions.get_mut(session_id)
            .ok_or_else(|| SessionError::SessionNotFound(session_id.to_string()))?;

        session.failed_attempts += 1;

        // Lock session if too many failed attempts
        if session.failed_attempts >= self.config.max_failed_attempts {
            session.locked_until = Some(Utc::now() + self.config.lockout_duration);
        }

        Ok(())
    }

    pub fn cleanup_expired_sessions(&mut self) -> Vec<String> {
        let now = Utc::now();
        let mut expired_sessions = Vec::new();

        self.sessions.retain(|session_id, session| {
            let is_expired = now > session.expires_at || 
                           (session.locked_until.is_some_and(|locked| now < locked));

            if is_expired {
                expired_sessions.push(session_id.clone());
                
                // Remove from user sessions mapping
                if let Some(user_sessions) = self.user_sessions.get_mut(&session.user_id) {
                    user_sessions.remove(session_id);
                }
            }

            !is_expired
        });

        expired_sessions
    }

    pub fn get_user_sessions(&self, user_id: &Uuid) -> Vec<&Session> {
        self.user_sessions.get(user_id)
            .map(|session_ids| {
                session_ids.iter()
                    .filter_map(|session_id| self.sessions.get(session_id))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_active_sessions_count(&self) -> usize {
        self.sessions.values()
            .filter(|session| session.is_active && !session.is_revoked)
            .count()
    }

    fn check_concurrent_sessions(&self, user_id: &Uuid) -> Result<(), SessionError> {
        let active_sessions = self.get_user_sessions(user_id)
            .iter()
            .filter(|session| session.is_active && !session.is_revoked)
            .count();

        if active_sessions >= self.config.max_concurrent_sessions as usize {
            return Err(SessionError::ConcurrentSessionLimit(
                format!("Maximum concurrent sessions ({}) exceeded", self.config.max_concurrent_sessions)
            ));
        }

        Ok(())
    }

    fn generate_token(
        &self,
        session: &Session,
        user_tier: &crate::UserTier,
        permissions: &[String],
        is_refresh_token: bool,
        expires_at: DateTime<Utc>,
    ) -> Result<String, SessionError> {
        let claims = Claims {
            sub: session.user_id.to_string(),
            exp: expires_at.timestamp(),
            iat: Utc::now().timestamp(),
            jti: session.id.to_string(),
            iss: "mr-darkpromth".to_string(),
            aud: "mr-darkpromth-users".to_string(),
            session_type: session.session_type.clone(),
            user_tier: format!("{:?}", user_tier),
            permissions: permissions.to_vec(),
            device_id: session.device_id.clone(),
            ip_address: Some(session.ip_address.clone()),
            user_agent_hash: Some(session.user_agent_hash.clone()),
            is_refresh_token,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| SessionError::TokenGenerationFailed(e.to_string()))
    }

    fn generate_device_fingerprint(&self, ip_address: &str, user_agent: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(ip_address.as_bytes());
        hasher.update(user_agent.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn hash_user_agent(&self, user_agent: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(user_agent.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn update_session_access(&mut self, session_id: &str) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.last_accessed = Utc::now();
        }
    }

    pub fn get_session_stats(&self) -> SessionStats {
        let now = Utc::now();
        let active_sessions = self.sessions.values()
            .filter(|session| session.is_active && !session.is_revoked)
            .count();
        
        let expired_sessions = self.sessions.values()
            .filter(|session| now > session.expires_at)
            .count();

        let revoked_sessions = self.sessions.values()
            .filter(|session| session.is_revoked)
            .count();

        let locked_sessions = self.sessions.values()
            .filter(|session| session.locked_until.is_some())
            .count();

        SessionStats {
            total_sessions: self.sessions.len(),
            active_sessions,
            expired_sessions,
            revoked_sessions,
            locked_sessions,
            unique_users: self.user_sessions.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub expired_sessions: usize,
    pub revoked_sessions: usize,
    pub locked_sessions: usize,
    pub unique_users: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UserTier;

    #[test]
    fn test_session_creation() {
        let config = SessionConfig::default();
        let mut manager = SessionManager::new(config).unwrap();

        let user_id = Uuid::new_v4();
        let (access_token, refresh_token, session) = manager.create_session(
            user_id,
            &UserTier::Free,
            vec!["read".to_string()],
            "127.0.0.1".to_string(),
            "Mozilla/5.0".to_string(),
            None,
        ).unwrap();

        assert!(!access_token.is_empty());
        assert!(!refresh_token.is_empty());
        assert_eq!(session.user_id, user_id);
        assert!(session.is_active);
    }

    #[test]
    fn test_session_validation() {
        let config = SessionConfig::default();
        let mut manager = SessionManager::new(config).unwrap();

        let user_id = Uuid::new_v4();
        let (access_token, _, _) = manager.create_session(
            user_id,
            &UserTier::Free,
            vec!["read".to_string()],
            "127.0.0.1".to_string(),
            "Mozilla/5.0".to_string(),
            None,
        ).unwrap();

        let result = manager.validate_session(&access_token, "127.0.0.1", "Mozilla/5.0").unwrap();
        assert!(result.is_valid);
        assert!(result.session.is_some());
    }

    #[test]
    fn test_session_revocation() {
        let config = SessionConfig::default();
        let mut manager = SessionManager::new(config).unwrap();

        let user_id = Uuid::new_v4();
        let (access_token, _, session) = manager.create_session(
            user_id,
            &UserTier::Free,
            vec!["read".to_string()],
            "127.0.0.1".to_string(),
            "Mozilla/5.0".to_string(),
            None,
        ).unwrap();

        manager.revoke_session(&session.id.to_string()).unwrap();

        let result = manager.validate_session(&access_token, "127.0.0.1", "Mozilla/5.0");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SessionError::SessionRevoked(_)));
    }

    #[test]
    fn test_concurrent_session_limit() {
        let mut config = SessionConfig::default();
        config.max_concurrent_sessions = 2;
        let mut manager = SessionManager::new(config).unwrap();

        let user_id = Uuid::new_v4();

        // Create first session
        manager.create_session(
            user_id,
            &UserTier::Free,
            vec!["read".to_string()],
            "127.0.0.1".to_string(),
            "Mozilla/5.0".to_string(),
            None,
        ).unwrap();

        // Create second session
        manager.create_session(
            user_id,
            &UserTier::Free,
            vec!["read".to_string()],
            "127.0.0.1".to_string(),
            "Chrome/90.0".to_string(),
            None,
        ).unwrap();

        // Third session should fail
        let result = manager.create_session(
            user_id,
            &UserTier::Free,
            vec!["read".to_string()],
            "127.0.0.1".to_string(),
            "Safari/14.0".to_string(),
            None,
        );

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SessionError::ConcurrentSessionLimit(_)));
    }

    #[test]
    fn test_ip_binding() {
        let mut config = SessionConfig::default();
        config.enable_ip_binding = true;
        let mut manager = SessionManager::new(config).unwrap();

        let user_id = Uuid::new_v4();
        let (access_token, _, _) = manager.create_session(
            user_id,
            &UserTier::Free,
            vec!["read".to_string()],
            "127.0.0.1".to_string(),
            "Mozilla/5.0".to_string(),
            None,
        ).unwrap();

        // Different IP should fail
        let result = manager.validate_session(&access_token, "192.168.1.1", "Mozilla/5.0");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SessionError::SuspiciousActivity(_)));
    }
}
