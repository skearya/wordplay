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

use crate::{
    messages::RoomMessage, room::general::messages::GeneralMessage, state::AppState, task,
};

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

    // Room message -> WebSocket Sink
    task::spawn(async move {
        while let Some(message) = reciever.recv().await {
            sink.send(message).await?;
        }

        Ok(())
    });

    // Random UUID for this client.
    let uuid = Uuid::new_v4();
    let room = state.get_or_insert_room(room);

    room.send(RoomMessage::General(GeneralMessage::Joined {
        uuid,
        sender,
    }))?;

    // WebSocket Stream -> Room
    task::spawn(async move {
        while let Some(message) = stream.next().await {
            match message {
                Ok(ws::Message::Text(bytes)) => {
                    if let Ok(message) = serde_json::from_str(bytes.as_str()) {
                        room.send(RoomMessage::Client { uuid, message })?;
                    } else {
                        tracing::error!("failed deserializing: {}", bytes.as_str());
                    }
                }
                Ok(ws::Message::Close(_)) => break,
                Err(err) => {
                    tracing::error!(?err, "socket error");
                    break;
                }
                _ => (),
            }
        }

        room.send(RoomMessage::General(GeneralMessage::Left { uuid }))?;

        Ok(())
    });

    Ok(())
}
