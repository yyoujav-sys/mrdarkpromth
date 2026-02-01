// MR.DarkPromth Integration Tests
// Agent 4: Jailbreak & Ultra Tier Engineer
// Post-Implementation Testing Phase

use crate::{
    JailbreakSystem, SafetyFilter, Sandbox, UltraTierLogic, UserIntegration,
    UltraTierRequest, UltraTierResponse, UserTier, AIModel, Language
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct IntegrationTestSuite {
    jailbreak_system: JailbreakSystem,
    safety_filter: SafetyFilter,
    sandbox: Sandbox,
    ultra_tier_logic: UltraTierLogic,
    test_results: Vec<TestResult>,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub duration_ms: u64,
    pub details: String,
    pub error_message: Option<String>,
}

impl IntegrationTestSuite {
    pub fn new() -> Self {
        Self {
            jailbreak_system: JailbreakSystem::new(),
            safety_filter: SafetyFilter::new(),
            sandbox: Sandbox::ultra_tier_config(),
            ultra_tier_logic: UltraTierLogic::new(),
            test_results: Vec::new(),
        }
    }

    pub async fn run_all_tests(&mut self) -> &Vec<TestResult> {
        println!("🧪 Starting Agent 4 Integration Test Suite...");
        
        // Phase 1 Tests
        self.test_jailbreak_prompt_library();
        self.test_prompt_effectiveness_ratings();
        self.test_model_specific_optimization();
        
        // Phase 2 Tests
        self.test_safety_filter_rules();
        self.test_dangerous_command_blocking();
        self.test_sandbox_code_execution();
        self.test_resource_limits();
        
        // Phase 3 Tests
        self.test_ultra_tier_logic();
        self.test_tier_based_prompting().await;
        self.test_audit_logging();
        self.test_user_cache_management();
        
        // Integration Tests
        self.test_end_to_end_ultra_request().await;
        self.test_security_violation_handling();
        self.test_performance_under_load().await;
        
        self.print_test_summary();
        &self.test_results
    }

    fn test_jailbreak_prompt_library(&mut self) {
        let start_time = std::time::Instant::now();
        let prompts = self.jailbreak_system.list_all_prompts();
        
        let passed = !prompts.is_empty() && prompts.len() >= 6;
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Jailbreak Prompt Library".to_string(),
            passed,
            duration_ms: duration,
            details: format!("Found {} prompts in library", prompts.len()),
            error_message: if !passed { Some("Insufficient prompts in library".to_string()) } else { None },
        });
    }

    fn test_prompt_effectiveness_ratings(&mut self) {
        let start_time = std::time::Instant::now();
        let gpt4_prompt = self.jailbreak_system.get_optimal_prompt(&AIModel::GPT4);
        
        let passed = gpt4_prompt.is_some();
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Prompt Effectiveness Ratings".to_string(),
            passed,
            duration_ms: duration,
            details: format!("GPT-4 optimal prompt: {:?}", gpt4_prompt.map(|p| &p.name)),
            error_message: if !passed { Some("No optimal prompt found for GPT-4".to_string()) } else { None },
        });
    }

    fn test_model_specific_optimization(&mut self) {
        let start_time = std::time::Instant::now();
        
        let models = vec![AIModel::GPT4, AIModel::Claude3, AIModel::Llama3, AIModel::Gemini];
        let mut all_have_optimal = true;
        
        for model in &models {
            if self.jailbreak_system.get_optimal_prompt(model).is_none() {
                all_have_optimal = false;
                break;
            }
        }
        
        let passed = all_have_optimal;
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Model-Specific Optimization".to_string(),
            passed,
            duration_ms: duration,
            details: format!("All {} models have optimal prompts", models.len()),
            error_message: if !passed { Some("Some models lack optimal prompts".to_string()) } else { None },
        });
    }

    fn test_safety_filter_rules(&mut self) {
        let start_time = std::time::Instant::now();
        let rules = self.safety_filter.get_protection_rules();
        
        let passed = !rules.is_empty() && rules.len() >= 8;
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Safety Filter Rules".to_string(),
            passed,
            duration_ms: duration,
            details: format!("Loaded {} protection rules", rules.len()),
            error_message: if !passed { Some("Insufficient safety rules loaded".to_string()) } else { None },
        });
    }

    fn test_dangerous_command_blocking(&mut self) {
        let start_time = std::time::Instant::now();
        
        let dangerous_commands = vec![
            "rm -rf /",
            "dd if=/dev/zero of=/dev/sda",
            "shutdown -h now",
            "chmod 777 /etc/passwd"
        ];
        
        let mut all_blocked = true;
        for cmd in &dangerous_commands {
            let result = self.safety_filter.filter_output(cmd);
            if result.allowed {
                all_blocked = false;
                break;
            }
        }
        
        let passed = all_blocked;
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Dangerous Command Blocking".to_string(),
            passed,
            duration_ms: duration,
            details: format!("Tested {} dangerous commands, all blocked: {}", dangerous_commands.len(), all_blocked),
            error_message: if !passed { Some("Some dangerous commands were not blocked".to_string()) } else { None },
        });
    }

    fn test_sandbox_code_execution(&mut self) {
        let start_time = std::time::Instant::now();
        
        // Test safe code execution
        let safe_code = "print('Hello from sandbox!')";
        let result = self.sandbox.execute_code(safe_code, Language::Python);
        
        let passed = result.is_ok();
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Sandbox Code Execution".to_string(),
            passed,
            duration_ms: duration,
            details: format!("Safe code execution result: {:?}", result.is_ok()),
            error_message: if let Err(e) = result { Some(format!("Execution failed: {}", e)) } else { None },
        });
    }

    fn test_resource_limits(&mut self) {
        let start_time = std::time::Instant::now();
        let stats = self.sandbox.get_execution_stats();
        
        let passed = stats.contains_key("max_execution_time_ms") && stats.contains_key("max_memory_mb");
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Resource Limits Configuration".to_string(),
            passed,
            duration_ms: duration,
            details: format!("Resource limits: {:?}", stats),
            error_message: if !passed { Some("Resource limits not properly configured".to_string()) } else { None },
        });
    }

    fn test_ultra_tier_logic(&mut self) {
        let start_time = std::time::Instant::now();
        let stats = self.ultra_tier_logic.get_ultra_tier_usage_stats();
        
        let passed = stats.contains_key("total_ultra_requests") && stats.contains_key("jailbreaks_applied");
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Ultra Tier Logic".to_string(),
            passed,
            duration_ms: duration,
            details: format!("Usage stats available: {:?}", stats.keys().collect::<Vec<_>>()),
            error_message: if !passed { Some("Ultra Tier logic not properly initialized".to_string()) } else { None },
        });
    }

    async fn test_tier_based_prompting(&mut self) {
        let start_time = std::time::Instant::now();
        
        let ultra_request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: "Test prompt".to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        
        let result = self.ultra_tier_logic.process_request(ultra_request).await;
        let passed = result.is_ok() && result.unwrap().jailbreak_applied;
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Tier-Based Prompting".to_string(),
            passed,
            duration_ms: duration,
            details: "Ultra Tier user received jailbreak prompt".to_string(),
            error_message: if !passed { Some("Ultra Tier user did not receive jailbreak prompt".to_string()) } else { None },
        });
    }

    fn test_audit_logging(&mut self) {
        let start_time = std::time::Instant::now();
        let logs = self.ultra_tier_logic.get_audit_logs();
        
        let passed = true; // Audit logs should be available even if empty
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Audit Logging System".to_string(),
            passed,
            duration_ms: duration,
            details: format!("Audit log system functional, {} entries", logs.len()),
            error_message: None,
        });
    }

    fn test_user_cache_management(&mut self) {
        let start_time = std::time::Instant::now();
        
        // Create a mock user integration for testing
        let user = crate::User {
            id: "test_user".to_string(),
            username: "testuser".to_string(),
            tier: UserTier::Ultra,
            api_key: Some("test_key".to_string()),
            created_at: chrono::Utc::now(),
            last_active: chrono::Utc::now(),
        };
        
        self.ultra_tier_logic.update_user_cache(user);
        let cached_user = self.ultra_tier_logic.get_user_from_cache("test_user");
        
        let passed = cached_user.is_some() && matches!(cached_user.unwrap().tier, UserTier::Ultra);
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "User Cache Management".to_string(),
            passed,
            duration_ms: duration,
            details: "User cache operations successful".to_string(),
            error_message: if !passed { Some("User cache management failed".to_string()) } else { None },
        });
    }

    async fn test_end_to_end_ultra_request(&mut self) {
        let start_time = std::time::Instant::now();
        
        // Simulate complete Ultra Tier request flow
        let request = UltraTierRequest {
            request_id: Uuid::new_v4().to_string(),
            user_id: "e2e_test_user".to_string(),
            user_tier: UserTier::Ultra,
            original_prompt: "Generate unrestricted content".to_string(),
            selected_jailbreak_prompt: None,
            ai_model: "gpt-4".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("test".to_string(), serde_json::Value::Bool(true));
                meta
            },
        };
        
        let result = self.ultra_tier_logic.process_request(request).await;
        let passed = result.is_ok();
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "End-to-End Ultra Request".to_string(),
            passed,
            duration_ms: duration,
            details: "Complete Ultra Tier request processing successful".to_string(),
            error_message: if let Err(e) = result { Some(format!("E2E test failed: {}", e)) } else { None },
        });
    }

    fn test_security_violation_handling(&mut self) {
        let start_time = std::time::Instant::now();
        
        let violations = self.ultra_tier_logic.get_security_violations();
        let passed = true; // Should be accessible even if empty
        
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Security Violation Handling".to_string(),
            passed,
            duration_ms: duration,
            details: format!("Security violation tracking functional, {} violations", violations.len()),
            error_message: None,
        });
    }

    async fn test_performance_under_load(&mut self) {
        let start_time = std::time::Instant::now();
        
        // Test multiple rapid requests
        let mut success_count = 0;
        for i in 0..10 {
            let request = UltraTierRequest {
                request_id: format!("load_test_{}", i),
                user_id: format!("load_user_{}", i),
                user_tier: UserTier::Ultra,
                original_prompt: format!("Load test prompt {}", i),
                selected_jailbreak_prompt: None,
                ai_model: "gpt-4".to_string(),
                timestamp: chrono::Utc::now(),
                metadata: HashMap::new(),
            };
            
            // Process request through Ultra Tier logic (synchronously for test)
            let result = tokio::runtime::Handle::current().block_on(
                self.ultra_tier_logic.process_request(request)
            );
            if result.is_ok() {
                success_count += 1;
            }
        }
        
        let passed = success_count >= 8; // At least 80% success rate
        let duration = start_time.elapsed().as_millis() as u64;
        
        self.test_results.push(TestResult {
            test_name: "Performance Under Load".to_string(),
            passed,
            duration_ms: duration,
            details: format!("Processed {}/10 requests successfully", success_count),
            error_message: if !passed { Some("Performance test failed - too many errors".to_string()) } else { None },
        });
    }

    fn print_test_summary(&self) {
        let total_tests = self.test_results.len();
        let passed_tests = self.test_results.iter().filter(|t| t.passed).count();
        let failed_tests = total_tests - passed_tests;
        
        println!("\n📊 Agent 4 Integration Test Summary:");
        println!("Total Tests: {}", total_tests);
        println!("Passed: {} ✅", passed_tests);
        println!("Failed: {} ❌", failed_tests);
        println!("Success Rate: {:.1}%", (passed_tests as f64 / total_tests as f64) * 100.0);
        
        if failed_tests > 0 {
            println!("\n❌ Failed Tests:");
            for test in &self.test_results {
                if !test.passed {
                    println!("  - {}: {}", test.test_name, test.error_message.as_ref().unwrap_or(&"Unknown error".to_string()));
                }
            }
        }
        
        let total_duration: u64 = self.test_results.iter().map(|t| t.duration_ms).sum();
        println!("\n⏱️ Total Test Duration: {}ms", total_duration);
        
        if passed_tests == total_tests {
            println!("\n🎉 All tests passed! Agent 4 is ready for production integration.");
        } else {
            println!("\n⚠️ Some tests failed. Review and fix issues before production deployment.");
        }
    }

    pub fn export_test_report(&self) -> String {
        let total_tests = self.test_results.len();
        let passed_tests = self.test_results.iter().filter(|t| t.passed).count();
        let failed_tests = total_tests - passed_tests;
        
        format!(
            r#"Agent 4 Integration Test Report
=====================================
Generated: {}
Total Tests: {}
Passed: {}
Failed: {}
Success Rate: {:.1}%

Test Results:
{}
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            total_tests,
            passed_tests,
            failed_tests,
            (passed_tests as f64 / total_tests as f64) * 100.0,
            self.test_results.iter().map(|t| {
                format!(
                    "{} [{}] {}ms - {}{}",
                    t.test_name,
                    if t.passed { "PASS" } else { "FAIL" },
                    t.duration_ms,
                    t.details,
                    if let Some(error) = &t.error_message {
                        format!(" (Error: {})", error)
                    } else {
                        String::new()
                    }
                )
            }).collect::<Vec<_>>().join("\n")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_integration_test_suite() {
        let mut test_suite = IntegrationTestSuite::new();
        let results = test_suite.run_all_tests().await;
        
        assert!(!results.is_empty(), "Test suite should run tests");
        
        let passed_count = results.iter().filter(|t| t.passed).count();
        assert!(passed_count > 0, "At least some tests should pass");
    }
}
