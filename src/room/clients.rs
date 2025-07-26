use std::{collections::HashMap, time::Duration};

use axum::extract::ws::{self, Utf8Bytes};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    messages::{RoomMessage, ServerMessage},
    room::{
        general::messages::{GeneralMessage, ServerGeneral},
        messenger::ClientMessenger,
    },
    task,
};

pub struct Clients {
    room: mpsc::UnboundedSender<RoomMessage>,
    clients: HashMap<Uuid, Client>,
}

struct Client {
    sender: mpsc::UnboundedSender<ws::Message>,
}

impl Clients {
    pub fn new(room: mpsc::UnboundedSender<RoomMessage>) -> Self {
        Self {
            room,
            clients: HashMap::new(),
        }
    }

    pub fn add(&mut self, uuid: Uuid, sender: mpsc::UnboundedSender<ws::Message>) {
        self.clients.insert(uuid, Client { sender });

        self.broadcast(ServerMessage::General(ServerGeneral::Join { uuid }));
    }

    pub fn remove(&mut self, uuid: Uuid) {
        if self.clients.remove(&uuid).is_some() {
            self.broadcast(ServerMessage::General(ServerGeneral::Leave { uuid }));

            if self.clients.is_empty() {
                let room = self.room.clone();

                task::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    room.send(RoomMessage::General(GeneralMessage::Close))?;

                    Ok(())
                });
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }
}

impl ClientMessenger<ServerMessage> for Clients {
    fn send(&self, uuid: Uuid, message: ServerMessage) {
        let text = serde_json::to_string(&message)
            .expect("ServerMessage serialization shouldn't ever fail?");

        let message = ws::Message::Text(Utf8Bytes::from(text));

        self.clients[&uuid].sender.send(message).ok();
    }

    fn broadcast(&self, message: ServerMessage) {
        let text = serde_json::to_string(&message)
            .expect("ServerMessage serialization shouldn't ever fail?");

        let message = ws::Message::Text(Utf8Bytes::from(text));

        for client in self.clients.values() {
            client.sender.send(message.clone()).ok();
        }
    }
}

impl ClientMessenger<ServerMessage> for &Clients {
    fn send(&self, uuid: Uuid, message: ServerMessage) {
        (*self).send(uuid, message);
    }

    fn broadcast(&self, message: ServerMessage) {
        (*self).broadcast(message);
    }
}
