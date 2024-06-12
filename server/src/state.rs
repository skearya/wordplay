pub mod games;
pub mod lobby;
pub mod room;

use self::room::Room;
use crate::{messages::ClientMessage, utils::filter_string};
use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<Mutex<AppStateInner>>,
}

#[derive(Debug)]
pub struct AppStateInner {
    rooms: HashMap<String, Room>,
}

#[derive(Serialize, Debug)]
pub struct ServerInfo {
    pub clients_connected: usize,
    pub public_rooms: Vec<RoomData>,
}

#[derive(Serialize, Debug)]
pub struct RoomData {
    pub name: String,
    pub players: usize,
}

#[derive(Clone, Copy)]
pub struct SenderInfo<'a> {
    pub uuid: Uuid,
    pub room: &'a str,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AppStateInner {
                rooms: HashMap::new(),
            })),
        }
    }

    pub fn info(&self) -> ServerInfo {
        let lock = self.inner.lock().unwrap();

        ServerInfo {
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

    pub fn handle(&self, sender: SenderInfo, message: ClientMessage) {
        let result = match message {
            ClientMessage::RoomSettings(settings) => self.client_room_settings(sender, settings),
            ClientMessage::Ready => self.client_ready(sender),
            ClientMessage::StartEarly => self.client_start_early(sender),
            ClientMessage::Unready => self.client_unready(sender),
            ClientMessage::ChatMessage { content } => self.client_chat_message(sender, content),
            ClientMessage::WordBombInput { input } => self.word_bomb_input(sender, input),
            ClientMessage::WordBombGuess { word } => {
                self.word_bomb_guess(sender, &filter_string(word))
            }
            ClientMessage::AnagramsGuess { word } => {
                self.anagrams_guess(sender, &filter_string(word))
            }
        };

        if let Err(e) = result {
            eprintln!("error handling msg: {e}");
            self.send_error_msg(sender, &e.to_string()).ok();
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
