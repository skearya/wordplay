pub mod games;
mod lobby;
mod room;

use self::games::GameState;
use crate::{Info, RoomData};

use anyhow::{Context, Result};
use axum::extract::ws::Message;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<AppStateInner>>,
}

pub struct AppStateInner {
    rooms: HashMap<String, Room>,
}

#[derive(Default)]
pub struct Room {
    pub public: bool,
    pub owner: Uuid,
    pub clients: HashMap<Uuid, Client>,
    pub state: GameState,
}

pub struct Client {
    pub socket: Option<Uuid>,
    pub tx: UnboundedSender<Message>,
    pub username: String,
}

impl AppStateInner {
    pub fn room(&self, room: &str) -> Result<&Room> {
        self.rooms.get(room).context("Room not found")
    }

    pub fn room_mut(&mut self, room: &str) -> Result<&mut Room> {
        self.rooms.get_mut(room).context("Room not found")
    }
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
