use std::{collections::HashMap, time::Duration};

use axum::extract::ws::{self, Utf8Bytes};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::task;

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
    CloseTimeout,
}

enum State {
    Lobby,
    Game,
}

pub struct Room {
    sender: mpsc::UnboundedSender<RoomMessage>,
    reciever: mpsc::UnboundedReceiver<RoomMessage>,
    clients: HashMap<Uuid, Client>,
}

pub struct Client {
    sender: mpsc::UnboundedSender<ws::Message>,
}

impl Room {
    pub fn new(
        sender: mpsc::UnboundedSender<RoomMessage>,
        reciever: mpsc::UnboundedReceiver<RoomMessage>,
    ) -> Self {
        Self {
            sender,
            reciever,
            clients: HashMap::new(),
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
                    self.add(uuid, Client { sender });
                }
                RoomMessage::Left { uuid } => {
                    self.remove(&uuid);
                }
                RoomMessage::Client { uuid, message } => {
                    if let ws::Message::Text(bytes) = message {
                        let text = format!("{uuid}: {}", bytes.as_str());
                        let message = ws::Message::Text(Utf8Bytes::from(text));

                        for client in self.clients.values() {
                            client.sender.send(message.clone()).ok();
                        }
                    } else if let ws::Message::Close(_) = message {
                        self.remove(&uuid);
                    }
                }
                RoomMessage::CloseTimeout => {
                    if self.clients.is_empty() {
                        break;
                    }
                }
            }
        }

        tracing::info!("room done");

        Ok(())
    }

    fn add(&mut self, uuid: Uuid, client: Client) {
        self.clients.insert(uuid, client);
    }

    fn remove(&mut self, uuid: &Uuid) {
        if self.clients.remove(uuid).is_some() && self.clients.is_empty() {
            let sender = self.sender.clone();

            task::spawn(async move {
                tokio::time::sleep(Duration::from_secs(5)).await;

                sender.send(RoomMessage::CloseTimeout)?;

                Ok(())
            });
        }
    }
}
