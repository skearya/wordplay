pub mod game;
mod global;
pub mod messages;
mod models;

use global::{GlobalData, GLOBAL};
use messages::{ClientMessage, ServerMessage};
use models::{AppState, Client};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_extra::TypedHeader;
use futures::{stream::StreamExt, SinkExt, TryFutureExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use uuid::Uuid;

#[tokio::main]
async fn main() {
    GLOBAL.set(GlobalData::new()).unwrap();

    dotenvy::dotenv().ok();
    console_subscriber::init();

    let app = Router::new()
        .route("/rooms/*room", get(ws_handler))
        .with_state(AppState::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize)]
struct Params {
    username: String,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room): Path<String>,
    Query(params): Query<Params>,
    State(state): State<AppState>,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    let user_agent = user_agent.map_or("unknown user agent".into(), |header| header.to_string());

    println!(
        "`{user_agent}` connected at {room}, {} total connections",
        state.clients_connected() + 1
    );

    ws.on_upgrade(move |socket| handle_socket(socket, state, room, params.username))
}

async fn handle_socket(socket: WebSocket, state: AppState, room: String, username: String) {
    let uuid = Uuid::new_v4();

    let (mut sender, mut reciever) = socket.split();
    let (proxy, mut inbox) = mpsc::unbounded_channel::<ServerMessage>();

    state.add_client(
        room.clone(),
        Client {
            uuid,
            tx: proxy,
            username: username.clone(),
        },
    );

    let sending_task = tokio::task::spawn(async move {
        while let Some(msg) = inbox.recv().await {
            let serialized = serde_json::to_string(&msg).unwrap();

            sender
                .send(serialized.clone().into())
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {e}, msg: {serialized:#?}");
                })
                .await;
        }
    });

    while let Some(msg) = reciever.next().await {
        if let Ok(msg) = msg {
            if let Message::Text(text) = msg {
                if let Ok(msg) = serde_json::from_str::<ClientMessage>(&text) {
                    msg.handle(&state, &room, &username, uuid);
                } else {
                    eprintln!("couldnt parse message: {text}");
                }
            }
        } else {
            break;
        }
    }

    sending_task.abort();
    state.remove_client(&room, uuid);
}
