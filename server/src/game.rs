pub mod error;
pub mod games;
pub mod lobby;
pub mod messages;
pub mod room;

use crate::{utils::filter_str, AppState};
use error::GameError;
use messages::ClientMessage;
use room::Room;
use serde::Serialize;
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Debug)]
pub struct GameState {
    rooms: HashMap<String, Room>,
}

#[derive(Clone, Copy)]
pub struct SenderInfo<'a> {
    pub uuid: Uuid,
    pub room: &'a str,
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

impl AppState {
    pub fn new(db: SqlitePool) -> Self {
        Self {
            db,
            game: Arc::new(Mutex::new(GameState {
                rooms: HashMap::new(),
            })),
        }
    }

    pub fn info(&self) -> ServerInfo {
        let lock = self.game.lock().unwrap();

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
                self.word_bomb_guess(sender, filter_str(&word))
            }
            ClientMessage::AnagramsGuess { word } => self.anagrams_guess(sender, filter_str(&word)),
        };

        if let Err(error) = result {
            eprintln!("error: {} caused {:#?}", sender.uuid, error);
            self.send_error_msg(sender, &error.to_string()).ok();
        }
    }
}

impl GameState {
    pub fn room(&self, room: &str) -> Result<&Room, GameError> {
        self.rooms.get(room).ok_or(GameError::RoomNotFound {
            room: room.to_string(),
        })
    }

    pub fn room_mut(&mut self, room: &str) -> Result<&mut Room, GameError> {
        self.rooms.get_mut(room).ok_or(GameError::RoomNotFound {
            room: room.to_string(),
        })
    }
}
