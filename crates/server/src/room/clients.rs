use std::collections::HashMap;

use axum::extract::ws::{self, CloseFrame, Utf8Bytes, close_code};
use rand::{rng, seq::IteratorRandom};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{messages::ServerMessage, room::messenger::ClientMessenger};

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
        self.socket.as_ref().is_some_and(|(uuid, _)| *uuid == other)
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

impl Clients {
    pub fn add(&mut self, uuid: Uuid, client: Client) {
        self.clients.insert(uuid, client);
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

    pub fn get(&self, uuid: Uuid) -> Option<&Client> {
        self.clients.get(&uuid)
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
        self.clients.retain(|_, client| client.socket.is_some());
    }

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty() || self.clients.values().all(|client| client.socket.is_none())
    }
}

impl ClientMessenger<ServerMessage> for Clients {
    fn send(&self, uuid: Uuid, message: ServerMessage) {
        let Some((_, socket)) = self.clients[&uuid].socket.as_ref() else {
            return;
        };

        let message: ws::Message = (&message).into();

        socket.send(message).ok();
    }

    fn broadcast(&self, message: ServerMessage) {
        let message: ws::Message = (&message).into();

        for (_, socket) in self
            .clients
            .values()
            .filter_map(|client| client.socket.as_ref())
        {
            socket.send(message.clone()).ok();
        }
    }

    fn broadcast_except(&self, exclude: Uuid, message: ServerMessage) {
        let message: ws::Message = (&message).into();

        for (_, socket) in self
            .clients
            .iter()
            .filter(|&(uuid, _)| *uuid != exclude)
            .filter_map(|(_, client)| client.socket.as_ref())
        {
            socket.send(message.clone()).ok();
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
