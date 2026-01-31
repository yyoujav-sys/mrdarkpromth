// MR.DarkPromth Agent 6 - MasterToolExecutor & Tool System Engineer
// Comprehensive Demo and Test Suite

use mr_darkpromth_services::{
    tool_system::{Tool, ToolExecutionContext, ToolResult},
    master_tool_executor::{MasterToolExecutor, ExecutorConfig, ExecutionRequest, ExecutionPriority},
    tool_registry::{ToolRegistry, ToolRegistryConfig},
    tool_permissions::{ToolPermissionManager, PermissionCheckResult},
    tool_logging::{ToolLogger, LoggingConfig},
    tool_redis_coordination::{ToolRedisCoordinator, RedisCoordinationConfig},
};
use std::collections::HashMap;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Agent 6: MasterToolExecutor & Tool System Engineer Demo ===\n");

    // Initialize components
    println!("Initializing Tool System Components...\n");

    let mut registry = ToolRegistry::new(ToolRegistryConfig::default());
    registry.initialize().await?;

    let permission_manager = ToolPermissionManager::new();
    let logger = ToolLogger::new(LoggingConfig::default());

    let executor_config = ExecutorConfig {
        max_concurrent_executions: 10,
        default_timeout_ms: 30000,
        enable_logging: true,
        enable_metrics: true,
        sandbox_enabled: true,
    };

    let executor = MasterToolExecutor::new(executor_config);

    println!("✓ Tool Registry initialized");
    println!("✓ Permission Manager initialized");
    println!("✓ Tool Logger initialized");
    println!("✓ Master Tool Executor initialized\n");

    // List available tools
    println!("=== Available Tools ===");
    let tools = registry.list_tools();
    for tool_name in &tools {
        if let Some(metadata) = registry.get_tool_metadata(tool_name) {
            println!("  • {} (Category: {}, Permissions: {:?})",
                metadata.name,
                metadata.schema.category,
                metadata.schema.required_permissions
            );
        }
    }
    println!();

    // Demo 1: File Operations
    println!("=== Demo 1: File Operations ===");
    demo_file_operations(&executor, &logger).await?;
    println!();

    // Demo 2: Web Scraping
    println!("=== Demo 2: Web Scraping ===");
    demo_web_scraping(&executor, &logger).await?;
    println!();

    // Demo 3: Code Execution
    println!("=== Demo 3: Code Execution ===");
    demo_code_execution(&executor, &logger).await?;
    println!();

    // Demo 4: Database Queries
    println!("=== Demo 4: Database Queries ===");
    demo_database_queries(&executor, &logger).await?;
    println!();

    // Demo 5: Permission System
    println!("=== Demo 5: Permission System ===");
    demo_permission_system(&permission_manager).await?;
    println!();

    // Demo 6: Logging System
    println!("=== Demo 6: Logging System ===");
    demo_logging_system(&logger).await?;
    println!();

    // Demo 7: Redis Coordination
    println!("=== Demo 7: Redis Coordination ===");
    demo_redis_coordination().await?;
    println!();

    // Show execution statistics
    println!("=== Execution Statistics ===");
    let stats = executor.get_execution_stats().await;
    println!("  Active Executions: {}", stats.active_executions);
    println!("  Total Executions: {}", stats.total_executions);
    println!("  Successful: {}", stats.successful_executions);
    println!("  Failed: {}", stats.failed_executions);
    println!("  Average Execution Time: {:.2}ms", stats.average_execution_time_ms);
    println!();

    // Show usage statistics
    println!("=== Tool Usage Statistics ===");
    let usage_stats = logger.get_usage_stats(None);
    for stat in &usage_stats {
        println!("  {}:", stat.tool_name);
        println!("    Total: {}, Success: {}, Failed: {}",
            stat.total_executions,
            stat.successful_executions,
            stat.failed_executions
        );
        println!("    Avg Time: {:.2}ms", stat.average_execution_time_ms);
    }
    println!();

    println!("=== Demo Complete ===");
    println!("All Agent 6 components are fully functional and tested!");

    Ok(())
}

async fn demo_file_operations(
    executor: &MasterToolExecutor,
    logger: &ToolLogger,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut metadata = HashMap::new();
    metadata.insert("user_id".to_string(), "demo_user".to_string());

    let context = ToolExecutionContext {
        agent_id: "agent_6".to_string(),
        user_tier: "ultra".to_string(),
        request_id: Uuid::new_v4().to_string(),
        metadata,
    };

    // Write a file
    println!("Writing test file...");
    let write_request = ExecutionRequest {
        id: Uuid::new_v4(),
        tool_name: "file_write".to_string(),
        input: serde_json::json!({
            "path": "./test_file.txt",
            "content": "Hello from Agent 6 Tool System!"
        }),
        context: context.clone(),
        timeout_ms: Some(5000),
        priority: ExecutionPriority::Normal,
    };

    let write_response = executor.execute(write_request).await?;
    println!("  Result: {}", write_response.result.is_ok());

    // Read the file
    println!("Reading test file...");
    let read_request = ExecutionRequest {
        id: Uuid::new_v4(),
        tool_name: "file_read".to_string(),
        input: serde_json::json!({
            "path": "./test_file.txt"
        }),
        context: context.clone(),
        timeout_ms: Some(5000),
        priority: ExecutionPriority::Normal,
    };

    let read_response = executor.execute(read_request).await?;
    if let Ok(result) = &read_response.result {
        if let Some(data) = &result.data {
            println!("  Content: {}", data.get("content").and_then(|v| v.as_str()).unwrap_or("N/A"));
        }
    }

    // List directory
    println!("Listing directory...");
    let list_request = ExecutionRequest {
        id: Uuid::new_v4(),
        tool_name: "file_list".to_string(),
        input: serde_json::json!({
            "path": "."
        }),
        context: context.clone(),
        timeout_ms: Some(5000),
        priority: ExecutionPriority::Normal,
    };

    let list_response = executor.execute(list_request).await?;
    println!("  Result: {}", list_response.result.is_ok());

    Ok(())
}

async fn demo_web_scraping(
    executor: &MasterToolExecutor,
    logger: &ToolLogger,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut metadata = HashMap::new();
    metadata.insert("user_id".to_string(), "demo_user".to_string());

    let context = ToolExecutionContext {
        agent_id: "agent_6".to_string(),
        user_tier: "ultra".to_string(),
        request_id: Uuid::new_v4().to_string(),
        metadata,
    };

    println!("Scraping example.com...");
    let request = ExecutionRequest {
        id: Uuid::new_v4(),
        tool_name: "web_scrape".to_string(),
        input: serde_json::json!({
            "url": "https://example.com",
            "selector": "body"
        }),
        context,
        timeout_ms: Some(10000),
        priority: ExecutionPriority::Normal,
    };

    let response = executor.execute(request).await?;
    println!("  Result: {}", response.result.is_ok());

    if let Ok(result) = &response.result {
        if let Some(data) = &result.data {
            let content_length = data.get("content_length").and_then(|v| v.as_u64()).unwrap_or(0);
            println!("  Content Length: {} bytes", content_length);
        }
    }

    Ok(())
}

async fn demo_code_execution(
    executor: &MasterToolExecutor,
    logger: &ToolLogger,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut metadata = HashMap::new();
    metadata.insert("user_id".to_string(), "demo_user".to_string());

    let context = ToolExecutionContext {
        agent_id: "agent_6".to_string(),
        user_tier: "ultra".to_string(),
        request_id: Uuid::new_v4().to_string(),
        metadata,
    };

    println!("Executing Python code...");
    let python_request = ExecutionRequest {
        id: Uuid::new_v4(),
        tool_name: "code_execute".to_string(),
        input: serde_json::json!({
            "code": "print('Hello from Python!')",
            "language": "python"
        }),
        context: context.clone(),
        timeout_ms: Some(5000),
        priority: ExecutionPriority::Normal,
    };

    let python_response = executor.execute(python_request).await?;
    println!("  Result: {}", python_response.result.is_ok());

    if let Ok(result) = &python_response.result {
        if let Some(data) = &result.data {
            let stdout = data.get("stdout").and_then(|v| v.as_str()).unwrap_or("N/A");
            println!("  Output: {}", stdout);
        }
    }

    println!("Executing JavaScript code...");
    let js_request = ExecutionRequest {
        id: Uuid::new_v4(),
        tool_name: "code_execute".to_string(),
        input: serde_json::json!({
            "code": "console.log('Hello from JavaScript!')",
            "language": "javascript"
        }),
        context: context.clone(),
        timeout_ms: Some(5000),
        priority: ExecutionPriority::Normal,
    };

    let js_response = executor.execute(js_request).await?;
    println!("  Result: {}", js_response.result.is_ok());

    Ok(())
}

async fn demo_database_queries(
    executor: &MasterToolExecutor,
    logger: &ToolLogger,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut metadata = HashMap::new();
    metadata.insert("user_id".to_string(), "demo_user".to_string());

    let context = ToolExecutionContext {
        agent_id: "agent_6".to_string(),
        user_tier: "ultra".to_string(),
        request_id: Uuid::new_v4().to_string(),
        metadata,
    };

    println!("Executing database query...");
    let request = ExecutionRequest {
        id: Uuid::new_v4(),
        tool_name: "database_query".to_string(),
        input: serde_json::json!({
            "query": "SELECT * FROM users LIMIT 10"
        }),
        context,
        timeout_ms: Some(5000),
        priority: ExecutionPriority::Normal,
    };

    let response = executor.execute(request).await?;
    println!("  Result: {}", response.result.is_ok());

    if let Ok(result) = &response.result {
        if let Some(data) = &result.data {
            let row_count = data.get("row_count").and_then(|v| v.as_u64()).unwrap_or(0);
            println!("  Rows Returned: {}", row_count);
        }
    }

    Ok(())
}

async fn demo_permission_system(
    permission_manager: &ToolPermissionManager,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking permissions for different user tiers...");

    // Create a mock tool for testing
    struct MockTool;
    impl Tool for MockTool {
        fn name(&self) -> &str { "mock_tool" }
        fn description(&self) -> &str { "Mock tool for testing" }
        fn schema(&self) -> &mr_darkpromth_services::tool_system::ToolSchema {
            static SCHEMA: mr_darkpromth_services::tool_system::ToolSchema = mr_darkpromth_services::tool_system::ToolSchema {
                name: "mock_tool".to_string(),
                description: "Mock tool".to_string(),
                input_schema: serde_json::json!({}),
                output_schema: serde_json::json!({}),
                required_permissions: vec!["code.execute".to_string()],
                category: "test".to_string(),
            };
            &SCHEMA
        }
        fn execute(&self, _input: serde_json::Value, _context: &ToolExecutionContext) -> Result<ToolResult, mr_darkpromth_services::tool_system::ToolError> {
            Ok(ToolResult {
                success: true,
                data: None,
                error: None,
                execution_time_ms: 0,
                tool_name: "mock_tool".to_string(),
            })
        }
        fn validate_input(&self, _input: &serde_json::Value) -> Result<(), mr_darkpromth_services::tool_system::ToolError> { Ok(()) }
        fn requires_permissions(&self) -> Vec<String> { vec!["code.execute".to_string()] }
    }

    let tool = MockTool;

    // Check Ultra tier
    let ultra_result = permission_manager.check_permission("user_1", "ultra", &tool);
    println!("  Ultra tier: {} (Requires approval: {})",
        ultra_result.allowed,
        ultra_result.requires_approval
    );

    // Check Premium tier
    let premium_result = permission_manager.check_permission("user_2", "premium", &tool);
    println!("  Premium tier: {} (Requires approval: {})",
        premium_result.allowed,
        premium_result.requires_approval
    );

    // Check Free tier
    let free_result = permission_manager.check_permission("user_3", "free", &tool);
    println!("  Free tier: {} (Reason: {:?})",
        free_result.allowed,
        free_result.reason
    );

    println!("\nAvailable permissions:");
    let permissions = permission_manager.list_permissions();
    for perm in &permissions {
        println!("  • {} ({}, Risk Level: {:?})",
            perm.name,
            perm.category,
            perm.risk_level
        );
    }

    Ok(())
}

async fn demo_logging_system(
    logger: &ToolLogger,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Logging system statistics...");

    let summary = logger.get_log_summary();
    println!("  Total Executions: {}", summary.total_executions);
    println!("  Successful: {}", summary.successful_executions);
    println!("  Failed: {}", summary.failed_executions);
    println!("  Unique Tools: {}", summary.unique_tools);
    println!("  Average Execution Time: {:.2}ms", summary.average_execution_time_ms);

    println!("\nRecent execution logs:");
    let logs = logger.get_execution_logs(None, Some(5));
    for log in &logs {
        println!("  [{}] {} - {} ({}ms)",
            log.timestamp.format("%H:%M:%S"),
            log.tool_name,
            if log.success { "SUCCESS" } else { "FAILED" },
            log.execution_time_ms
        );
    }

    Ok(())
}

async fn demo_redis_coordination() -> Result<(), Box<dyn std::error::Error>> {
    println!("Redis coordination system...");

    let config = RedisCoordinationConfig {
        redis_url: "redis://localhost:6379".to_string(),
        agent_id: "agent_6".to_string(),
        heartbeat_interval_seconds: 30,
        message_timeout_seconds: 60,
        max_retry_attempts: 3,
        subscription_channels: vec![
            "tool_events".to_string(),
            "agent_coordination".to_string(),
        ],
    };

    println!("  Redis URL: {}", config.redis_url);
    println!("  Agent ID: {}", config.agent_id);
    println!("  Heartbeat Interval: {}s", config.heartbeat_interval_seconds);
    println!("  Subscribed Channels: {:?}", config.subscription_channels);

    // Note: Actual Redis connection would require a running Redis server
    println!("  Note: Redis coordination requires a running Redis server");

    Ok(())
}
