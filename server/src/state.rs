use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rustrict::CensorStr;

use crate::room::{Room, sender::RoomSender};

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<AppStateInner>>,
}

struct AppStateInner {
    /// Room name -> Room task message sender.
    rooms: HashMap<String, RoomSender>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AppStateInner::new())),
        }
    }

    pub fn get_or_insert_room(&self, name: String) -> Result<RoomSender, &'static str> {
        let mut lock = match self.inner.lock() {
            Ok(lock) => lock,
            Err(poison) => poison.into_inner(),
        };

        lock.get_or_insert_room(name)
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

    fn make_room(&mut self, name: String) -> Result<RoomSender, &'static str> {
        if name.len() > 6 {
            Err("invalid room name, must be less than 6 characters")
        } else if !name.chars().all(|c| c.is_ascii_alphanumeric()) {
            Err("invalid room name, must be alphanumeric")
        } else if name.is_inappropriate() {
            Err("invalid room name, contains innappropriate content")
        } else {
            let room = Room::spawn();
            self.rooms.insert(name, room.clone());

            Ok(room)
        }
    }

    fn get_or_insert_room(&mut self, name: String) -> Result<RoomSender, &'static str> {
        if let Some(room) = self.get_room(&name) {
            Ok(room)
        } else {
            self.make_room(name)
        }
    }
}
