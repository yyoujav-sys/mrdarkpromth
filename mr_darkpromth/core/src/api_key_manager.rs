// MR.DarkPromth Secure API Key Management and Rotation System
// Phase 2: Safety and Security Implementation

use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use chrono::{DateTime, Duration, Utc};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ApiKeyError {
    #[error("API key not found: {0}")]
    KeyNotFound(String),
    #[error("API key expired: {0}")]
    KeyExpired(String),
    #[error("API key revoked: {0}")]
    KeyRevoked(String),
    #[error("Invalid API key format: {0}")]
    InvalidFormat(String),
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key generation failed: {0}")]
    KeyGenerationFailed(String),
    #[error("Rotation not allowed yet: {0}")]
    RotationNotAllowed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    pub key_length: usize,
    pub default_expiry_days: i64,
    pub max_keys_per_user: u8,
    pub rotation_interval_days: i64,
    pub grace_period_days: i64,
    pub encryption_enabled: bool,
    pub audit_key_usage: bool,
    pub require_ip_whitelist: bool,
    pub rate_limit_per_key: u32,
}

impl Default for ApiKeyConfig {
    fn default() -> Self {
        Self {
            key_length: 64,
            default_expiry_days: 90,
            max_keys_per_user: 5,
            rotation_interval_days: 30,
            grace_period_days: 7,
            encryption_enabled: true,
            audit_key_usage: true,
            require_ip_whitelist: false,
            rate_limit_per_key: 1000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub key_hash: String,
    pub key_prefix: String,
    pub name: String,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub usage_count: u64,
    pub is_active: bool,
    pub permissions: Vec<String>,
    pub ip_whitelist: Vec<String>,
    pub rotation_scheduled_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyUsage {
    pub key_id: Uuid,
    pub user_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub ip_address: String,
    pub user_agent: String,
    pub endpoint: String,
    pub success: bool,
    pub response_time_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ApiKeyManager {
    config: ApiKeyConfig,
    encryption_key: Key<Aes256Gcm>,
    keys: HashMap<String, ApiKey>,
    usage_logs: Vec<ApiKeyUsage>,
}

impl ApiKeyManager {
    pub fn new(config: ApiKeyConfig) -> Result<Self, ApiKeyError> {
        let encryption_key = Aes256Gcm::generate_key(&mut OsRng);
        
        Ok(Self {
            config,
            encryption_key,
            keys: HashMap::new(),
            usage_logs: Vec::new(),
        })
    }

    pub fn generate_api_key(&mut self, user_id: Uuid, name: &str, permissions: Vec<String>) -> Result<(String, ApiKey), ApiKeyError> {
        // Check user key limit
        let user_keys = self.keys.values()
            .filter(|key| key.user_id == user_id && key.is_active)
            .count();
        
        if user_keys >= self.config.max_keys_per_user as usize {
            return Err(ApiKeyError::KeyGenerationFailed(
                "Maximum API keys per user exceeded".to_string()
            ));
        }

        // Generate random key
        let key_value: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(self.config.key_length)
            .map(char::from)
            .collect();

        let key_prefix = format!("mrpk_{}", &key_value[..8]);
        
        // Hash the key for storage
        let key_hash = self.hash_api_key(&key_value)?;
        
        let now = Utc::now();
        let expires_at = now + Duration::days(self.config.default_expiry_days);
        
        let api_key = ApiKey {
            id: Uuid::new_v4(),
            key_hash,
            key_prefix: key_prefix.clone(),
            name: name.to_string(),
            user_id,
            created_at: now,
            expires_at,
            last_used_at: None,
            usage_count: 0,
            is_active: true,
            permissions,
            ip_whitelist: vec![],
            rotation_scheduled_at: None,
            metadata: serde_json::json!({}),
        };

        self.keys.insert(key_value.clone(), api_key.clone());
        
        Ok((key_value, api_key))
    }

    pub fn validate_api_key(&mut self, key_value: &str, ip_address: &str) -> Result<ApiKey, ApiKeyError> {
        let api_key = self.keys.get(key_value)
            .ok_or_else(|| ApiKeyError::KeyNotFound(key_value.to_string()))?
            .clone();

        // Check if key is active
        if !api_key.is_active {
            return Err(ApiKeyError::KeyRevoked(api_key.id.to_string()));
        }

        // Check expiration
        if Utc::now() > api_key.expires_at {
            return Err(ApiKeyError::KeyExpired(api_key.id.to_string()));
        }

        // Check IP whitelist if required
        if self.config.require_ip_whitelist && !api_key.ip_whitelist.is_empty() {
            if !api_key.ip_whitelist.contains(&ip_address.to_string()) {
                return Err(ApiKeyError::KeyNotFound(
                    "IP address not in whitelist".to_string()
                ));
            }
        }

        // Update usage statistics
        self.update_key_usage(&api_key.id, ip_address);

        Ok(api_key)
    }

    pub fn revoke_api_key(&mut self, key_id: &Uuid) -> Result<(), ApiKeyError> {
        let key_value = self.keys.iter()
            .find(|(_, key)| key.id == *key_id)
            .map(|(key_value, _)| key_value.clone())
            .ok_or_else(|| ApiKeyError::KeyNotFound(key_id.to_string()))?;

        if let Some(api_key) = self.keys.get_mut(&key_value) {
            api_key.is_active = false;
            Ok(())
        } else {
            Err(ApiKeyError::KeyNotFound(key_id.to_string()))
        }
    }

    pub fn rotate_api_key(&mut self, key_id: &Uuid) -> Result<(String, ApiKey), ApiKeyError> {
        let key_value = self.keys.iter()
            .find(|(_, key)| key.id == *key_id)
            .map(|(key_value, key)| (key_value.clone(), key.clone()))
            .ok_or_else(|| ApiKeyError::KeyNotFound(key_id.to_string()))?;

        // Check if rotation is allowed
        if let Some(rotation_scheduled) = key_value.1.rotation_scheduled_at {
            if Utc::now() < rotation_scheduled {
                return Err(ApiKeyError::RotationNotAllowed(
                    format!("Rotation scheduled for {}", rotation_scheduled)
                ));
            }
        }

        // Generate new key
        let new_key_value: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(self.config.key_length)
            .map(char::from)
            .collect();

        let new_key_hash = self.hash_api_key(&new_key_value)?;
        
        // Update existing key
        if let Some(api_key) = self.keys.get_mut(&key_value.0) {
            api_key.key_hash = new_key_hash.clone();
            api_key.last_used_at = None;
            api_key.usage_count = 0;
            api_key.rotation_scheduled_at = None;
            
            // Schedule next rotation
            api_key.rotation_scheduled_at = Some(
                Utc::now() + Duration::days(self.config.rotation_interval_days)
            );
        }

        // Add new key to storage
        let mut new_api_key = key_value.1.clone();
        new_api_key.key_hash = new_key_hash;
        new_api_key.key_prefix = format!("mrpk_{}", &new_key_value[..8]);
        self.keys.insert(new_key_value.clone(), new_api_key.clone());

        // Remove old key
        self.keys.remove(&key_value.0);

        Ok((new_key_value, new_api_key))
    }

    pub fn schedule_rotation(&mut self, key_id: &Uuid, scheduled_at: DateTime<Utc>) -> Result<(), ApiKeyError> {
        let key_value = self.keys.iter_mut()
            .find(|(_, key)| key.id == *key_id)
            .map(|(_, key)| key)
            .ok_or_else(|| ApiKeyError::KeyNotFound(key_id.to_string()))?;

        key_value.rotation_scheduled_at = Some(scheduled_at);
        Ok(())
    }

    pub fn get_user_keys(&self, user_id: &Uuid) -> Vec<&ApiKey> {
        self.keys.values()
            .filter(|key| key.user_id == *user_id)
            .collect()
    }

    pub fn get_expiring_keys(&self, days_ahead: i64) -> Vec<&ApiKey> {
        let cutoff = Utc::now() + Duration::days(days_ahead);
        self.keys.values()
            .filter(|key| key.is_active && key.expires_at <= cutoff)
            .collect()
    }

    pub fn get_keys_requiring_rotation(&self) -> Vec<&ApiKey> {
        let now = Utc::now();
        self.keys.values()
            .filter(|key| {
                key.is_active && 
                key.rotation_scheduled_at.map_or(false, |scheduled| scheduled <= now)
            })
            .collect()
    }

    pub fn get_key_usage_stats(&self, key_id: &Uuid) -> Option<KeyUsageStats> {
        let api_key = self.keys.values().find(|key| key.id == *key_id)?;
        
        let recent_usage = self.usage_logs.iter()
            .filter(|usage| usage.key_id == *key_id)
            .count();

        Some(KeyUsageStats {
            key_id: *key_id,
            total_usage: api_key.usage_count,
            recent_usage: recent_usage as u64,
            last_used: api_key.last_used_at,
            success_rate: self.calculate_success_rate(key_id),
        })
    }

    fn hash_api_key(&self, key_value: &str) -> Result<String, ApiKeyError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        argon2.hash_password(key_value.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| ApiKeyError::KeyGenerationFailed(e.to_string()))
    }

    fn update_key_usage(&mut self, key_id: &Uuid, ip_address: &str) {
        if let Some(api_key) = self.keys.values_mut().find(|key| key.id == *key_id) {
            api_key.last_used_at = Some(Utc::now());
            api_key.usage_count += 1;
        }

        // Log usage if enabled
        if self.config.audit_key_usage {
            let usage = ApiKeyUsage {
                key_id: *key_id,
                user_id: Uuid::new_v4(), // This should be extracted from the API key
                timestamp: Utc::now(),
                ip_address: ip_address.to_string(),
                user_agent: "Unknown".to_string(), // This should be extracted from request
                endpoint: "Unknown".to_string(), // This should be extracted from request
                success: true,
                response_time_ms: 0,
            };
            self.usage_logs.push(usage);
        }
    }

    fn calculate_success_rate(&self, key_id: &Uuid) -> f64 {
        let key_usage = self.usage_logs.iter()
            .filter(|usage| usage.key_id == *key_id)
            .collect::<Vec<_>>();
        
        if key_usage.is_empty() {
            return 100.0;
        }

        let successful = key_usage.iter().filter(|usage| usage.success).count();
        (successful as f64 / key_usage.len() as f64) * 100.0
    }

    pub fn cleanup_expired_keys(&mut self) -> Vec<Uuid> {
        let now = Utc::now();
        let mut expired_keys = Vec::new();

        self.keys.retain(|_, key| {
            if key.expires_at < now {
                expired_keys.push(key.id);
                false
            } else {
                true
            }
        });

        expired_keys
    }

    pub fn encrypt_sensitive_data(&self, data: &str) -> Result<String, ApiKeyError> {
        if !self.config.encryption_enabled {
            return Ok(data.to_string());
        }

        let cipher = Aes256Gcm::new(&self.encryption_key);
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        cipher.encrypt(&nonce, data.as_bytes())
            .map(|encrypted| {
                let mut result = nonce.to_vec();
                result.extend_from_slice(&encrypted);
                base64::encode(&result)
            })
            .map_err(|e| ApiKeyError::EncryptionFailed(e.to_string()))
    }

    pub fn decrypt_sensitive_data(&self, encrypted_data: &str) -> Result<String, ApiKeyError> {
        if !self.config.encryption_enabled {
            return Ok(encrypted_data.to_string());
        }

        let decoded = base64::decode(encrypted_data)
            .map_err(|e| ApiKeyError::DecryptionFailed(e.to_string()))?;
        
        if decoded.len() < 12 {
            return Err(ApiKeyError::DecryptionFailed(
                "Invalid encrypted data format".to_string()
            ));
        }

        let (nonce_bytes, ciphertext) = decoded.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let cipher = Aes256Gcm::new(&self.encryption_key);
        cipher.decrypt(nonce, ciphertext)
            .map(|decrypted| String::from_utf8_lossy(&decrypted).to_string())
            .map_err(|e| ApiKeyError::DecryptionFailed(e.to_string()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyUsageStats {
    pub key_id: Uuid,
    pub total_usage: u64,
    pub recent_usage: u64,
    pub last_used: Option<DateTime<Utc>>,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_generation() {
        let config = ApiKeyConfig::default();
        let mut manager = ApiKeyManager::new(config).unwrap();
        
        let user_id = Uuid::new_v4();
        let (key_value, api_key) = manager.generate_api_key(
            user_id, 
            "Test Key", 
            vec!["read".to_string(), "write".to_string()]
        ).unwrap();
        
        assert_eq!(key_value.len(), 64);
        assert!(key_value.starts_with("mrpk_"));
        assert_eq!(api_key.user_id, user_id);
        assert_eq!(api_key.name, "Test Key");
        assert!(api_key.is_active);
    }

    #[test]
    fn test_api_key_validation() {
        let config = ApiKeyConfig::default();
        let mut manager = ApiKeyManager::new(config).unwrap();
        
        let user_id = Uuid::new_v4();
        let (key_value, _) = manager.generate_api_key(
            user_id, 
            "Test Key", 
            vec!["read".to_string()]
        ).unwrap();
        
        let validated_key = manager.validate_api_key(&key_value, "127.0.0.1").unwrap();
        assert_eq!(validated_key.user_id, user_id);
        assert!(validated_key.is_active);
    }

    #[test]
    fn test_api_key_revocation() {
        let config = ApiKeyConfig::default();
        let mut manager = ApiKeyManager::new(config).unwrap();
        
        let user_id = Uuid::new_v4();
        let (key_value, api_key) = manager.generate_api_key(
            user_id, 
            "Test Key", 
            vec!["read".to_string()]
        ).unwrap();
        
        manager.revoke_api_key(&api_key.id).unwrap();
        
        let result = manager.validate_api_key(&key_value, "127.0.0.1");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiKeyError::KeyRevoked(_)));
    }

    #[test]
    fn test_key_rotation() {
        let config = ApiKeyConfig::default();
        let mut manager = ApiKeyManager::new(config).unwrap();
        
        let user_id = Uuid::new_v4();
        let (key_value, api_key) = manager.generate_api_key(
            user_id, 
            "Test Key", 
            vec!["read".to_string()]
        ).unwrap();
        
        // Schedule immediate rotation
        manager.schedule_rotation(&api_key.id, Utc::now()).unwrap();
        
        let (new_key_value, new_api_key) = manager.rotate_api_key(&api_key.id).unwrap();
        
        assert_ne!(key_value, new_key_value);
        assert_eq!(new_api_key.user_id, user_id);
        assert!(new_api_key.is_active);
        
        // Old key should no longer be valid
        let result = manager.validate_api_key(&key_value, "127.0.0.1");
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_decryption() {
        let config = ApiKeyConfig::default();
        let manager = ApiKeyManager::new(config).unwrap();
        
        let sensitive_data = "super_secret_api_key_123";
        
        let encrypted = manager.encrypt_sensitive_data(sensitive_data).unwrap();
        assert_ne!(encrypted, sensitive_data);
        
        let decrypted = manager.decrypt_sensitive_data(&encrypted).unwrap();
        assert_eq!(decrypted, sensitive_data);
    }
}
