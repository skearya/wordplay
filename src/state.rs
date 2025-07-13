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
        match self.get_room(&name) {
            Some(room) => room,
            None => {
                let room = Room::spawn();

                self.rooms.insert(name, room.clone());

                room
            }
        }
    }

    fn get_room(&mut self, name: &str) -> Option<mpsc::UnboundedSender<RoomMessage>> {
        match self.rooms.get(name) {
            Some(room) => {
                if !room.is_closed() {
                    Some(room.clone())
                } else {
                    self.rooms.remove(name);
                    None
                }
            }
            None => None,
        }
    }
}
