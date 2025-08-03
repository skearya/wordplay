use std::collections::HashMap;

use axum::extract::ws::{self, CloseFrame, Utf8Bytes, close_code};
use rand::{rng, seq::IteratorRandom};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    messages::ServerMessage,
    room::messenger::{ClientMessenger, ClientUtils, ClientUtilsMut},
};

pub struct Client {
    /// 0: UUID unique to the client's socket task. **Not** client UUID.
    /// 1: Sender to the client's reciever task that proxies WebSocket messages.
    /// `Some` if connected.
    socket: Option<(Uuid, mpsc::UnboundedSender<ws::Message>)>,
    /// Client username.
    pub username: String,
}

impl Client {
    pub fn new(socket: Uuid, sender: mpsc::UnboundedSender<ws::Message>, username: String) -> Self {
        Self {
            socket: Some((socket, sender)),
            username,
        }
    }

    pub fn socket_uuid_eq(&self, other: Uuid) -> bool {
        self.socket.as_ref().is_some_and(|socket| socket.0 == other)
    }

    pub fn close(&self, reason: &'static str) {
        let Some(socket) = &self.socket else { return };

        socket
            .1
            .send(ws::Message::Close(Some(CloseFrame {
                code: close_code::ERROR,
                reason: Utf8Bytes::from_static(reason),
            })))
            .ok();
    }
}

pub struct Clients {
    clients: HashMap<Uuid, Client>,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
}

impl Into<ws::Message> for ServerMessage {
    fn into(self) -> ws::Message {
        let text =
            serde_json::to_string(&self).expect("ServerMessage serialization shouldn't ever fail?");

        ws::Message::Text(Utf8Bytes::from(text))
    }
}

impl ClientMessenger<ServerMessage> for Clients {
    fn send(&self, uuid: Uuid, message: ServerMessage) {
        let Some(socket) = self.clients[&uuid].socket.as_ref() else {
            return;
        };

        let message: ws::Message = message.into();

        socket.1.send(message).ok();
    }

    fn broadcast(&self, message: ServerMessage) {
        let message: ws::Message = message.into();

        for socket in self
            .clients
            .values()
            .filter_map(|client| client.socket.as_ref())
        {
            socket.1.send(message.clone()).ok();
        }
    }

    fn broadcast_except(&self, exclude: Uuid, message: ServerMessage) {
        let message: ws::Message = message.into();

        for socket in self
            .clients
            .values()
            .filter_map(|client| client.socket.as_ref())
            .filter(|socket| socket.0 != exclude)
        {
            socket.1.send(message.clone()).ok();
        }
    }
}

impl ClientUtils for Clients {
    fn get(&self, uuid: Uuid) -> Option<&Client> {
        self.clients.get(&uuid)
    }

    fn random(&self) -> (&Uuid, &Client) {
        self.iter()
            .choose(&mut rng())
            .expect("should always be at least one client")
    }

    fn is_empty(&self) -> bool {
        self.clients.is_empty() || self.clients.values().all(|client| client.socket.is_none())
    }

    fn iter(&self) -> std::collections::hash_map::Iter<'_, Uuid, Client> {
        self.clients.iter()
    }
}

impl ClientUtilsMut for Clients {
    fn add(&mut self, uuid: Uuid, client: Client) {
        self.clients.insert(uuid, client);
    }

    fn remove(&mut self, uuid: Uuid) {
        self.clients.remove(&uuid);
    }

    fn disconnect(&mut self, uuid: Uuid) {
        self.clients.get_mut(&uuid).unwrap().socket = None;
    }

    fn get_mut(&mut self, uuid: Uuid) -> Option<&mut Client> {
        self.clients.get_mut(&uuid)
    }
}
