use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::sync::mpsc;

use crate::room::{Room, RoomMessage};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<AppStateInner>>,
}

struct AppStateInner {
    /// Room name -> Room task message sender
    rooms: HashMap<String, mpsc::UnboundedSender<RoomMessage>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AppStateInner::new())),
        }
    }

    pub fn get_or_insert_room(&self, name: String) -> mpsc::UnboundedSender<RoomMessage> {
        let mut lock = match self.inner.lock() {
            Ok(lock) => lock,
            Err(poison) => poison.into_inner(),
        };

        lock.get_or_insert_room(name)
    }

    pub fn get_room(&self, name: &str) -> Option<mpsc::UnboundedSender<RoomMessage>> {
        let mut lock = match self.inner.lock() {
            Ok(lock) => lock,
            Err(poison) => poison.into_inner(),
        };

        lock.get_room(name)
    }
}

impl AppStateInner {
    fn new() -> Self {
        Self {
            rooms: HashMap::new(),
        }
    }

    fn get_or_insert_room(&mut self, name: String) -> mpsc::UnboundedSender<RoomMessage> {
        if let Some(room) = self.get_room(&name) {
            room
        } else {
            let room = Room::spawn();
            self.rooms.insert(name, room.clone());

            room
        }
    }

    fn get_room(&mut self, name: &str) -> Option<mpsc::UnboundedSender<RoomMessage>> {
        let room = self.rooms.get(name)?;

        if room.is_closed() {
            self.rooms.remove(name);
            None
        } else {
            Some(room.clone())
        }
    }
}
