mod games;
mod lobby;
mod room;

use self::room::{Client, Room};
use crate::{messages::ServerMessage, Info, RoomData};

use anyhow::{Context, Result};
use axum::extract::ws::Message;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<AppStateInner>>,
}

pub struct AppStateInner {
    rooms: HashMap<String, Room>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AppStateInner {
                rooms: HashMap::new(),
            })),
        }
    }

    pub fn info(&self) -> Info {
        let lock = self.inner.lock().unwrap();

        Info {
            clients_connected: lock.rooms.values().map(|room| room.clients.len()).sum(),
            public_rooms: lock
                .rooms
                .iter()
                .filter(|(_, room)| room.public)
                .map(|(name, data)| RoomData {
                    name: name.clone(),
                    players: data.clients.len(),
                })
                .collect(),
        }
    }
}

impl AppStateInner {
    pub fn room(&self, room: &str) -> Result<&Room> {
        self.rooms.get(room).context("Room not found")
    }

    pub fn room_mut(&mut self, room: &str) -> Result<&mut Room> {
        self.rooms.get_mut(room).context("Room not found")
    }
}

pub trait ClientUtils {
    fn send_each(&self, f: impl Fn(&Uuid, &Client) -> ServerMessage);
    fn broadcast(&self, message: ServerMessage);
}

impl ClientUtils for HashMap<Uuid, Client> {
    fn send_each(&self, f: impl Fn(&Uuid, &Client) -> ServerMessage) {
        for (uuid, client) in self.iter().filter(|client| client.1.socket.is_some()) {
            client.tx.send(f(uuid, client).into()).ok();
        }
    }

    fn broadcast(&self, message: ServerMessage) {
        let serialized: Message = message.into();

        for client in self.values().filter(|client| client.socket.is_some()) {
            client.tx.send(serialized.clone()).ok();
        }
    }
}
