use std::{collections::HashMap, time::Duration};

use axum::extract::ws::{self, CloseFrame, Utf8Bytes, close_code};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    messages::{RoomMessage, ServerMessage},
    room::{
        general::messages::{GeneralMessage, ServerGeneral},
        messenger::{ClientMessenger, RoomMessenger},
        sender::RoomSender,
    },
    task,
};

pub struct Clients {
    room: RoomSender,
    clients: HashMap<Uuid, Client>,
}

pub struct Client {
    /// UUID unique to the client's socket task. **Not** client UUID.
    socket_uuid: Uuid,
    /// Sender to the client's reciever task that proxies WebSocket messages.
    sender: mpsc::UnboundedSender<ws::Message>,
}

impl Client {
    pub fn new(socket_uuid: Uuid, sender: mpsc::UnboundedSender<ws::Message>) -> Self {
        Self {
            socket_uuid,
            sender,
        }
    }

    pub fn close(&self, reason: &'static str) {
        self.sender
            .send(ws::Message::Close(Some(CloseFrame {
                code: close_code::ERROR,
                reason: Utf8Bytes::from_static(reason),
            })))
            .ok();
    }
}

impl Clients {
    pub fn new(room: RoomSender, owner: (Uuid, Client)) -> Self {
        Self {
            room,
            clients: HashMap::from([owner]),
        }
    }

    pub fn add(
        &mut self,
        uuid: Uuid,
        socket_uuid: Uuid,
        sender: mpsc::UnboundedSender<ws::Message>,
    ) {
        self.clients.insert(
            uuid,
            Client {
                socket_uuid,
                sender,
            },
        );

        self.broadcast_except(uuid, ServerMessage::General(ServerGeneral::Join { uuid }));
    }

    pub fn remove(&mut self, uuid: Uuid, socket_uuid: Uuid) {
        if let Some(client) = self.get(&uuid) {
            if client.socket_uuid == socket_uuid {
                self.clients.remove(&uuid);

                self.broadcast(ServerMessage::General(ServerGeneral::Leave { uuid }));

                if self.is_empty() {
                    let room = self.room.clone();

                    task::spawn(async move {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        room.send(RoomMessage::General(GeneralMessage::Close));

                        Ok(())
                    });
                }
            }
        }
    }

    pub fn get(&self, uuid: &Uuid) -> Option<&Client> {
        self.clients.get(uuid)
    }

    pub fn get_mut(&mut self, uuid: &Uuid) -> Option<&mut Client> {
        self.clients.get_mut(uuid)
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

    fn broadcast_except(&self, exclude: Uuid, message: ServerMessage) {
        let text = serde_json::to_string(&message)
            .expect("ServerMessage serialization shouldn't ever fail?");

        let message = ws::Message::Text(Utf8Bytes::from(text));

        for (uuid, client) in &self.clients {
            if *uuid == exclude {
                continue;
            }

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

    fn broadcast_except(&self, exclude: Uuid, message: ServerMessage) {
        (*self).broadcast_except(exclude, message);
    }
}
