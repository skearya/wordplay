use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use axum::extract::ws::{self, CloseFrame, Utf8Bytes, close_code};
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

    pub fn close(&self, reason: &'static str) {
        self.sender
            .send(ws::Message::Close(Some(CloseFrame {
                code: close_code::ERROR,
                reason: Utf8Bytes::from_static(reason),
            })))
            .ok();
    }
}

pub struct Client {
    /// `Some` if connected.
    pub socket: Option<SocketRef>,
    /// Client username.
    pub username: String,
    /// Rejoin token, should be locally saved by client.
    pub rejoin_token: Uuid,
}

impl Client {
    pub fn new(socket: SocketRef, username: String) -> Self {
        Self {
            socket: Some(socket),
            username,
            rejoin_token: Uuid::new_v4(),
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
        let Some(socket) = &self.socket else {
            return;
        };

        socket.close(reason);
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
    /// Client UUID -> Client Data.
    clients: HashMap<Uuid, Client>,
}

impl Clients {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn get_by_rejoin_token(&mut self, rejoin_token: Uuid) -> Option<(&Uuid, &mut Client)> {
        self.clients
            .iter_mut()
            .find(|(_uuid, client)| client.rejoin_token == rejoin_token)
    }

    pub fn disconnect(&mut self, uuid: Uuid) {
        if let Some(client) = self.clients.get_mut(&uuid) {
            client.socket = None;
        }
    }

    pub fn len(&self) -> usize {
        self.clients
            .values()
            .filter(|client| client.connected())
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
}

impl Deref for Clients {
    type Target = HashMap<Uuid, Client>;

    fn deref(&self) -> &Self::Target {
        &self.clients
    }
}

impl DerefMut for Clients {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.clients
    }
}

impl From<&ServerMessage> for ws::Message {
    fn from(message: &ServerMessage) -> Self {
        let text = serde_json::to_string(message)
            .expect("ServerMessage serialization shouldn't ever fail?");

        ws::Message::Text(Utf8Bytes::from(text))
    }
}
