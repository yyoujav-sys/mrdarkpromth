use actix::{Actor, StreamHandler, ActorContext, AsyncContext};
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::time::{Duration, Instant};
use uuid::Uuid;
use log::info;
use chrono;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ChatMessage {
    pub id: Uuid,
    pub content: String,
    pub sender: String,
    pub timestamp: String,
}

pub struct WebSocketChat {
    hb: Instant,
}

impl Actor for WebSocketChat {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketChat {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                self.handle_text(text.to_string(), ctx);
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

impl WebSocketChat {
    fn handle_text(&mut self, msg: String, ctx: &mut ws::WebsocketContext<Self>) {
        info!("Received message: {}", msg);

        let response = ChatMessage {
            id: Uuid::new_v4(),
            content: msg,
            sender: "user".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        if let Ok(json) = serde_json::to_string(&response) {
            ctx.text(json);
        }
    }

    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(Duration::from_secs(5), |act: &mut WebSocketChat, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(10) {
                info!("Websocket client disconnected due to heartbeat failure");
                ctx.stop();
            }
        });
    }
}

#[utoipa::path(
    get,
    path = "/ws/chat",
    responses(
        (status = 101, description = "WebSocket connection established")
    ),
    tag = "websocket"
)]
pub async fn ws_chat_route(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    let ws_chat = WebSocketChat {
        hb: Instant::now(),
    };
    
    let resp = ws::start(ws_chat, &req, stream)?;
    Ok(resp)
}
