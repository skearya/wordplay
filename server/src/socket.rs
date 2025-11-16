use axum::{
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures::{SinkExt, StreamExt};
use rustrict::CensorStr;
use serde::Deserialize;
use tokio::sync::{mpsc, oneshot};
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    room::{
        Room,
        clients::{Client, SocketRef},
        messages::CoreMessage,
    },
    state::AppState,
    task,
};

#[derive(Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SocketParams {
    pub username: String,
    pub rejoin_token: Option<Uuid>,
}

pub async fn handler(
    State(state): State<AppState>,
    Path(room_name): Path<String>,
    Query(params): Query<SocketParams>,
    ws: WebSocketUpgrade,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let error = match () {
        () if params.username.is_empty() => Some("username cannot be empty"),
        () if params.username.len() > 20 => Some("username too long (max 20 characters)"),
        () if room_name.len() > 6 => Some("invalid room name, must be less than 6 characters"),
        () if !room_name.chars().all(|c| c.is_ascii_alphanumeric()) => {
            Some("invalid room name, must be alphanumeric")
        }
        () if params.username.is_inappropriate() => {
            Some("username likely contains innappropriate content")
        }
        () if room_name.is_inappropriate() => {
            Some("room name likely contains innappropriate content")
        }
        () => None,
    };

    if let Some(error) = error {
        Err((StatusCode::BAD_REQUEST, error))
    } else {
        let upgrade = ws.max_message_size(512).on_upgrade(|ws| async {
            if let Err(err) = socket(state, room_name, params, ws).await {
                tracing::error!(?err);
            }
        });

        Ok(upgrade)
    }
}

async fn socket(
    state: AppState,
    room_name: String,
    SocketParams {
        username,
        rejoin_token,
    }: SocketParams,
    socket: WebSocket,
) -> anyhow::Result<()> {
    let (mut sink, mut stream) = socket.split();
    let (sender, mut reciever) = mpsc::unbounded_channel::<ws::Message>();

    // Room message -> WebSocket Sink.
    task::spawn(async move {
        while let Some(message) = reciever.recv().await {
            sink.send(message).await?;
        }

        Ok(())
    });

    // Random UUID for this socket, not client.
    let socket = Uuid::new_v4();

    let room = if let Some(room) = state.get_room(&room_name) {
        room
    } else {
        let room = Room::spawn();
        state.insert_room(room_name, room.clone());

        room
    };

    let (response, uuid) = oneshot::channel();

    room.send(CoreMessage::Join {
        response,
        rejoin_token,
        client: Client::new(SocketRef::new(socket, sender), username),
    });

    // Client UUID returned by the room.
    let uuid = uuid.await?;

    // WebSocket Stream -> Room message.
    task::spawn(async move {
        while let Some(message) = stream.next().await {
            if let Ok(ws::Message::Text(bytes)) = message {
                if let Ok(message) = serde_json::from_str(bytes.as_str()) {
                    room.send(CoreMessage::Client { uuid, message });
                } else {
                    tracing::error!("failed deserializing: {}", bytes.as_str());
                }
            }
        }

        room.send(CoreMessage::Leave { uuid, socket });

        Ok(())
    });

    Ok(())
}
