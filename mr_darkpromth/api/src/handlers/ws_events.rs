// MR.DarkPromth WebSocket Events Handler
// User-facing WebSocket endpoint for real-time event streaming

use axum::{
    extract::{State, ws::{WebSocketUpgrade, WebSocket, Message}},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::handlers::auth::extract_token;
use crate::error_handler::ApiError;
use crate::event_hub::{WsEvent, SessionInfo};

/// WebSocket upgrade handler for real-time events
pub async fn ws_events_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Authenticate before upgrading
    let token = match extract_token(&headers) {
        Some(t) => t,
        None => {
            return ApiError::new("MISSING_TOKEN", "Authorization header required").into_response();
        }
    };

    let claims = match state.user_service.validate_token(&token).await {
        Ok(c) => c,
        Err(_) => {
            return ApiError::new("INVALID_TOKEN", "Invalid or expired token").into_response();
        }
    };

    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => {
            return ApiError::new("INVALID_USER_ID", "Invalid user ID in token").into_response();
        }
    };

    let client_type = headers.get("x-client-type")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("website")
        .to_string();

    let username = claims.username.clone();

    // Check if we need to negotiate subprotocol for browser clients
    // Browser sends: Sec-WebSocket-Protocol: access_token, <token>
    // Server must respond with: Sec-WebSocket-Protocol: access_token
    let mut ws = ws;
    if let Some(proto) = headers.get("sec-websocket-protocol").and_then(|h| h.to_str().ok()) {
        if proto.contains("access_token") {
            ws = ws.protocols(["access_token"]);
        }
    }

    ws.on_upgrade(move |socket| {
        handle_ws_events(socket, state, user_id, client_type, username)
    })
}

async fn handle_ws_events(
    socket: WebSocket,
    state: Arc<AppState>,
    user_id: Uuid,
    client_type: String,
    username: String,
) {
    let (sender, mut receiver) = socket.split();
    let sender = Arc::new(tokio::sync::Mutex::new(sender));
    let repo = mr_darkpromth_db::UserRepository::new(state.pool.clone());

    // 1. Register this WS connection as an active session
    let ws_session_id = format!("ws-events-{}-{}", user_id, client_type);
    let _ = repo.upsert_active_session(user_id, &client_type, None, None).await;
    let _ = repo.update_user_online_status(user_id, true, Some(&client_type)).await;

    // 2. Send initial SessionSync to the connecting client
    let sessions = get_user_sessions(&repo, user_id).await;
    let welcome = WsEvent::SessionSync {
        active_sessions: sessions,
        is_online: true,
    };
    let _ = sender.lock().await.send(Message::Text(
        serde_json::to_string(&welcome).unwrap_or_default().into()
    )).await;

    // 3. Notify other connected clients about this new session
    state.event_hub.send_to_user(user_id, WsEvent::UserStatusChanged {
        user_id: user_id.to_string(),
        username: username.clone(),
        is_online: true,
        client_type: client_type.clone(),
    });

    // 4. Subscribe to event hub
    let mut event_rx = state.event_hub.subscribe();

    // 5. Spawn heartbeat task
    let heartbeat_sender = Arc::clone(&sender);
    let heartbeat_handle = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let hb = WsEvent::Heartbeat {
                timestamp: chrono::Utc::now().to_rfc3339(),
                server_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            };
            if heartbeat_sender.lock().await.send(Message::Text(
                serde_json::to_string(&hb).unwrap_or_default().into()
            )).await.is_err() {
                break;
            }
        }
    });

    // 6. Main event loop — forward hub events to this client, handle incoming messages
    loop {
        tokio::select! {
            // Receive events from the hub and forward to client
            event = event_rx.recv() => {
                match event {
                    Ok(targeted) => {
                        // Filter: only send events targeted to this user, or broadcasts
                        let should_send = match targeted.target_user_id {
                            Some(target) => target == user_id,
                            None => true, // broadcast
                        };
                        if should_send {
                            let msg = serde_json::to_string(&targeted.event).unwrap_or_default();
                            if sender.lock().await.send(Message::Text(msg.into())).await.is_err() {
                                break; // client disconnected
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        log::warn!("WS event hub lagged by {} messages for user {}", n, user_id);
                    }
                    Err(_) => break, // channel closed
                }
            }
            // Handle incoming messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Client can send ping/heartbeat
                        if text.contains("ping") {
                            let pong = serde_json::json!({"type": "pong", "timestamp": chrono::Utc::now().to_rfc3339()});
                            let _ = sender.lock().await.send(Message::Text(pong.to_string().into())).await;
                        }
                        // Heartbeat the active session
                        let _ = repo.heartbeat_active_session(user_id, &client_type).await;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sender.lock().await.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }
        }
    }

    // 7. Cleanup on disconnect
    heartbeat_handle.abort();
    let _ = repo.delete_active_session_by_type(user_id, &client_type).await;

    // Check if user still has other active sessions
    let remaining = repo.count_user_active_sessions(user_id).await.unwrap_or(0);
    if remaining == 0 {
        let _ = repo.update_user_online_status(user_id, false, Some(&client_type)).await;
    }

    // Notify other clients about disconnect
    state.event_hub.send_to_user(user_id, WsEvent::SessionSync {
        active_sessions: get_user_sessions(&repo, user_id).await,
        is_online: remaining > 0,
    });

    log::info!("WS events disconnected: user={}, client={}, remaining_sessions={}", user_id, client_type, remaining);
}

/// Helper to fetch current active sessions for a user
async fn get_user_sessions(repo: &mr_darkpromth_db::UserRepository, user_id: Uuid) -> Vec<SessionInfo> {
    match sqlx::query_as::<_, (String, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
        "SELECT client_type, last_activity, created_at FROM user_active_sessions WHERE user_id = $1 ORDER BY last_activity DESC"
    )
    .bind(user_id)
    .fetch_all(repo.get_pool())
    .await {
        Ok(rows) => rows.into_iter().map(|(ct, la, ca)| SessionInfo {
            client_type: ct,
            last_activity: la.to_rfc3339(),
            created_at: ca.to_rfc3339(),
        }).collect(),
        Err(_) => Vec::new(),
    }
}
