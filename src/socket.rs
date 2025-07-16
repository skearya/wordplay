use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{room::RoomMessage, state::AppState, task};

pub async fn handler(
    State(state): State<AppState>,
    Path(room): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.max_message_size(256).on_upgrade(|ws| async {
        if let Err(err) = socket(state, room, ws) {
            tracing::error!(?err);
        }
    })
}

fn socket(state: AppState, room: String, socket: WebSocket) -> anyhow::Result<()> {
    let (mut sink, mut stream) = socket.split();
    let (sender, mut reciever) = mpsc::unbounded_channel::<ws::Message>();

    task::spawn(async move {
        while let Some(message) = reciever.recv().await {
            sink.send(message).await?;
        }

        Ok(())
    });

    // Random UUID for this client.
    let uuid = Uuid::new_v4();
    let room = state.get_or_insert_room(room);

    room.send(RoomMessage::Joined { uuid, sender })?;

    task::spawn(async move {
        while let Some(message) = stream.next().await {
            match message {
                Ok(message) => {
                    room.send(RoomMessage::Client { uuid, message })?;
                }
                Err(err) => {
                    tracing::error!(?err);
                    break;
                }
            }
        }

        room.send(RoomMessage::Left { uuid })?;

        Ok(())
    });

    Ok(())
}
