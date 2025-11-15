use std::collections::HashMap;

use axum::extract::ws::{self, CloseFrame, Utf8Bytes, close_code};
use rand::{rng, seq::IteratorRandom};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::messages::ServerMessage;

pub struct SocketRef {
    /// UUID unique to the client's socket task. **Not** client UUID.
    uuid: Uuid,
    /// Sender to the client's reciever task that proxies WebSocket messages.
    sender: mpsc::UnboundedSender<ws::Message>,
}

impl SocketRef {
    pub fn new(uuid: Uuid, sender: mpsc::UnboundedSender<ws::Message>) -> Self {
        Self { uuid, sender }
    }
}

pub struct Client {
    /// `Some` if connected.
    socket: Option<SocketRef>,
    /// Client username.
    pub username: String,
}

impl Client {
    pub fn new(socket: SocketRef, username: String) -> Self {
        Self {
            socket: Some(socket),
            username,
        }
    }

    pub fn send_raw(&self, message: ws::Message) {
        let Some(socket) = &self.socket else { return };

        socket.sender.send(message).ok();
    }

    pub fn send(&self, message: impl Into<ServerMessage>) {
        let Some(socket) = &self.socket else { return };

        let message: ServerMessage = message.into();
        let message: ws::Message = (&message).into();

        socket.sender.send(message).ok();
    }

    pub fn close(&self, reason: &'static str) {
        let Some(socket) = &self.socket else { return };

        socket
            .sender
            .send(ws::Message::Close(Some(CloseFrame {
                code: close_code::ERROR,
                reason: Utf8Bytes::from_static(reason),
            })))
            .ok();
    }

    pub fn socket_uuid_eq(&self, other: Uuid) -> bool {
        self.socket
            .as_ref()
            .is_some_and(|socket| socket.uuid == other)
    }

    pub fn connected(&self) -> bool {
        self.socket.is_some()
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

    pub fn insert(&mut self, uuid: Uuid, client: Client) {
        self.clients.insert(uuid, client);
    }

    pub fn get(&self, uuid: Uuid) -> Option<&Client> {
        self.clients.get(&uuid)
    }

    pub fn get_mut(&mut self, uuid: Uuid) -> Option<&mut Client> {
        self.clients.get_mut(&uuid)
    }

    pub fn disconnect(&mut self, uuid: Uuid) {
        if let Some(client) = self.clients.get_mut(&uuid) {
            client.socket = None;
        }
    }

    pub fn remove(&mut self, uuid: Uuid) {
        self.clients.remove(&uuid);
    }

    pub fn random(&self) -> (&Uuid, &Client) {
        self.clients
            .iter()
            .choose(&mut rng())
            .expect("should always be at least one client")
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Uuid, &Client)> {
        self.clients.iter()
    }

    pub fn keep_connected(&mut self) {
        self.clients.retain(|_, client| client.connected());
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty() || self.clients.values().all(|client| !client.connected())
    }

    pub fn send(&self, uuid: Uuid, message: impl Into<ServerMessage>) {
        self.clients[&uuid].send(message);
    }

    pub fn broadcast(&self, message: impl Into<ServerMessage>) {
        let message: ServerMessage = message.into();
        let message: ws::Message = (&message).into();

        for client in self.clients.values() {
            client.send_raw(message.clone());
        }
    }

    pub fn broadcast_except(&self, exclude: Uuid, message: impl Into<ServerMessage>) {
        let message: ServerMessage = message.into();
        let message: ws::Message = (&message).into();

        for (&uuid, client) in &self.clients {
            if uuid != exclude {
                client.send_raw(message.clone());
            }
        }
    }
}

impl From<&ServerMessage> for ws::Message {
    fn from(message: &ServerMessage) -> Self {
        let text = serde_json::to_string(message)
            .expect("ServerMessage serialization shouldn't ever fail?");

        ws::Message::Text(Utf8Bytes::from(text))
    }
}
