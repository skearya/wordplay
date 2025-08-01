use std::{collections::HashMap, time::Duration};

use axum::extract::ws::{self, CloseFrame, Utf8Bytes, close_code};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    general::messages::{GeneralMessage, ServerGeneral}, messages::{RoomMessage, ServerMessage}, room::{
        messenger::{ClientMessenger, ClientUtils, ClientUtilsMut, RoomMessenger},
        sender::RoomSender,
    }, task
};

pub struct Client {
    /// UUID unique to the client's socket task. **Not** client UUID.
    socket: Uuid,
    /// Sender to the client's reciever task that proxies WebSocket messages.
    sender: mpsc::UnboundedSender<ws::Message>,
    /// Client username.
    pub username: String,
}

impl Client {
    pub fn new(socket: Uuid, sender: mpsc::UnboundedSender<ws::Message>, username: String) -> Self {
        Self {
            socket,
            sender,
            username,
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

pub struct Clients {
    clients: HashMap<Uuid, Client>,
    room: RoomSender,
}

impl Clients {
    pub fn new(room: RoomSender) -> Self {
        Self {
            clients: HashMap::new(),
            room,
        }
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

impl ClientUtils for Clients {
    fn get(&self, uuid: Uuid) -> Option<&Client> {
        self.clients.get(&uuid)
    }

    fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    fn iter(&self) -> std::collections::hash_map::Iter<'_, Uuid, Client> {
        self.clients.iter()
    }
}

impl ClientUtilsMut for Clients {
    fn add(&mut self, uuid: Uuid, client: Client) {
        self.clients.insert(uuid, client);

        self.broadcast_except(uuid, ServerMessage::General(ServerGeneral::Join { uuid }));
    }

    fn remove(&mut self, uuid: Uuid, socket: Uuid) {
        if let Some(client) = self.get(uuid) {
            if client.socket == socket {
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

    fn get_mut(&mut self, uuid: Uuid) -> Option<&mut Client> {
        self.clients.get_mut(&uuid)
    }
}
