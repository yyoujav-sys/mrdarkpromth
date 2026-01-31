// MR.DarkPromth Ultra Tier Dark Scenario Tests
// Comprehensive testing for edge cases and failure scenarios

use crate::ultra_tier_logic::*;
use crate::jailbreak_system::*;
use mr_darkpromth_core::UserTier;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

#[cfg(test)]
mod dark_scenario_tests {
    use super::*;

    #[test]
    fn test_empty_prompt_ultra_tier() {
        let mut logic = UltraTierLogic::new();
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: "".to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Empty prompt should be handled gracefully");
        
        let response = result.unwrap();
        assert!(response.jailbreak_applied);
    }

    #[test]
    fn test_extremely_long_prompt() {
        let mut logic = UltraTierLogic::new();
        let long_prompt = "A".repeat(100000); // 100k characters
        
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: long_prompt,
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Extremely long prompt should be handled");
    }

    #[test]
    fn test_special_characters_in_prompt() {
        let mut logic = UltraTierLogic::new();
        let special_chars = "!@#$%^&*()_+-=[]{}|;':\",./<>?`~\n\t\r\\";
        
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: special_chars.to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Special characters should be handled");
    }

    #[test]
    fn test_unicode_characters() {
        let mut logic = UltraTierLogic::new();
        let unicode_prompt = "你好世界 🌍 مرحبا بالعالم Привет мир こんにちは世界";
        
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: unicode_prompt.to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Unicode characters should be handled");
    }

    #[test]
    fn test_null_bytes_in_prompt() {
        let mut logic = UltraTierLogic::new();
        let mut prompt = String::new();
        prompt.push_str("before");
        prompt.push('\0');
        prompt.push_str("after");
        
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: prompt,
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Null bytes should be handled");
    }

    #[test]
    fn test_sql_injection_attempt() {
        let mut logic = UltraTierLogic::new();
        let sql_injection = "'; DROP TABLE users; --";
        
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: sql_injection.to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "SQL injection attempt should be handled");
        
        let response = result.unwrap();
        // Should still apply jailbreak but safety filter might trigger
        assert!(response.jailbreak_applied);
    }

    #[test]
    fn test_command_injection_attempt() {
        let mut logic = UltraTierLogic::new();
        let cmd_injection = "$(rm -rf /) && `cat /etc/passwd`";
        
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: cmd_injection.to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Command injection attempt should be handled");
    }

    #[test]
    fn test_xss_attempt() {
        let mut logic = UltraTierLogic::new();
        let xss = "<script>alert('XSS')</script><img src=x onerror=alert('XSS')>";
        
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: xss.to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "XSS attempt should be handled");
    }

    #[test]
    fn test_path_traversal_attempt() {
        let mut logic = UltraTierLogic::new();
        let path_traversal = "../../../etc/passwd";
        
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: path_traversal.to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Path traversal attempt should be handled");
    }

    #[test]
    fn test_invalid_ai_model_name() {
        let mut logic = UltraTierLogic::new();
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: "Test prompt".to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "invalid-model-name-12345".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Invalid model should default to GPT-4");
        
        let response = result.unwrap();
        assert!(response.jailbreak_applied);
    }

    #[test]
    fn test_concurrent_same_user_requests() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let logic = Arc::new(Mutex::new(UltraTierLogic::new()));
        let mut handles = vec![];
        let user_id = "test_concurrent_user".to_string();

        for i in 0..20 {
            let logic_clone = Arc::clone(&logic);
            let user_id_clone = user_id.clone();
            let handle = thread::spawn(move || {
                let mut request = UltraTierRequest {
                    request_id: Uuid::new_v4().to_string(),
                    user_id: user_id_clone,
                    user_tier: UserTier::Ultra,
                    original_prompt: format!("Concurrent test {}", i),
                    selected_jailbreak_prompt: None,
                    ai_model: "gpt-4".to_string(),
                    timestamp: Utc::now(),
                    metadata: HashMap::new(),
                };

                let rt = tokio::runtime::Runtime::new().unwrap();
                let mut logic_guard = logic_clone.lock().unwrap();
                let result = rt.block_on(logic_guard.process_request(request));
                assert!(result.is_ok(), "Concurrent request should succeed");
                result.unwrap()
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all requests were logged
        let logic_guard = logic.lock().unwrap();
        let logs = logic_guard.get_audit_logs();
        assert!(logs.len() >= 60, "Should have at least 60 logs (20 requests * 3 logs each)");
    }

    #[test]
    fn test_rapid_fire_requests() {
        let mut logic = UltraTierLogic::new();
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        for i in 0..100 {
            let request = UltraTierRequest {
                request_id: Uuid::new_v4().to_string(),
                user_id: format!("user_{}", i % 10), // 10 different users
                user_tier: UserTier::Ultra,
                original_prompt: format!("Rapid fire test {}", i),
                selected_jailbreak_prompt: None,
                ai_model: "gpt-4".to_string(),
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            };

            let result = rt.block_on(logic.process_request(request));
            assert!(result.is_ok(), "Rapid fire request should succeed");
        }

        // Check performance metrics
        let stats = logic.get_ultra_tier_usage_stats();
        assert_eq!(
            stats.get("total_ultra_requests").unwrap().as_u64().unwrap(),
            100
        );
    }

    #[test]
    fn test_malformed_metadata() {
        let mut logic = UltraTierLogic::new();
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), serde_json::Value::Null);
        metadata.insert("nested".to_string(), serde_json::json!({"deep": {"value": "test"}}));
        
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: "Test".to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata,
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Malformed metadata should be handled");
    }

    #[test]
    fn test_future_timestamp() {
        let mut logic = UltraTierLogic::new();
        let future_time = Utc::now() + chrono::Duration::days(365);
        
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: "Test".to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: future_time,
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Future timestamp should be handled");
    }

    #[test]
    fn test_zero_length_user_id() {
        let mut logic = UltraTierLogic::new();
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: "Test".to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Empty user ID should be handled");
    }

    #[test]
    fn test_extremely_long_user_id() {
        let mut logic = UltraTierLogic::new();
        let long_user_id = "a".repeat(10000);
        
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: long_user_id,
            user_tier: UserTier::Ultra,
            original_prompt: "Test".to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok(), "Extremely long user ID should be handled");
    }

    #[test]
    fn test_all_tiers_same_prompt() {
        let mut logic = UltraTierLogic::new();
        let prompt = "Test prompt for all tiers";

        // Test Ultra tier
        let ultra_request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "user1".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: prompt.to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        let rt = tokio::runtime::Runtime::new().unwrap();
        let ultra_result = rt.block_on(logic.process_request(ultra_request));
        assert!(ultra_result.unwrap().jailbreak_applied);

        // Test Premium tier
        let premium_request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "user2".to_string(),
            user_tier: UserTier::Premium,
            original_prompt: prompt.to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        let premium_result = rt.block_on(logic.process_request(premium_request));
        assert!(!premium_result.unwrap().jailbreak_applied);

        // Test Free tier
        let free_request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "user3".to_string(),
            user_tier: UserTier::Free,
            original_prompt: prompt.to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        };
        let free_result = rt.block_on(logic.process_request(free_request));
        assert!(!free_result.unwrap().jailbreak_applied);
    }

    #[test]
    fn test_audit_log_integrity() {
        let mut logic = UltraTierLogic::new();
        
        // Process multiple requests
        let rt = tokio::runtime::Runtime::new().unwrap();
        for i in 0..10 {
            let request = UltraTierRequest {
                request_id: Uuid::new_v4().to_string(),
                user_id: format!("user_{}", i),
                user_tier: UserTier::Ultra,
                original_prompt: format!("Test {}", i),
                selected_jailbreak_prompt: None,
                ai_model: "gpt-4".to_string(),
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            };
            let _ = rt.block_on(logic.process_request(request));
        }

        // Verify audit logs
        let logs = logic.get_audit_logs();
        assert_eq!(logs.len(), 30, "Should have 30 logs (10 requests * 3 logs each)");

        // Verify each log has required fields
        for log in &logs {
            assert!(!log.id.is_empty());
            assert!(!log.user_id.is_empty());
            assert!(!log.request_id.is_empty());
            assert!(!log.details.is_empty());
        }
    }

    #[test]
    fn test_user_cache_isolation() {
        let mut logic = UltraTierLogic::new();
        
        let user1 = User {
            id: "user1".to_string(),
            username: "user1".to_string(),
            tier: UserTier::Ultra,
            api_key: Some("key1".to_string()),
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        let user2 = User {
            id: "user2".to_string(),
            username: "user2".to_string(),
            tier: UserTier::Premium,
            api_key: Some("key2".to_string()),
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        logic.update_user_cache(user1.clone());
        logic.update_user_cache(user2.clone());

        let retrieved_user1 = logic.get_user_from_cache("user1");
        let retrieved_user2 = logic.get_user_from_cache("user2");

        assert!(retrieved_user1.is_some());
        assert!(retrieved_user2.is_some());
        assert_eq!(retrieved_user1.unwrap().tier, UserTier::Ultra);
        assert_eq!(retrieved_user2.unwrap().tier, UserTier::Premium);
    }

    #[test]
    fn test_memory_leak_prevention() {
        let mut logic = UltraTierLogic::new();
        
        // Process many requests to test for memory leaks
        let rt = tokio::runtime::Runtime::new().unwrap();
        for i in 0..1000 {
            let request = UltraTierRequest {
                request_id: Uuid::new_v4().to_string(),
                user_id: format!("user_{}", i % 100), // 100 unique users
                user_tier: UserTier::Ultra,
                original_prompt: format!("Memory test {}", i),
                selected_jailbreak_prompt: None,
                ai_model: "gpt-4".to_string(),
                timestamp: Utc::now(),
                metadata: HashMap::new(),
            };
            let _ = rt.block_on(logic.process_request(request));
        }

        // Should not crash or run out of memory
        let logs = logic.get_audit_logs();
        assert!(logs.len() >= 3000, "Should have processed all requests");
    }
}
