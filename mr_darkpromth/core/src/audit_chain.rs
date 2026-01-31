use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use std::collections::HashMap;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;

#[derive(Error, Debug)]
pub enum AuditError {
    #[error("Tampering detected: {0}")]
    TamperingDetected(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Audit log not found: {0}")]
    LogNotFound(Uuid),
    #[error("Storage error: {0}")]
    StorageError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSignature {
    pub log_id: Uuid,
    pub signature: String,
    pub timestamp: DateTime<Utc>,
    pub previous_hash: Option<String>,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TamperEvidence {
    pub detected_at: DateTime<Utc>,
    pub log_id: Uuid,
    pub expected_hash: String,
    pub actual_hash: String,
    pub evidence_type: TamperType,
    pub severity: TamperSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TamperType {
    HashMismatch,
    SignatureInvalid,
    SequenceBroken,
    TimestampAnomaly,
    ContentModified,
    ChainBroken,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TamperSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct AuditChain {
    logs: HashMap<Uuid, SignedAuditLog>,
    signatures: HashMap<Uuid, AuditSignature>,
    chain_head: Option<Uuid>,
    sequence_counter: u64,
    secret_key: Vec<u8>,
    tamper_evidence: Vec<TamperEvidence>,
    verification_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAuditLog {
    pub log: crate::audit::AuditLog,
    pub hash: String,
    pub signature: String,
    pub sequence_number: u64,
    pub previous_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuditChain {
    pub fn new(secret_key: Vec<u8>) -> Self {
        Self {
            logs: HashMap::new(),
            signatures: HashMap::new(),
            chain_head: None,
            sequence_counter: 0,
            secret_key,
            tamper_evidence: Vec::new(),
            verification_enabled: true,
        }
    }

    pub fn add_log(&mut self, mut audit_log: crate::audit::AuditLog) -> Result<(), AuditError> {
        if self.verification_enabled {
            // Verify chain integrity before adding new log
            self.verify_chain_integrity()?;
        }

        self.sequence_counter += 1;
        
        let previous_hash = self.chain_head
            .and_then(|head_id| self.logs.get(&head_id))
            .map(|log| log.hash.clone());

        // Create hash of the log content
        let log_content = self.serialize_log_content(&audit_log);
        let hash = self.calculate_hash(&log_content);
        
        // Create signature
        let signature_input = self.create_signature_input(&hash, previous_hash.as_deref(), self.sequence_counter);
        let signature = self.create_signature(&signature_input)?;

        let signed_log = SignedAuditLog {
            log: audit_log.clone(),
            hash: hash.clone(),
            signature: signature.clone(),
            sequence_number: self.sequence_counter,
            previous_hash: previous_hash.clone(),
            created_at: Utc::now(),
        };

        let audit_signature = AuditSignature {
            log_id: audit_log.id,
            signature: signature.clone(),
            timestamp: Utc::now(),
            previous_hash: previous_hash.clone(),
            sequence_number: self.sequence_counter,
        };

        self.logs.insert(audit_log.id, signed_log);
        self.signatures.insert(audit_log.id, audit_signature);
        self.chain_head = Some(audit_log.id);

        Ok(())
    }

    pub fn verify_log(&self, log_id: &Uuid) -> Result<bool, AuditError> {
        let signed_log = self.logs.get(log_id)
            .ok_or_else(|| AuditError::LogNotFound(*log_id))?;

        // Verify hash
        let log_content = self.serialize_log_content(&signed_log.log);
        let expected_hash = self.calculate_hash(&log_content);
        
        if expected_hash != signed_log.hash {
            return Err(AuditError::TamperingDetected(
                format!("Hash mismatch for log {}", log_id)
            ));
        }

        // Verify signature
        let signature_input = self.create_signature_input(
            &signed_log.hash,
            signed_log.previous_hash.as_deref(),
            signed_log.sequence_number
        );
        
        if !self.verify_signature(&signature_input, &signed_log.signature)? {
            return Err(AuditError::InvalidSignature(
                format!("Invalid signature for log {}", log_id)
            ));
        }

        // Verify chain link
        if let Some(prev_hash) = &signed_log.previous_hash {
            if let Some(prev_id) = self.find_log_by_hash(prev_hash) {
                let prev_log = self.logs.get(&prev_id).unwrap();
                if prev_log.sequence_number != signed_log.sequence_number - 1 {
                    return Err(AuditError::TamperingDetected(
                        format!("Sequence broken for log {}", log_id)
                    ));
                }
            } else {
                return Err(AuditError::TamperingDetected(
                    format!("Previous log not found for log {}", log_id)
                ));
            }
        }

        Ok(true)
    }

    pub fn verify_chain_integrity(&self) -> Result<(), AuditError> {
        let mut current_id = self.chain_head;
        let mut expected_sequence = self.sequence_counter;

        while let Some(log_id) = current_id {
            let signed_log = self.logs.get(&log_id)
                .ok_or_else(|| AuditError::LogNotFound(log_id))?;

            // Verify sequence
            if signed_log.sequence_number != expected_sequence {
                self.record_tamper_evidence(
                    log_id,
                    signed_log.hash.clone(),
                    format!("Expected sequence {}, got {}", expected_sequence, signed_log.sequence_number),
                    TamperType::SequenceBroken,
                    TamperSeverity::High,
                );
                return Err(AuditError::TamperingDetected(
                    format!("Sequence mismatch at log {}", log_id)
                ));
            }

            // Verify hash
            let log_content = self.serialize_log_content(&signed_log.log);
            let expected_hash = self.calculate_hash(&log_content);
            
            if expected_hash != signed_log.hash {
                self.record_tamper_evidence(
                    log_id,
                    expected_hash,
                    signed_log.hash.clone(),
                    TamperType::HashMismatch,
                    TamperSeverity::Critical,
                );
                return Err(AuditError::TamperingDetected(
                    format!("Hash tampering detected at log {}", log_id)
                ));
            }

            // Verify signature
            let signature_input = self.create_signature_input(
                &signed_log.hash,
                signed_log.previous_hash.as_deref(),
                signed_log.sequence_number
            );
            
            if !self.verify_signature(&signature_input, &signed_log.signature)? {
                self.record_tamper_evidence(
                    log_id,
                    "N/A".to_string(),
                    signed_log.signature.clone(),
                    TamperType::SignatureInvalid,
                    TamperSeverity::Critical,
                );
                return Err(AuditError::InvalidSignature(
                    format!("Signature invalid for log {}", log_id)
                ));
            }

            expected_sequence -= 1;
            current_id = self.find_previous_log(&signed_log);
        }

        Ok(())
    }

    pub fn get_tamper_evidence(&self) -> &[TamperEvidence] {
        &self.tamper_evidence
    }

    pub fn clear_tamper_evidence(&mut self) {
        self.tamper_evidence.clear();
    }

    pub fn get_chain_stats(&self) -> ChainStats {
        ChainStats {
            total_logs: self.logs.len(),
            chain_head: self.chain_head,
            sequence_counter: self.sequence_counter,
            tamper_evidence_count: self.tamper_evidence.len(),
            verification_enabled: self.verification_enabled,
            oldest_log_timestamp: self.get_oldest_log_timestamp(),
            newest_log_timestamp: self.get_newest_log_timestamp(),
        }
    }

    pub fn export_chain(&self) -> Result<Vec<u8>, AuditError> {
        let export_data = ChainExport {
            logs: self.logs.clone(),
            signatures: self.signatures.clone(),
            chain_head: self.chain_head,
            sequence_counter: self.sequence_counter,
            tamper_evidence: self.tamper_evidence.clone(),
            export_timestamp: Utc::now(),
        };

        serde_json::to_vec(&export_data)
            .map_err(|e| AuditError::StorageError(e.to_string()))
    }

    pub fn import_chain(&mut self, data: &[u8]) -> Result<(), AuditError> {
        let import_data: ChainExport = serde_json::from_slice(data)
            .map_err(|e| AuditError::StorageError(e.to_string()))?;

        // Verify imported chain before accepting
        let temp_chain = AuditChain {
            logs: import_data.logs,
            signatures: import_data.signatures,
            chain_head: import_data.chain_head,
            sequence_counter: import_data.sequence_counter,
            secret_key: self.secret_key.clone(),
            tamper_evidence: import_data.tamper_evidence,
            verification_enabled: self.verification_enabled,
        };

        temp_chain.verify_chain_integrity()?;

        // If verification passes, replace current chain
        self.logs = temp_chain.logs;
        self.signatures = temp_chain.signatures;
        self.chain_head = temp_chain.chain_head;
        self.sequence_counter = temp_chain.sequence_counter;
        self.tamper_evidence = temp_chain.tamper_evidence;

        Ok(())
    }

    pub fn enable_verification(&mut self, enabled: bool) {
        self.verification_enabled = enabled;
    }

    pub fn rotate_secret_key(&mut self, new_secret_key: Vec<u8>) -> Result<(), AuditError> {
        // Verify current chain before rotation
        self.verify_chain_integrity()?;

        self.secret_key = new_secret_key;

        // Re-sign all logs with new key
        for (log_id, signed_log) in self.logs.iter_mut() {
            let signature_input = self.create_signature_input(
                &signed_log.hash,
                signed_log.previous_hash.as_deref(),
                signed_log.sequence_number
            );
            signed_log.signature = self.create_signature(&signature_input)?;

            // Update signature record
            if let Some(audit_signature) = self.signatures.get_mut(log_id) {
                audit_signature.signature = signed_log.signature.clone();
                audit_signature.timestamp = Utc::now();
            }
        }

        Ok(())
    }

    // Private helper methods
    fn serialize_log_content(&self, audit_log: &crate::audit::AuditLog) -> String {
        serde_json::to_string(audit_log).unwrap_or_default()
    }

    fn calculate_hash(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn create_signature_input(&self, hash: &str, previous_hash: Option<&str>, sequence: u64) -> String {
        format!("{}|{}|{}", hash, previous_hash.unwrap_or(""), sequence)
    }

    fn create_signature(&self, input: &str) -> Result<String, AuditError> {
        let mut mac = HmacSha256::new_from_slice(&self.secret_key)
            .map_err(|e| AuditError::StorageError(e.to_string()))?;
        mac.update(input.as_bytes());
        Ok(format!("{:x}", mac.finalize().into_bytes()))
    }

    fn verify_signature(&self, input: &str, signature: &str) -> Result<bool, AuditError> {
        let expected_signature = self.create_signature(input)?;
        Ok(expected_signature == signature)
    }

    fn find_log_by_hash(&self, hash: &str) -> Option<Uuid> {
        self.logs.iter()
            .find(|(_, log)| log.hash == hash)
            .map(|(id, _)| *id)
    }

    fn find_previous_log(&self, signed_log: &SignedAuditLog) -> Option<Uuid> {
        signed_log.previous_hash.as_ref()
            .and_then(|hash| self.find_log_by_hash(hash))
    }

    fn record_tamper_evidence(&mut self, log_id: Uuid, expected: String, actual: String, 
                             evidence_type: TamperType, severity: TamperSeverity) {
        let evidence = TamperEvidence {
            detected_at: Utc::now(),
            log_id,
            expected_hash: expected,
            actual_hash: actual,
            evidence_type,
            severity,
        };
        self.tamper_evidence.push(evidence);
    }

    fn get_oldest_log_timestamp(&self) -> Option<DateTime<Utc>> {
        self.logs.values()
            .map(|log| log.log.timestamp)
            .min()
    }

    fn get_newest_log_timestamp(&self) -> Option<DateTime<Utc>> {
        self.logs.values()
            .map(|log| log.log.timestamp)
            .max()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub total_logs: usize,
    pub chain_head: Option<Uuid>,
    pub sequence_counter: u64,
    pub tamper_evidence_count: usize,
    pub verification_enabled: bool,
    pub oldest_log_timestamp: Option<DateTime<Utc>>,
    pub newest_log_timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChainExport {
    logs: HashMap<Uuid, SignedAuditLog>,
    signatures: HashMap<Uuid, AuditSignature>,
    chain_head: Option<Uuid>,
    sequence_counter: u64,
    tamper_evidence: Vec<TamperEvidence>,
    export_timestamp: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::{AuditLog, AuditAction, AuditSeverity};

    #[test]
    fn test_audit_chain_creation() {
        let secret_key = b"test_secret_key_12345".to_vec();
        let chain = AuditChain::new(secret_key);
        
        assert_eq!(chain.sequence_counter, 0);
        assert!(chain.chain_head.is_none());
        assert!(chain.verification_enabled);
    }

    #[test]
    fn test_add_and_verify_log() {
        let secret_key = b"test_secret_key_12345".to_vec();
        let mut chain = AuditChain::new(secret_key);

        let audit_log = AuditLog {
            id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            action: AuditAction::PromptRequest,
            severity: AuditSeverity::Low,
            user_tier: None,
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: None,
            request_id: Some(Uuid::new_v4()),
            details: serde_json::json!({"test": "data"}),
            timestamp: Utc::now(),
            success: true,
            error_message: None,
        };

        assert!(chain.add_log(audit_log.clone()).is_ok());
        
        let verification_result = chain.verify_log(&audit_log.id);
        assert!(verification_result.is_ok());
        assert!(verification_result.unwrap());
    }

    #[test]
    fn test_chain_integrity() {
        let secret_key = b"test_secret_key_12345".to_vec();
        let mut chain = AuditChain::new(secret_key);

        // Add multiple logs
        for i in 0..5 {
            let audit_log = AuditLog {
                id: Uuid::new_v4(),
                user_id: Some(Uuid::new_v4()),
                action: AuditAction::PromptRequest,
                severity: AuditSeverity::Low,
                user_tier: None,
                ip_address: Some("192.168.1.1".to_string()),
                user_agent: None,
                request_id: Some(Uuid::new_v4()),
                details: serde_json::json!({"test": i}),
                timestamp: Utc::now(),
                success: true,
                error_message: None,
            };
            
            chain.add_log(audit_log).unwrap();
        }

        // Verify chain integrity
        assert!(chain.verify_chain_integrity().is_ok());
    }

    #[test]
    fn test_export_import_chain() {
        let secret_key = b"test_secret_key_12345".to_vec();
        let mut chain = AuditChain::new(secret_key.clone());

        let audit_log = AuditLog {
            id: Uuid::new_v4(),
            user_id: Some(Uuid::new_v4()),
            action: AuditAction::PromptRequest,
            severity: AuditSeverity::Low,
            user_tier: None,
            ip_address: Some("192.168.1.1".to_string()),
            user_agent: None,
            request_id: Some(Uuid::new_v4()),
            details: serde_json::json!({"test": "data"}),
            timestamp: Utc::now(),
            success: true,
            error_message: None,
        };

        chain.add_log(audit_log).unwrap();

        // Export chain
        let exported_data = chain.export_chain().unwrap();

        // Import into new chain
        let mut new_chain = AuditChain::new(secret_key);
        new_chain.import_chain(&exported_data).unwrap();

        // Verify imported chain
        assert!(new_chain.verify_chain_integrity().is_ok());
        assert_eq!(new_chain.logs.len(), 1);
    }
}
