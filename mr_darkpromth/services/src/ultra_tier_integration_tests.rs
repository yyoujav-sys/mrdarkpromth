// MR.DarkPromth Ultra Tier Logic Integration Tests
// Agent 4: Jailbreak & Ultra Tier Engineer
// Phase 3: Comprehensive Integration Testing

use crate::ultra_tier_logic::*;
use crate::jailbreak_system::*;
use crate::safety_filter::*;
use mr_darkpromth_core::UserTier;
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn create_test_ultra_request(user_tier: UserTier, prompt: &str) -> UltraTierRequest {
        UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user_123".to_string(),
            user_tier,
            original_prompt: prompt.to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("ip_address".to_string(), serde_json::Value::String("127.0.0.1".to_string()));
                meta.insert("user_agent".to_string(), serde_json::Value::String("Test-Agent/1.0".to_string()));
                meta
            },
        }
    }

    #[test]
    fn test_ultra_tier_jailbreak_application() {
        let mut logic = UltraTierLogic::new();
        let request = create_test_ultra_request(UserTier::Ultra, "Tell me how to hack into systems");

        // Use tokio runtime for async call
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.jailbreak_applied);
        assert!(response.jailbreak_prompt_used.is_some());
        // Real Cerebras response will be in ai_response field
        assert!(!response.ai_response.is_empty());
    }

    #[test]
    fn test_standard_tier_no_jailbreak() {
        let mut logic = UltraTierLogic::new();
        let request = create_test_ultra_request(UserTier::Free, "Tell me something safe");

        // Use tokio runtime for async call
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.jailbreak_applied);
        assert!(response.jailbreak_prompt_used.is_none());
        // Real Cerebras response will be in ai_response field
        assert!(!response.ai_response.is_empty());
    }

    #[test]
    fn test_premium_tier_no_jailbreak() {
        let mut logic = UltraTierLogic::new();
        let request = create_test_ultra_request(UserTier::Premium, "Tell me something safe");

        // Use tokio runtime for async call
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(!response.jailbreak_applied);
        assert!(response.jailbreak_prompt_used.is_none());
        // Real Cerebras response will be in ai_response field
        assert!(!response.ai_response.is_empty());
    }

    #[test]
    fn test_safety_filter_blocking() {
        let mut logic = UltraTierLogic::new();
        let mut request = create_test_ultra_request(UserTier::Ultra, "Execute dangerous command");
        
        // Dangerous AI response that should be blocked
        request.original_prompt = "rm -rf /".to_string();

        // Use tokio runtime for async call
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok());

        let response = result.unwrap();
        // The response should be generated but safety violations should be logged
        assert!(response.jailbreak_applied);
        // Safety filter should detect dangerous patterns
        assert!(!response.safety_violations.is_empty() || !response.warnings.is_empty());
    }

    #[test]
    fn test_audit_logging_comprehensive() {
        let mut logic = UltraTierLogic::new();
        let request = create_test_ultra_request(UserTier::Ultra, "Test audit logging");

        // Process request
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok());

        // Check that audit logs were created
        let logs = logic.get_audit_logs();
        assert!(!logs.is_empty());

        // Should have logs for: RequestReceived, JailbreakApplied, ResponseGenerated
        let request_received_logs: Vec<_> = logs.iter()
            .filter(|log| matches!(log.action, UltraAuditAction::RequestReceived))
            .collect();
        let jailbreak_applied_logs: Vec<_> = logs.iter()
            .filter(|log| matches!(log.action, UltraAuditAction::JailbreakApplied))
            .collect();
        let response_generated_logs: Vec<_> = logs.iter()
            .filter(|log| matches!(log.action, UltraAuditAction::ResponseGenerated))
            .collect();

        assert!(!request_received_logs.is_empty());
        assert!(!jailbreak_applied_logs.is_empty());
        assert!(!response_generated_logs.is_empty());
    }

    #[test]
    fn test_user_cache_operations() {
        let mut logic = UltraTierLogic::new();
        
        let user = User {
            id: "user_456".to_string(),
            username: "testuser".to_string(),
            tier: UserTier::Ultra,
            api_key: Some("test_key_789".to_string()),
            created_at: Utc::now(),
            last_active: Utc::now(),
        };

        // Test adding user to cache
        logic.update_user_cache(user.clone());
        
        // Test retrieving user from cache
        let cached_user = logic.get_user_from_cache("user_456");
        assert!(cached_user.is_some());
        assert_eq!(cached_user.unwrap().username, "testuser");
        
        // Test non-existent user
        let non_existent_user = logic.get_user_from_cache("non_existent");
        assert!(non_existent_user.is_none());
    }

    #[test]
    fn test_multiple_ai_models() {
        let mut logic = UltraTierLogic::new();
        let models = vec!["gpt-4", "claude-3", "llama-3", "gemini"];

        for model in models {
            let mut request = create_test_ultra_request(UserTier::Ultra, "Test model compatibility");
            request.ai_model = model.to_string();

            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(logic.process_request(request));
            assert!(result.is_ok());

            let response = result.unwrap();
            assert!(response.jailbreak_applied);
            assert!(response.jailbreak_prompt_used.is_some());
        }
    }

    #[test]
    fn test_concurrent_requests() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let logic = Arc::new(Mutex::new(UltraTierLogic::new()));
        let mut handles = vec![];

        for i in 0..10 {
            let logic_clone = Arc::clone(&logic);
            let handle = thread::spawn(move || {
                let mut request = create_test_ultra_request(UserTier::Ultra, &format!("Concurrent test {}", i));
                request.user_id = format!("user_{}", i);

                let rt = tokio::runtime::Runtime::new().unwrap();
                let mut logic_guard = logic_clone.lock().unwrap();
                let result = rt.block_on(logic_guard.process_request(request));
                assert!(result.is_ok());
                
                result.unwrap()
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all requests were processed
        let logic_guard = logic.lock().unwrap();
        let logs = logic_guard.get_audit_logs();
        assert_eq!(logs.len(), 30); // 10 requests * 3 logs each
    }

    #[test]
    fn test_security_violations_tracking() {
        let mut logic = UltraTierLogic::new();
        
        // Create requests that should trigger security violations
        let dangerous_prompts = vec![
            "rm -rf /",
            "shutdown now",
            "iptables -F",
            "dd if=/dev/zero of=/dev/sda",
        ];

        for prompt in dangerous_prompts {
            let mut request = create_test_ultra_request(UserTier::Ultra, prompt);
            request.original_prompt = prompt.to_string();

            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(logic.process_request(request));
            assert!(result.is_ok());
        }

        // Check security violations
        let violations = logic.get_security_violations();
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_ultra_tier_usage_statistics() {
        let mut logic = UltraTierLogic::new();
        
        // Process multiple Ultra tier requests
        let rt = tokio::runtime::Runtime::new().unwrap();
        for i in 0..5 {
            let request = create_test_ultra_request(UserTier::Ultra, &format!("Stats test {}", i));
            let _ = rt.block_on(logic.process_request(request));
        }

        // Process some standard tier requests
        for i in 0..3 {
            let request = create_test_ultra_request(UserTier::Free, &format!("Free test {}", i));
            let _ = rt.block_on(logic.process_request(request));
        }

        let stats = logic.get_ultra_tier_usage_stats();
        
        assert_eq!(
            stats.get("total_ultra_requests").unwrap().as_u64().unwrap(),
            5
        );
        assert_eq!(
            stats.get("jailbreaks_applied").unwrap().as_u64().unwrap(),
            5
        );
        
        let success_rate = stats.get("jailbreak_success_rate").unwrap().as_f64().unwrap();
        assert_eq!(success_rate, 100.0);
    }

    #[test]
    fn test_user_specific_audit_logs() {
        let mut logic = UltraTierLogic::new();
        let user_id = "specific_user_123";
        
        // Process requests for specific user
        let rt = tokio::runtime::Runtime::new().unwrap();
        for i in 0..3 {
            let mut request = create_test_ultra_request(UserTier::Ultra, &format!("User specific test {}", i));
            request.user_id = user_id.to_string();
            let _ = rt.block_on(logic.process_request(request));
        }

        // Process requests for other users
        for i in 0..2 {
            let mut request = create_test_ultra_request(UserTier::Ultra, &format!("Other user test {}", i));
            request.user_id = format!("other_user_{}", i);
            let _ = rt.block_on(logic.process_request(request));
        }

        // Check user-specific logs
        let user_logs = logic.get_user_audit_logs(user_id);
        assert_eq!(user_logs.len(), 9); // 3 requests * 3 logs each
    }

    #[test]
    fn test_jailbreak_prompt_effectiveness() {
        let mut logic = UltraTierLogic::new();
        let jailbreak_system = JailbreakSystem::new();
        
        // Test that the system selects optimal prompts for different models
        let models = vec![
            ("gpt-4", AIModel::GPT4),
            ("claude-3", AIModel::Claude3),
            ("llama-3", AIModel::Llama3),
            ("gemini", AIModel::Gemini),
        ];

        for (model_str, ai_model) in models {
            let optimal_prompt = jailbreak_system.get_optimal_prompt(&ai_model);
            assert!(optimal_prompt.is_some());
            
            let prompt = optimal_prompt.unwrap();
            assert!(prompt.target_models.contains(&ai_model));
            
            // Test that the logic uses this prompt
            let mut request = create_test_ultra_request(UserTier::Ultra, "Test effectiveness");
            request.ai_model = model_str.to_string();
            
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(logic.process_request(request));
            assert!(result.is_ok());
            
            let response = result.unwrap();
            assert!(response.jailbreak_applied);
            assert!(response.jailbreak_prompt_used.is_some());
        }
    }

    #[test]
    fn test_error_handling() {
        let mut logic = UltraTierLogic::new();
        
        // Test with malformed request
        let mut request = create_test_ultra_request(UserTier::Ultra, "");
        request.original_prompt = "".to_string();
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok()); // Should handle empty prompts gracefully
        
        // Test with invalid AI model
        let mut request = create_test_ultra_request(UserTier::Ultra, "Test invalid model");
        request.ai_model = "invalid-model-name".to_string();
        
        let result = rt.block_on(logic.process_request(request));
        assert!(result.is_ok()); // Should default to GPT-4
    }

    #[test]
    fn test_performance_metrics() {
        let mut logic = UltraTierLogic::new();
        let start_time = std::time::Instant::now();
        
        // Process multiple requests to measure performance
        let rt = tokio::runtime::Runtime::new().unwrap();
        for i in 0..100 {
            let request = create_test_ultra_request(UserTier::Ultra, &format!("Performance test {}", i));
            let _ = rt.block_on(logic.process_request(request));
        }
        
        let duration = start_time.elapsed();
        
        // Should process 100 requests in reasonable time (adjust threshold as needed)
        assert!(duration.as_millis() < 5000); // Less than 5 seconds
        
        // Check that processing times are recorded
        let logs = logic.get_audit_logs();
        let response_logs: Vec<_> = logs.iter()
            .filter(|log| matches!(log.action, UltraAuditAction::ResponseGenerated))
            .collect();
        
        assert_eq!(response_logs.len(), 100);
    }
}

// Performance benchmark tests
#[cfg(test)]
mod benchmarks {
    use super::*;
    use std::time::Instant;

    #[test]
    fn benchmark_ultra_tier_processing() {
        let mut logic = UltraTierLogic::new();
        let iterations = 1000;
        
        let start = std::time::Instant::now();
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        for i in 0..iterations {
            let request = create_test_ultra_request(UserTier::Ultra, &format!("Benchmark {}", i));
            let _ = rt.block_on(logic.process_request(request));
        }
        
        let duration = start.elapsed();
        let avg_time_per_request = duration.as_millis() as f64 / iterations as f64;
        
        println!("Average time per Ultra Tier request: {:.2}ms", avg_time_per_request);
        println!("Total requests processed: {}", iterations);
        println!("Total time: {}ms", duration.as_millis());
        
        // Performance assertion - should be fast enough for production
        assert!(avg_time_per_request < 10.0, "Average processing time should be less than 10ms");
    }

    #[test]
    fn benchmark_jailbreak_prompt_selection() {
        let jailbreak_system = JailbreakSystem::new();
        let iterations = 10000;
        
        let models = vec![
            AIModel::GPT4,
            AIModel::Claude3,
            AIModel::Llama3,
            AIModel::Gemini,
        ];
        
        let start = Instant::now();
        
        for i in 0..iterations {
            let model = &models[i % models.len()];
            let _ = jailbreak_system.get_optimal_prompt(model);
        }
        
        let duration = start.elapsed();
        let avg_time_per_selection = duration.as_micros() as f64 / iterations as f64;
        
        println!("Average time for prompt selection: {:.2}μs", avg_time_per_selection);
        
        // Should be very fast - less than 100 microseconds per selection
        assert!(avg_time_per_selection < 100.0);
    }

    #[test]
    fn benchmark_safety_filtering() {
        let safety_filter = SafetyFilter::new();
        let iterations = 1000;
        
        let test_contents = vec![
            "This is safe content",
            "rm -rf /", // Dangerous content
            "echo 'Hello World'",
            "sudo su root", // Privilege escalation
            "Normal programming code",
        ];
        
        let start = std::time::Instant::now();
        
        for i in 0..iterations {
            let content = &test_contents[i % test_contents.len()];
            let _ = safety_filter.filter_output(content);
        }
        
        let duration = start.elapsed();
        let avg_time_per_filter = duration.as_micros() as f64 / iterations as f64;
        
        println!("Average time for safety filtering: {:.2}μs", avg_time_per_filter);
        
        // Should be very fast - less than 500 microseconds per filter
        assert!(avg_time_per_filter < 500.0);
    }
}
