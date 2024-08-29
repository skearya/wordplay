use crate::{
    db,
    state::{messages::ClientMessage, SenderInfo},
    AppState,
};
use axum::{
    extract::{
        ws::{close_code, CloseFrame, Message, WebSocket},
        Path, Query, State, WebSocketUpgrade,
    },
    response::Response,
};
use axum_extra::extract::CookieJar;
use futures::{stream::StreamExt, SinkExt};
use serde::Deserialize;
use std::borrow::Cow;
use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct Params {
    pub username: String,
    pub rejoin_token: Option<Uuid>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    jar: CookieJar,
    State(state): State<AppState>,
    Path(room): Path<String>,
    Query(params): Query<Params>,
) -> Response {
    let user = match jar.get("session") {
        Some(id) => db::get_user_from_session(&state.db, id.value()).await.ok(),
        None => None,
    };

    println!(
        "'{}' trying to connect to '{room}' | user: {user:?}",
        params.username,
    );

    let error = if params.username.len() > 12 {
        Some("username too long (max 12 characters)")
    } else if params.username.is_empty() {
        Some("username cannot be empty")
    } else if room.len() > 6 {
        Some("invalid room name, must be less than 6 characters")
    } else if !room.chars().all(|c| c.is_ascii_alphanumeric()) {
        Some("invalid room name, must be alphanumeric")
    } else if state.room_full(&room) {
        Some("room full")
    } else {
        None
    };

    if let Some(reason) = error {
        ws.on_upgrade(move |socket| send_error(socket, reason))
    } else {
        ws.on_upgrade(move |socket| handle_socket(socket, state, room, params))
    }
}

async fn send_error(mut socket: WebSocket, reason: &'static str) {
    socket
        .send(Message::Close(Some(CloseFrame {
            code: close_code::ERROR,
            reason: Cow::Borrowed(reason),
        })))
        .await
        .ok();

    socket.close().await.ok();
}

async fn handle_socket(socket: WebSocket, state: AppState, room: String, params: Params) {
    let (mut sender, mut reciever) = socket.split();
    let (proxy, mut inbox) = mpsc::unbounded_channel::<Message>();

    let socket_uuid = Uuid::new_v4();
    let uuid = state.add_client(&room, params, socket_uuid, proxy);
    let info = SenderInfo { uuid, room: &room };

    let sending_task = tokio::spawn(async move {
        while let Some(msg) = inbox.recv().await {
            sender.send(msg).await.unwrap_or_else(|e| {
                eprintln!("ws send error for {}: {e}", info.uuid);
            });
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
