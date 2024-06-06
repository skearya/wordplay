mod global;
mod messages;
mod state;
mod utils;

use std::borrow::Cow;

use global::{GlobalData, GLOBAL};
use messages::ClientMessage;
use state::{AppState, SenderInfo};

use axum::{
    extract::{
        ws::{close_code, CloseFrame, Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    response::Response,
    routing::get,
    Json, Router,
};
use axum_extra::TypedHeader;
use futures::{stream::StreamExt, SinkExt, TryFutureExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tower_http::cors;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    GLOBAL.set(GlobalData::new()).unwrap();

    let app = Router::new()
        .route("/info", get(info))
        .route("/rooms/*room", get(ws_handler))
        .layer(cors::CorsLayer::new().allow_origin(cors::Any))
        .with_state(AppState::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize, Debug)]
pub struct Info {
    pub clients_connected: usize,
    pub public_rooms: Vec<RoomData>,
}

#[derive(Serialize, Debug)]
pub struct RoomData {
    pub name: String,
    pub players: usize,
}

async fn info(State(state): State<AppState>) -> Json<Info> {
    Json(state.info())
}

#[derive(Deserialize, Debug)]
pub struct Params {
    username: String,
    rejoin_token: Option<Uuid>,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room): Path<String>,
    Query(params): Query<Params>,
    State(state): State<AppState>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> Response {
    let user_agent = user_agent.map_or("unknown user agent".into(), |header| header.to_string());

    println!(
        "`{user_agent}` connected at {room}, {} total connections",
        state.info().clients_connected + 1
    );

    ws.on_upgrade(move |socket| handle_socket(socket, state, room, params))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, room: String, params: Params) {
    if params.username.len() > 20 {
        socket
            .send(Message::Close(Some(CloseFrame {
                code: close_code::ABNORMAL,
                reason: Cow::from("Username too long (max 20 characters)"),
            })))
            .await
            .ok();
    }

    let (mut sender, mut reciever) = socket.split();
    let (proxy, mut inbox) = mpsc::unbounded_channel::<Message>();

    let socket_uuid = Uuid::new_v4();
    let uuid = state.add_client(&room, params, socket_uuid, proxy);
    let info = SenderInfo { uuid, room: &room };

    let sending_task = tokio::spawn(async move {
        while let Some(msg) = inbox.recv().await {
            sender
                .send(msg.clone())
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {e}, msg: {msg:#?}");
                })
                .await;
        }
    });

    while let Some(Ok(msg)) = reciever.next().await {
        if let Message::Text(text) = msg {
            if text.len() > 500 {
                eprintln!("ignoring large message ({} bytes)", text.len());
                continue;
            }

            if let Ok(msg) = serde_json::from_str::<ClientMessage>(&text) {
                state.handle(info, msg);
            } else {
                eprintln!("couldnt parse message: {text}");
            }
        }
    }

    sending_task.abort();
    state
        .remove_client(info, socket_uuid)
        .unwrap_or_else(|e| eprintln!("failed to remove client: {e}"));
}
