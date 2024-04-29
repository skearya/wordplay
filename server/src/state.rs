pub mod games;
mod lobby;
pub mod room;

use self::room::Room;
use crate::{messages::ClientMessage, Info, RoomData};

use anyhow::{Context, Result};
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

    pub fn handle(&self, sender: SenderInfo, message: ClientMessage) {
        let result = match message {
            ClientMessage::RoomSettings(room_settings) => {
                self.client_room_settings(sender, room_settings)
            }
            ClientMessage::Ready => self.client_ready(sender),
            ClientMessage::StartEarly => self.client_start_early(sender),
            ClientMessage::Unready => self.client_unready(sender),
            ClientMessage::ChatMessage { content } => {
                if content.len() > 250 {
                    Ok(())
                } else {
                    self.client_chat_message(sender, content)
                }
            }
            ClientMessage::WordBombInput { input } => {
                if input.len() > 35 {
                    Ok(())
                } else {
                    self.word_bomb_input(sender, input)
                }
            }
            ClientMessage::WordBombGuess { word } => {
                if word.len() > 35 {
                    Ok(())
                } else {
                    self.word_bomb_guess(
                        sender,
                        &word
                            .to_ascii_lowercase()
                            .chars()
                            .filter(|char| char.is_alphabetic())
                            .collect::<String>(),
                    )
                }
            }
            ClientMessage::AnagramsGuess { word } => {
                if word.len() > 35 {
                    Ok(())
                } else {
                    self.anagrams_guess(
                        sender,
                        &word
                            .to_ascii_lowercase()
                            .chars()
                            .filter(|char| char.is_alphabetic())
                            .collect::<String>(),
                    )
                }
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
