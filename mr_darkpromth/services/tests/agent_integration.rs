use mr_darkpromth_services::learning_system::LearningSystem;
use mr_darkpromth_services::self_correction_agent::SelfCorrectionAgent;
use mr_darkpromth_services::error_detector::{ErrorDetection, ErrorCategory, ErrorSeverity, ErrorSource, ErrorContext};
use mr_darkpromth_services::test_db_utils::create_test_pool;
use cerebras_client::CerebrasClient;
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::test]
async fn test_learning_system_integration() {
    // 1. Setup
    // Ensure we are using the test environment DB
    let pool = create_test_pool().await.expect("Failed to connect to test DB");
    
    // Clean up previous test data
    sqlx::query("TRUNCATE TABLE correction_history, error_patterns CASCADE")
        .execute(&pool)
        .await
        .expect("Failed to truncate tables");

    let mut learning_system = LearningSystem::new(pool.clone());

    // 2. Simulate Error Detection
    let error_id = Uuid::new_v4();
    let error = ErrorDetection {
        id: error_id,
        category: ErrorCategory::SyntaxError,
        severity: ErrorSeverity::High,
        message: "Unexpected token '}' in user_service.rs".to_string(),
        source: ErrorSource::ApplicationLog,
        context: ErrorContext {
            file_path: Some("services/src/user_service.rs".to_string()),
            function_name: Some("update_user".to_string()),
            line_number: None,
            request_id: None,
            user_id: None,
            additional_metadata: HashMap::new(),
        },
        timestamp: chrono::Utc::now(),
        stack_trace: None,
    };

    // 3. Simulate Fix and Correction Record
    use mr_darkpromth_services::correction_validator::{CorrectionAttempt, CorrectionStatus};

    use mr_darkpromth_services::fix_generator::{GeneratedFix, RiskLevel};

    let fix_id = Uuid::new_v4();
    let fix = GeneratedFix {
        id: fix_id,
        error_id,
        description: "Remove extra closing brace".to_string(),
        code_changes: vec![],
        confidence_score: 0.95,
        reasoning: "Test reasoning".to_string(),
        estimated_risk: RiskLevel::Low,
        suggested_tests: vec!["cargo check".to_string()],
        timestamp: chrono::Utc::now(),
    };

    let attempt = CorrectionAttempt {
        id: Uuid::new_v4(),
        fix_id,
        error_id,
        status: CorrectionStatus::Validated,
        rollback_snapshot: None,
        test_results: vec![],
        validation_score: 1.0,
        timestamp: chrono::Utc::now(),
    };

    // 4. Record Success
    learning_system.record_correction(&attempt, &fix, &error).await.expect("Failed to record correction");

    // 5. Verify Database State for correction_history
    let history: (i64,) = sqlx::query_as("SELECT count(*) FROM correction_history")
        .fetch_one(&pool)
        .await
        .expect("Failed to count history");
    assert_eq!(history.0, 1, "Should have 1 history record");

    // 6. Verify Metrics (patterns are stored in-memory cache)
    let metrics = learning_system.get_metrics();
    assert!(metrics.total_corrections >= 1, "Should have at least 1 correction");
    assert!(metrics.successful_corrections >= 1, "Should have at least 1 success");
    
    // 7. Verify Insights
    let insights = learning_system.get_insights().await.expect("Failed to get insights");
    assert!(insights.contains("Success Rate: 100.0%"));
    assert!(insights.contains("SyntaxError"));
}

#[tokio::test]
async fn test_self_correction_agent_loop() {
    // 1. Setup
    let pool = create_test_pool().await.expect("Failed to connect to test DB");
    
    // We need a redis URL. Check env or use default test one
    let redis_url = std::env::var("REDIS_URL_TEST").unwrap_or_else(|_| "redis://localhost:6380".to_string());
    
    // Mock Cerebras Client (we can't make real API calls in integration test without key)
    // For this test, we might need to mock the internal components of the agent, 
    // but the Agent struct doesn't easily allow mocking fields without significant refactoring.
    // So we will instantiate it, but we won't call `start()` which might trigger loop.
    // Instead we will call specific methods.

    // Note: If CEREBRAS_API_KEY is not set, this might fail unless we mock the client or ensure config is optional.
    // CerebrasConfig::new requires a key.
    
    // To properly test SelfCorrectionAgent logic without hitting Cerebras API, 
    // we should rely on the internal `learning_system` which we verified above.
    // Testing the full agent loop requires mocking the `FixGenerator` which uses `CerebrasClient`.
    
    // Strategy: Test that the agent can be instantiated and connected to Redis/DB in this environment.
    
    let config_res = cerebras_client::CerebrasConfig::new(vec!["dummy_key".to_string()]);
    if let Ok(config) = config_res {
        let client = CerebrasClient::new(config);
        let project_root = std::env::current_dir().unwrap().to_string_lossy().to_string();
        
        let agent = SelfCorrectionAgent::new(
            &redis_url,
            client,
            pool.clone(),
            project_root,
            0.8
        );
        
        assert!(agent.is_ok(), "Agent failed to initialize");
        
        if let Ok(agent) = agent {
             // Test get_metrics which hits the DB
             let metrics = agent.get_metrics().await;
             assert!(metrics.is_ok());
             
             // Test reporting critical error (Redis)
             // We can't easily verify Redis message without a subscriber, but we verify it doesn't error
             // We need a dummy detection
             /*
             let detection = ErrorDetection {
                 id: Uuid::new_v4(),
                 category: ErrorCategory::DatabaseError,
                 severity: ErrorSeverity::Critical,
                 message: "DB Connection Lost".to_string(),
                 source: ErrorSource::SystemMonitor,
                 context: ErrorContext::default(),
                 timestamp: chrono::Utc::now(),
                 stack_trace: None,
             };
             // notify_critical_error is private.
             */
        }
    }
}
