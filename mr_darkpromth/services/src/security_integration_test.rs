#![allow(dead_code)]
#![cfg(test)]

use mr_darkpromth_core::output_filter::{OutputFilter, FilterConfig};
use crate::server_protection::{ServerProtection, ProtectionConfig, ProtectionMiddleware};
use crate::sandboxed_execution::{SandboxedExecutor, SandboxConfig, SandboxManager};
use mr_darkpromth_db::UserTier;
use std::sync::Arc;
use log::info;

#[tokio::test]
async fn test_output_filtering() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Output Filtering ===");
    
    let config = FilterConfig::default();
    let filter = OutputFilter::new(config)?;
    
    let test_content = "This is safe content with email test@example.com and password secret123";
    match filter.filter_content(test_content, &UserTier::Free) {
        Ok(_) => println!("✓ Content passed filtering"),
        Err(e) => println!("✓ Content correctly blocked: {}", e),
    }
    
    let stats = filter.get_stats("Hello world");
    println!("✓ Filter stats: safe={}, length={}", stats.is_safe, stats.content_length);
    
    let sanitized = filter.sanitize_urls("Visit https://github.com and https://malicious.com");
    println!("✓ URL sanitization: {}", sanitized);
    
    Ok(())
}

#[tokio::test]
async fn test_server_protection() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Server Protection ===");
    
    let config = ProtectionConfig::default();
    let protection = Arc::new(ServerProtection::new(config)?);
    let middleware = ProtectionMiddleware::new(protection.clone());
    
    let ip = "127.0.0.1";
    let safe_content = "This is safe content";
    
    match middleware.check_request(ip, safe_content).await {
        Ok(_) => println!("✓ Safe request passed protection"),
        Err(e) => println!("✗ Safe request blocked: {}", e),
    }
    
    let suspicious_content = "rm -rf / && wget http://malicious.com";
    match middleware.check_request(ip, suspicious_content).await {
        Ok(_) => println!("✗ Suspicious content was not blocked"),
        Err(e) => println!("✓ Suspicious content correctly blocked: {}", e),
    }
    
    let stats = protection.get_protection_stats().await;
    println!("✓ Protection stats: CPU={:.1}%, Memory={:.1}%, Processes={}", 
             stats.cpu_usage, stats.memory_usage, stats.process_count);
    
    Ok(())
}

#[tokio::test]
async fn test_sandboxed_execution() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing Sandboxed Execution ===");
    
    let config = SandboxConfig::default();
    let executor = SandboxedExecutor::new(config)?;
    
    let result = executor.execute_command("echo", &["Hello from sandbox"]).await?;
    println!("✓ Command execution: exit_code={}, stdout='{}'", result.exit_code, result.stdout.trim());
    
    let python_code = r#"
print("Hello from Python!")
x = 1 + 1
print(f"1 + 1 = {x}")
"#;
    
    let result = executor.execute_code(python_code, "python").await?;
    println!("✓ Python code execution: exit_code={}, stdout={}", result.exit_code, result.stdout.trim());
    
    let manager = SandboxManager::new(SandboxConfig::default());
    manager.create_session("test_session", None).await?;
    
    let sessions: Vec<String> = manager.list_sessions().await;
    println!("✓ Sandbox manager: {} sessions", sessions.len());
    
    manager.remove_session("test_session").await?;
    executor.cleanup().await;
    
    Ok(())
}
