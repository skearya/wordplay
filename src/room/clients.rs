use std::{collections::HashMap, time::Duration};

use axum::extract::ws::{self, Utf8Bytes};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    general::messages::ServerGeneral,
    messages::{RoomMessage, ServerMessage},
    task,
};

pub struct Clients {
    room: mpsc::UnboundedSender<RoomMessage>,
    clients: HashMap<Uuid, Client>,
}

struct Client {
    sender: mpsc::UnboundedSender<ws::Message>,
}

pub trait Messenger<Message> {
    fn send(&self, uuid: Uuid, message: Message);
    fn broadcast(&self, message: Message);
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

        self.broadcast(&ServerMessage::General(ServerGeneral::Join { uuid }));
    }

    pub fn remove(&mut self, uuid: Uuid) {
        if self.clients.remove(&uuid).is_some() {
            self.broadcast(&ServerMessage::General(ServerGeneral::Leave { uuid }));

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

    pub fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }
}

impl Messenger<&ServerMessage> for Clients {
    fn send(&self, uuid: Uuid, message: &ServerMessage) {
        let text = serde_json::to_string(message)
            .expect("ServerMessage serialization shouldn't ever fail?");

        let message = ws::Message::Text(Utf8Bytes::from(text));

        self.clients[&uuid]
            .sender
            .send(message)
            .expect("client sender shouldn't be closed");
    }

    fn broadcast(&self, message: &ServerMessage) {
        let text = serde_json::to_string(message)
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

/// Creates an implementation of `Messenger` that can **only** send sub-enums of `ServerMessage`.
///
/// ### Usage
/// ```
/// submessenger!(&Clients, ServerMessage::Variant(SubEnum))
/// ```
///
/// ### Example
/// ```
/// // Send messages of `ServerMessage::Lobby` variant, which hold `ServerLobby` enums.
/// let sub = submessenger!(&self.clients, ServerMessage::Lobby(ServerLobby));
///
/// // Equivalent to `clients.broadcast(ServerMessage::Lobby(ServerLobby::Ready { uuid }))`
/// sub.broadcast(ServerLobby::Ready { uuid });
/// ```
macro_rules! submessenger {
    ($clients:expr, ServerMessage::$variant:tt($subtype:ty)) => {{
        struct MessengerImpl<'a>(&'a Clients);

        impl Messenger<$subtype> for MessengerImpl<'_> {
            fn send(&self, uuid: Uuid, message: $subtype) {
                self.0.send(uuid, &ServerMessage::$variant(message));
            }

            fn broadcast(&self, message: $subtype) {
                self.0.broadcast(&ServerMessage::$variant(message));
            }
        }

        MessengerImpl($clients)
    }};
}

pub(crate) use submessenger;
