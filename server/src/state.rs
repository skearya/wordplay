pub mod error;
pub mod games;
pub mod lobby;
pub mod messages;
pub mod room;

use error::GameError;
use messages::ClientMessage;
use room::Room;
use sqlx::SqlitePool;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub game: Arc<Mutex<GameState>>,
}

#[derive(Debug)]
pub struct GameState {
    pub rooms: HashMap<String, Room>,
}

#[derive(Clone, Copy)]
pub struct SenderInfo<'a> {
    pub uuid: Uuid,
    pub room: &'a str,
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

    pub fn handle(&self, sender: SenderInfo, message: ClientMessage) {
        let result = match message {
            ClientMessage::Ping { timestamp } => self.client_ping(sender, timestamp),
            ClientMessage::RoomSettings(settings) => self.client_room_settings(sender, settings),
            ClientMessage::Ready => self.client_ready(sender),
            ClientMessage::StartEarly => self.client_start_early(sender),
            ClientMessage::Unready => self.client_unready(sender),
            ClientMessage::ChatMessage { content } => self.client_chat_message(sender, content),
            ClientMessage::WordBombInput { input } => self.word_bomb_input(sender, input),
            ClientMessage::WordBombGuess { word } => self.word_bomb_guess(sender, word),
            ClientMessage::AnagramsGuess { word } => self.anagrams_guess(sender, word),
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
