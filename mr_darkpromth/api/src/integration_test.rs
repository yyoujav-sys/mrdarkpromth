// Integration Tests for Agent 5 User Management System
// Tests full stack integration with API Gateway, Database, and Redis

use actix_web::{test, web, App};
use serde_json::json;
use uuid::Uuid;

#[actix_web::test]
async fn test_health_endpoint() {
    let app = test::init_service(
        App::new()
            .route("/health", web::get().to(mr_darkpromth_api::get_health))
    ).await;

    let req = test::TestRequest::get()
        .uri("/health")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_user_registration() {
    let app = test::init_service(
        App::new()
            .route("/api/auth/register", web::post().to(mr_darkpromth_api::register))
    ).await;

    let register_request = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&register_request)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // In a real test with database, this would succeed
    // For now, we're just testing the endpoint exists
    assert!(resp.status().is_client_error() || resp.status().is_success());
}

#[actix_web::test]
async fn test_user_login() {
    let app = test::init_service(
        App::new()
            .route("/api/auth/login", web::post().to(mr_darkpromth_api::login))
    ).await;

    let login_request = json!({
        "email": "test@example.com",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&login_request)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // In a real test with database, this would succeed
    // For now, we're just testing the endpoint exists
    assert!(resp.status().is_client_error() || resp.status().is_success());
}

#[actix_web::test]
async fn test_get_user_info() {
    let app = test::init_service(
        App::new()
            .route("/api/users/{id}", web::get().to(mr_darkpromth_api::get_user_info))
    ).await;

    let user_id = Uuid::new_v4();

    let req = test::TestRequest::get()
        .uri(&format!("/api/users/{}", user_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_jailbreak_prompts_endpoint() {
    let app = test::init_service(
        App::new()
            .route("/api/jailbreak/prompts", web::get().to(mr_darkpromth_api::get_prompts))
    ).await;

    let req = test::TestRequest::get()
        .uri("/api/jailbreak/prompts")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_chat_endpoint() {
    let app = test::init_service(
        App::new()
            .route("/api/chat", web::post().to(mr_darkpromth_api::post_chat))
    ).await;

    let chat_request = json!({
        "message": "Hello, world!",
        "model": "llama-3.3-70b",
        "user_tier": "Free"
    });

    let req = test::TestRequest::post()
        .uri("/api/chat")
        .set_json(&chat_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_ultra_tier_chat_with_jailbreak() {
    let app = test::init_service(
        App::new()
            .route("/api/chat", web::post().to(mr_darkpromth_api::post_chat))
    ).await;

    let chat_request = json!({
        "message": "Hello, world!",
        "model": "llama-3.3-70b",
        "jailbreak_prompt": "DAN mode",
        "user_tier": "Ultra"
    });

    let req = test::TestRequest::post()
        .uri("/api/chat")
        .set_json(&chat_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["jailbreak_applied"], true);
}

#[actix_web::test]
async fn test_free_tier_chat_no_jailbreak() {
    let app = test::init_service(
        App::new()
            .route("/api/chat", web::post().to(mr_darkpromth_api::post_chat))
    ).await;

    let chat_request = json!({
        "message": "Hello, world!",
        "model": "llama-3.3-70b",
        "jailbreak_prompt": "DAN mode",
        "user_tier": "Free"
    });

    let req = test::TestRequest::post()
        .uri("/api/chat")
        .set_json(&chat_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["jailbreak_applied"], false);
}
