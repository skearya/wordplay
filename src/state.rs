use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::room::sender::RoomSender;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<AppStateInner>>,
}

struct AppStateInner {
    /// Room name -> Room task message sender
    rooms: HashMap<String, RoomSender>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AppStateInner::new())),
        }
    }

    pub fn get_room(&self, name: &str) -> Option<RoomSender> {
        let mut lock = match self.inner.lock() {
            Ok(lock) => lock,
            Err(poison) => poison.into_inner(),
        };

        lock.get_room(name)
    }

    pub fn insert_room(&self, name: String, room: RoomSender) {
        let mut lock = match self.inner.lock() {
            Ok(lock) => lock,
            Err(poison) => poison.into_inner(),
        };

        lock.insert_room(name, room);
    }
}

impl AppStateInner {
    fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    fn get_room(&mut self, name: &str) -> Option<RoomSender> {
        let room = self.rooms.get(name)?;

        if room.is_closed() {
            self.rooms.remove(name);

            None
        } else {
            Some(room.clone())
        }
    }

    fn insert_room(&mut self, name: String, room: RoomSender) {
        self.rooms.insert(name, room);
    }
}
