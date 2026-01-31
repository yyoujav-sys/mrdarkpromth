use serde::{Deserialize, Serialize};
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit};
use aes_gcm::aead::{Aead, OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng as ArgonRng};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Key management error: {0}")]
    KeyManagementError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    #[error("Key not found: {0}")]
    KeyNotFound(Uuid),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionMetadata {
    pub key_id: Uuid,
    pub algorithm: String,
    pub nonce: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub data: Vec<u8>,
    pub metadata: EncryptionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub id: Uuid,
    pub name: String,
    pub key_data: Vec<u8>,
    pub key_type: KeyType,
    pub created_at: DateTime<Utc>,
    pub last_rotated: Option<DateTime<Utc>>,
    pub rotation_interval_days: Option<u32>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyType {
    DataEncryption,
    AuditEncryption,
    ConfigEncryption,
    MasterKey,
}

#[derive(Debug, Clone)]
pub struct EncryptionManager {
    keys: HashMap<Uuid, EncryptionKey>,
    master_key: Option<Vec<u8>>,
    key_storage_path: String,
    auto_rotation_enabled: bool,
    default_key_id: Option<Uuid>,
}

impl EncryptionManager {
    pub fn new(key_storage_path: String) -> Self {
        Self {
            keys: HashMap::new(),
            master_key: None,
            key_storage_path,
            auto_rotation_enabled: true,
            default_key_id: None,
        }
    }

    pub fn initialize_with_master_key(&mut self, master_password: &str) -> Result<(), EncryptionError> {
        // Generate or load master key
        let master_key = self.derive_master_key(master_password)?;
        self.master_key = Some(master_key.clone());

        // Load existing keys or create new ones
        if Path::new(&self.key_storage_path).exists() {
            self.load_keys()?;
        } else {
            self.create_default_keys(&master_key)?;
            self.save_keys()?;
        }

        Ok(())
    }

    pub fn encrypt_data(&mut self, data: &[u8], key_id: Option<Uuid>) -> Result<EncryptedData, EncryptionError> {
        let key_id = key_id.or(self.default_key_id)
            .ok_or_else(|| EncryptionError::KeyManagementError("No encryption key specified".to_string()))?;

        let key = self.keys.get(&key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound(key_id))?;

        if !key.is_active {
            return Err(EncryptionError::KeyManagementError(
                format!("Key {} is not active", key_id)
            ));
        }

        let cipher_key = Key::<Aes256Gcm>::from_slice(&key.key_data);
        let cipher = Aes256Gcm::new(cipher_key);
        
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        
        let ciphertext = cipher.encrypt(&nonce, data)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        let metadata = EncryptionMetadata {
            key_id,
            algorithm: "AES-256-GCM".to_string(),
            nonce: nonce.to_vec(),
            created_at: Utc::now(),
            version: 1,
        };

        Ok(EncryptedData {
            data: ciphertext,
            metadata,
        })
    }

    pub fn decrypt_data(&self, encrypted_data: &EncryptedData) -> Result<Vec<u8>, EncryptionError> {
        let key = self.keys.get(&encrypted_data.metadata.key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound(encrypted_data.metadata.key_id))?;

        let cipher_key = Key::<Aes256Gcm>::from_slice(&key.key_data);
        let cipher = Aes256Gcm::new(cipher_key);
        
        let nonce = Nonce::from_slice(&encrypted_data.metadata.nonce);
        
        let plaintext = cipher.decrypt(nonce, encrypted_data.data.as_ref())
            .map_err(|e| EncryptionError::DecryptionFailed(e.to_string()))?;

        Ok(plaintext)
    }

    pub fn encrypt_string(&mut self, text: &str, key_id: Option<Uuid>) -> Result<EncryptedData, EncryptionError> {
        self.encrypt_data(text.as_bytes(), key_id)
    }

    pub fn decrypt_string(&self, encrypted_data: &EncryptedData) -> Result<String, EncryptionError> {
        let plaintext = self.decrypt_data(encrypted_data)?;
        String::from_utf8(plaintext)
            .map_err(|e| EncryptionError::DecryptionFailed(format!("Invalid UTF-8: {}", e)))
    }

    pub fn create_key(&mut self, 
                     name: String,
                     key_type: KeyType,
                     rotation_interval_days: Option<u32>) -> Result<Uuid, EncryptionError> {
        let key_id = Uuid::new_v4();
        let key_data = self.generate_encryption_key()?;

        let key = EncryptionKey {
            id: key_id,
            name,
            key_data,
            key_type,
            created_at: Utc::now(),
            last_rotated: None,
            rotation_interval_days,
            is_active: true,
            metadata: serde_json::json!({}),
        };

        self.keys.insert(key_id, key);

        if self.default_key_id.is_none() && matches!(key_type, KeyType::DataEncryption) {
            self.default_key_id = Some(key_id);
        }

        self.save_keys()?;

        Ok(key_id)
    }

    pub fn rotate_key(&mut self, key_id: Uuid) -> Result<(), EncryptionError> {
        let key = self.keys.get_mut(&key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound(key_id))?;

        // Generate new key data
        let new_key_data = self.generate_encryption_key()?;
        
        // Store old key data for re-encryption if needed
        let old_key_data = key.key_data.clone();
        key.key_data = new_key_data;
        key.last_rotated = Some(Utc::now());

        // Save updated keys
        self.save_keys()?;

        log::info!("Key {} rotated successfully", key_id);
        Ok(())
    }

    pub fn deactivate_key(&mut self, key_id: Uuid) -> Result<(), EncryptionError> {
        let key = self.keys.get_mut(&key_id)
            .ok_or_else(|| EncryptionError::KeyNotFound(key_id))?;

        key.is_active = false;

        // Update default key if this was the default
        if self.default_key_id == Some(key_id) {
            self.default_key_id = self.find_active_key(KeyType::DataEncryption);
        }

        self.save_keys()?;
        Ok(())
    }

    pub fn get_key(&self, key_id: Uuid) -> Option<&EncryptionKey> {
        self.keys.get(&key_id)
    }

    pub fn list_keys(&self) -> Vec<&EncryptionKey> {
        self.keys.values().collect()
    }

    pub fn get_active_keys(&self, key_type: KeyType) -> Vec<&EncryptionKey> {
        self.keys.values()
            .filter(|key| key.key_type == key_type && key.is_active)
            .collect()
    }

    pub fn check_key_rotation(&mut self) -> Result<Vec<Uuid>, EncryptionError> {
        let mut rotated_keys = Vec::new();

        if !self.auto_rotation_enabled {
            return Ok(rotated_keys);
        }

        let now = Utc::now();
        let keys_to_rotate: Vec<_> = self.keys.values()
            .filter(|key| {
                key.is_active && 
                key.rotation_interval_days.is_some() &&
                key.last_rotated.is_some()
            })
            .filter(|key| {
                let rotation_interval = Duration::days(key.rotation_interval_days.unwrap() as i64);
                now - key.last_rotated.unwrap() > rotation_interval
            })
            .map(|key| key.id)
            .collect();

        for key_id in keys_to_rotate {
            self.rotate_key(key_id)?;
            rotated_keys.push(key_id);
        }

        Ok(rotated_keys)
    }

    pub fn export_encrypted_keys(&self, export_key_id: Uuid) -> Result<Vec<u8>, EncryptionError> {
        let export_data = KeyExport {
            keys: self.keys.clone(),
            export_timestamp: Utc::now(),
            version: 1,
        };

        let serialized = serde_json::to_vec(&export_data)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        // Encrypt the export with specified key
        self.encrypt_data(&serialized, Some(export_key_id))
            .map(|encrypted| encrypted.data)
    }

    pub fn import_encrypted_keys(&mut self, 
                                encrypted_data: Vec<u8>, 
                                import_key_id: Uuid) -> Result<(), EncryptionError> {
        // Decrypt the export data
        let encrypted_export = EncryptedData {
            data: encrypted_data,
            metadata: EncryptionMetadata {
                key_id: import_key_id,
                algorithm: "AES-256-GCM".to_string(),
                nonce: vec![0; 12], // This would need to be provided separately
                created_at: Utc::now(),
                version: 1,
            },
        };

        let decrypted_data = self.decrypt_data(&encrypted_export)?;
        let import_data: KeyExport = serde_json::from_slice(&decrypted_data)
            .map_err(|e| EncryptionError::InvalidKeyFormat(e.to_string()))?;

        // Validate and import keys
        for (key_id, key) in import_data.keys {
            if !self.keys.contains_key(&key_id) {
                self.keys.insert(key_id, key);
            }
        }

        self.save_keys()?;
        Ok(())
    }

    // Private helper methods
    fn derive_master_key(&self, password: &str) -> Result<Vec<u8>, EncryptionError> {
        let salt = SaltString::generate(&mut ArgonRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| EncryptionError::KeyManagementError(e.to_string()))?;

        // Extract the hash bytes as master key
        let hash = password_hash.hash.unwrap();
        Ok(hash.as_bytes().to_vec())
    }

    fn generate_encryption_key(&self) -> Result<Vec<u8>, EncryptionError> {
        let key = Aes256Gcm::generate_key(&mut OsRng);
        Ok(key.to_vec())
    }

    fn create_default_keys(&mut self, master_key: &[u8]) -> Result<(), EncryptionError> {
        // Create default encryption keys
        let data_key_id = self.create_key(
            "default_data_encryption".to_string(),
            KeyType::DataEncryption,
            Some(90), // Rotate every 90 days
        )?;

        let audit_key_id = self.create_key(
            "audit_log_encryption".to_string(),
            KeyType::AuditEncryption,
            Some(180), // Rotate every 180 days
        )?;

        let config_key_id = self.create_key(
            "config_encryption".to_string(),
            KeyType::ConfigEncryption,
            Some(365), // Rotate every year
        )?;

        self.default_key_id = Some(data_key_id);

        log::info!("Created default encryption keys: data={}, audit={}, config={}", 
                  data_key_id, audit_key_id, config_key_id);

        Ok(())
    }

    fn save_keys(&self) -> Result<(), EncryptionError> {
        let key_data = serde_json::to_vec(&self.keys)
            .map_err(|e| EncryptionError::EncryptionFailed(e.to_string()))?;

        // Ensure directory exists
        if let Some(parent) = Path::new(&self.key_storage_path).parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.key_storage_path, key_data)?;
        Ok(())
    }

    fn load_keys(&mut self) -> Result<(), EncryptionError> {
        let key_data = fs::read(&self.key_storage_path)?;
        
        let loaded_keys: HashMap<Uuid, EncryptionKey> = serde_json::from_slice(&key_data)
            .map_err(|e| EncryptionError::InvalidKeyFormat(e.to_string()))?;

        self.keys = loaded_keys;

        // Set default key if not already set
        if self.default_key_id.is_none() {
            self.default_key_id = self.find_active_key(KeyType::DataEncryption);
        }

        Ok(())
    }

    fn find_active_key(&self, key_type: KeyType) -> Option<Uuid> {
        self.keys.values()
            .find(|key| key.key_type == key_type && key.is_active)
            .map(|key| key.id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyExport {
    keys: HashMap<Uuid, EncryptionKey>,
    export_timestamp: DateTime<Utc>,
    version: u32,
}

use chrono::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionStats {
    pub total_keys: usize,
    pub active_keys: usize,
    pub keys_by_type: HashMap<KeyType, u64>,
    pub oldest_key_age_days: Option<u64>,
    pub keys_needing_rotation: Vec<Uuid>,
    pub auto_rotation_enabled: bool,
}

impl EncryptionManager {
    pub fn get_stats(&self) -> EncryptionStats {
        let total_keys = self.keys.len();
        let active_keys = self.keys.values().filter(|k| k.is_active).count();

        let keys_by_type = self.keys.values()
            .fold(HashMap::new(), |mut acc, key| {
                *acc.entry(key.key_type.clone()).or_insert(0) += 1;
                acc
            });

        let oldest_key_age_days = self.keys.values()
            .filter(|k| k.is_active)
            .map(|k| (Utc::now() - k.created_at).num_days())
            .min()
            .map(|days| days as u64);

        let keys_needing_rotation = self.keys.values()
            .filter(|key| {
                key.is_active && 
                key.rotation_interval_days.is_some() &&
                key.last_rotated.is_some()
            })
            .filter(|key| {
                let rotation_interval = Duration::days(key.rotation_interval_days.unwrap() as i64);
                Utc::now() - key.last_rotated.unwrap() > rotation_interval
            })
            .map(|key| key.id)
            .collect();

        EncryptionStats {
            total_keys,
            active_keys,
            keys_by_type,
            oldest_key_age_days,
            keys_needing_rotation,
            auto_rotation_enabled: self.auto_rotation_enabled,
        }
    }

    pub fn enable_auto_rotation(&mut self, enabled: bool) {
        self.auto_rotation_enabled = enabled;
    }

    pub fn set_default_key(&mut self, key_id: Uuid) -> Result<(), EncryptionError> {
        if !self.keys.contains_key(&key_id) {
            return Err(EncryptionError::KeyNotFound(key_id));
        }

        self.default_key_id = Some(key_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let mut manager = EncryptionManager::new("test_keys.json".to_string());
        
        // Initialize with test master key
        manager.initialize_with_master_key("test_password_123").unwrap();

        let original_data = b"Hello, World!";
        
        // Encrypt data
        let encrypted = manager.encrypt_data(original_data, None).unwrap();
        
        // Decrypt data
        let decrypted = manager.decrypt_data(&encrypted).unwrap();
        
        assert_eq!(original_data, &decrypted[..]);
    }

    #[test]
    fn test_string_encryption() {
        let mut manager = EncryptionManager::new("test_keys.json".to_string());
        manager.initialize_with_master_key("test_password_123").unwrap();

        let original_text = "Sensitive information";
        
        let encrypted = manager.encrypt_string(original_text, None).unwrap();
        let decrypted = manager.decrypt_string(&encrypted).unwrap();
        
        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_key_creation() {
        let mut manager = EncryptionManager::new("test_keys.json".to_string());
        manager.initialize_with_master_key("test_password_123").unwrap();

        let key_id = manager.create_key(
            "test_key".to_string(),
            KeyType::DataEncryption,
            Some(30)
        ).unwrap();

        let key = manager.get_key(key_id).unwrap();
        assert_eq!(key.name, "test_key");
        assert!(key.is_active);
    }

    #[test]
    fn test_key_rotation() {
        let mut manager = EncryptionManager::new("test_keys.json".to_string());
        manager.initialize_with_master_key("test_password_123").unwrap();

        let key_id = manager.create_key(
            "test_key".to_string(),
            KeyType::DataEncryption,
            Some(30)
        ).unwrap();

        let original_key = manager.get_key(key_id).unwrap();
        let original_key_data = original_key.key_data.clone();

        manager.rotate_key(key_id).unwrap();

        let rotated_key = manager.get_key(key_id).unwrap();
        assert_ne!(original_key_data, rotated_key.key_data);
        assert!(rotated_key.last_rotated.is_some());
    }

    #[test]
    fn test_encryption_stats() {
        let mut manager = EncryptionManager::new("test_keys.json".to_string());
        manager.initialize_with_master_key("test_password_123").unwrap();

        let stats = manager.get_stats();
        assert!(stats.total_keys >= 3); // Default keys
        assert!(stats.active_keys >= 3);
        assert!(stats.auto_rotation_enabled);
    }
}
