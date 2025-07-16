use std::{collections::HashMap, time::Duration};

use axum::extract::ws::{self, Utf8Bytes};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    messages::server::{ServerGeneral, ServerMessage},
    task,
};

#[derive(Debug)]
pub enum RoomMessage {
    Joined {
        uuid: Uuid,
        sender: mpsc::UnboundedSender<ws::Message>,
    },
    Left {
        uuid: Uuid,
    },
    Client {
        uuid: Uuid,
        message: ws::Message,
    },
    CloseCheck,
}

pub struct Room {
    /// Sender to our own room task.
    sender: mpsc::UnboundedSender<RoomMessage>,
    reciever: mpsc::UnboundedReceiver<RoomMessage>,
    clients: Clients,
}

struct Clients {
    room: mpsc::UnboundedSender<RoomMessage>,
    clients: HashMap<Uuid, Client>,
}

struct Client {
    sender: mpsc::UnboundedSender<ws::Message>,
}

impl Room {
    pub fn new(
        sender: mpsc::UnboundedSender<RoomMessage>,
        reciever: mpsc::UnboundedReceiver<RoomMessage>,
    ) -> Self {
        Self {
            sender: sender.clone(),
            reciever,
            clients: Clients::new(sender),
        }
    }

    pub fn spawn() -> mpsc::UnboundedSender<RoomMessage> {
        let (sender, reciever) = mpsc::unbounded_channel::<RoomMessage>();

        task::spawn(Self::new(sender.clone(), reciever).run());

        sender
    }

    async fn run(mut self) -> anyhow::Result<()> {
        while let Some(message) = self.reciever.recv().await {
            tracing::debug!(?message);

            match message {
                RoomMessage::Joined { uuid, sender } => {
                    self.clients.add(uuid, Client { sender });
                }
                RoomMessage::Left { uuid } => {
                    self.clients.remove(uuid);
                }
                RoomMessage::Client { uuid, message } => match message {
                    ws::Message::Text(bytes) => {
                        self.clients
                            .broadcast(ServerMessage::General(ServerGeneral::Chat {
                                author: uuid,
                                content: bytes.as_str().to_owned(),
                            }));
                    }
                    ws::Message::Close(_) => {
                        self.clients.remove(uuid);
                    }
                    _ => (),
                },
                RoomMessage::CloseCheck => {
                    if self.clients.is_empty() {
                        self.reciever.close();
                    }
                }
            }
        }

        Ok(())
    }
}

impl Clients {
    fn new(room: mpsc::UnboundedSender<RoomMessage>) -> Self {
        Self {
            room,
            clients: HashMap::new(),
        }
    }

    fn add(&mut self, uuid: Uuid, client: Client) {
        self.clients.insert(uuid, client);

        self.broadcast(ServerMessage::General(ServerGeneral::Join { uuid }));
    }

    fn remove(&mut self, uuid: Uuid) {
        if self.clients.remove(&uuid).is_some() {
            self.broadcast(ServerMessage::General(ServerGeneral::Leave { uuid }));

            if self.clients.is_empty() {
                let room = self.room.clone();

                task::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(5)).await;

                    room.send(RoomMessage::CloseCheck)?;

                    Ok(())
                });
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    fn send(&self, uuid: &Uuid, message: ServerMessage) {
        let text = serde_json::to_string(&message)
            .expect("ServerMessage serialization shouldn't ever fail?");

        let message = ws::Message::Text(Utf8Bytes::from(text));

        self.clients[uuid]
            .sender
            .send(message)
            .expect("client sender shouldn't be closed");
    }

    fn broadcast(&self, message: ServerMessage) {
        let text = serde_json::to_string(&message)
            .expect("ServerMessage serialization shouldn't ever fail?");

        let message = ws::Message::Text(Utf8Bytes::from(text));

        for client in self.clients.values() {
            client
                .sender
                .send(message.clone())
                .expect("client sender shouldn't be closed");
        }
    }
}
