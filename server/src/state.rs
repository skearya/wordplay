pub mod games;
mod lobby;
pub mod room;

use self::room::Room;
use crate::{Info, RoomData};

use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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
                .filter(|(_, room)| room.settings.public)
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
