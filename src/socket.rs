use axum::{
    extract::{
        Path, Query, State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;

use crate::{
    messages::RoomMessage,
    room::{clients::Client, general::messages::GeneralMessage, messenger::RoomMessenger},
    state::AppState,
    task,
};

#[derive(Deserialize)]
pub struct Params {
    pub username: String,
    pub rejoin_token: Option<Uuid>,
}

pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(room): Path<String>,
    Query(params): Query<Params>,
) -> Response {
    ws.max_message_size(256).on_upgrade(|ws| async {
        if let Err(err) = socket(ws, state, room, params).await {
            tracing::error!(?err);
        }
    })
}

async fn socket(
    socket: WebSocket,
    state: AppState,
    room: String,
    params: Params,
) -> anyhow::Result<()> {
    let (mut sink, mut stream) = socket.split();
    let (sender, mut reciever) = mpsc::unbounded_channel::<ws::Message>();

    // Room message -> WebSocket Sink
    task::spawn(async move {
        while let Some(message) = reciever.recv().await {
            sink.send(message).await?;
        }

        Ok(())
    });

    // Random UUID for this socket.
    let socket_uuid = Uuid::new_v4();

    let (uuid, room) = match (state.get_room(&room), params.rejoin_token) {
        (Some(room), Some(rejoin_token)) => {
            let (response, uuid) = oneshot::channel();

            room.send(RoomMessage::General(GeneralMessage::JoinWithRejoinToken {
                rejoin_token,
                socket_uuid,
                sender,
                response,
            }));

            let uuid = uuid.await?;

            (uuid, room)
        }
        (Some(room), None) => {
            let uuid = Uuid::new_v4();

            room.send(RoomMessage::General(GeneralMessage::Join {
                uuid,
                socket_uuid,
                sender,
            }));

            (uuid, room)
        }
        // If we don't have a room, it doesn't matter if we have a rejoin token.
        (None, Some(_)) | (None, None) => {
            let uuid = Uuid::new_v4();
            let room = state.insert_room(&room, (uuid, Client::new(socket_uuid, sender)));

            (uuid, room)
        }
    };

    // WebSocket Stream -> Room
    task::spawn(async move {
        while let Some(message) = stream.next().await {
            match message {
                Ok(ws::Message::Text(bytes)) => {
                    if let Ok(message) = serde_json::from_str(bytes.as_str()) {
                        room.send(RoomMessage::Client { uuid, message });
                    } else {
                        tracing::error!("failed deserializing: {}", bytes.as_str());
                    }
                }
                Ok(_) => (),
                Err(err) => tracing::error!(?err, "socket error"),
            }
        }

        room.send(RoomMessage::General(GeneralMessage::Leave {
            uuid,
            socket_uuid,
        }));

        Ok(())
    });

    Ok(())
}
