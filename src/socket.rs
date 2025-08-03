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
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    general::messages::GeneralMessage,
    messages::RoomMessage,
    room::{Room, clients::Client, messenger::RoomMessenger},
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
) -> Response {
    ws.max_message_size(256).on_upgrade(|ws| async {
        if let Err(err) = socket(state, room_name, params, ws).await {
            tracing::error!(?err);
        }
    })
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

    // Room message -> WebSocket Sink
    task::spawn(async move {
        while let Some(message) = reciever.recv().await {
            sink.send(message).await?;
        }

        Ok(())
    });

    // Random UUID for this socket.
    let socket = Uuid::new_v4();

    let (uuid, room) = match (state.get_room(&room_name), rejoin_token) {
        (Some(room), Some(rejoin_token)) => {
            let (response, uuid) = oneshot::channel();

            room.send(RoomMessage::General(GeneralMessage::JoinWithRejoinToken {
                rejoin_token,
                client: Client::new(socket, sender, username),
                response,
            }));

            let uuid = uuid.await?;

            (uuid, room)
        }
        (Some(room), None) => {
            let uuid = Uuid::new_v4();

            room.send(RoomMessage::General(GeneralMessage::Join {
                uuid,
                client: Client::new(socket, sender, username),
            }));

            (uuid, room)
        }
        // If we don't have a room, it doesn't matter if we have a rejoin token.
        (None, Some(_) | None) => {
            let uuid = Uuid::new_v4();
            let room = Room::spawn();

            room.send(RoomMessage::General(GeneralMessage::Join {
                uuid,
                client: Client::new(socket, sender, username),
            }));

            state.insert_room(room_name, room.clone());

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

        room.send(RoomMessage::General(GeneralMessage::Leave { uuid, socket }));

        Ok(())
    });

    Ok(())
}
