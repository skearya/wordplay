use axum::{
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::{
    mpsc::{self},
    oneshot,
};
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    room::{clients::SocketRef, messages::CoreMessage},
    state::AppState,
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
) -> Response {
    ws.max_message_size(512).on_upgrade(|ws| async {
        if let Err(err) = socket(state, room_name, params, ws).await {
            tracing::error!(?err);
        }
    })
}

async fn socket(
    state: AppState,
    room_name: String,
    params: SocketParams,
    socket: WebSocket,
) -> anyhow::Result<()> {
    let (mut sink, mut stream) = socket.split();
    let (sender, mut reciever) = mpsc::unbounded_channel::<ws::Message>();

    // Random UUID for this socket, not client.
    let socket_uuid = Uuid::new_v4();
    let socket = SocketRef::new(socket_uuid, sender);

    // Get room or try creating it if it doesn't exist.
    let room = match state.get_or_insert_room(room_name) {
        Ok(room) => room,
        Err(reason) => {
            socket.close(reason);
            return Ok(());
        }
    };

    // Room message -> WebSocket Sink.
    tokio::spawn(async move {
        while let Some(message) = reciever.recv().await {
            sink.send(message).await?;
        }

        anyhow::Ok(())
    });

    let uuid = {
        // Send a message to the room, requesting to join it.
        let (sender, reciever) = oneshot::channel();

        room.send(CoreMessage::Join {
            socket,
            params,
            response: sender,
        });

        // Client UUID returned by the room. `None` if join error.
        if let Some(uuid) = reciever.await? {
            uuid
        } else {
            return Ok(());
        }
    };

    // WebSocket Stream -> Room message.
    tokio::spawn(async move {
        while let Some(message) = stream.next().await {
            if let Ok(ws::Message::Text(bytes)) = message {
                if let Ok(message) = serde_json::from_str(bytes.as_str()) {
                    room.send(CoreMessage::Client { uuid, message });
                } else {
                    tracing::error!("failed deserializing: {}", bytes.as_str());
                }
            }
        }

        room.send(CoreMessage::Leave {
            uuid,
            socket: socket_uuid,
        });

        anyhow::Ok(())
    });

    Ok(())
}
