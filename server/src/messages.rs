use crate::state::{games::word_bomb, room::RoomSettings, AppState};

use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Games {
    WordBomb,
    Anagrams,
}

#[derive(Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ClientMessage {
    RoomSettings(RoomSettings),
    Ready,
    StartEarly,
    Unready,
    ChatMessage { content: String },
    WordBombInput { input: String },
    WordBombGuess { word: String },
}

#[derive(Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ServerMessage {
    // lobby / generic
    Info {
        uuid: Uuid,
        room: RoomInfo,
    },
    Error {
        content: String,
    },
    RoomSettings(RoomSettings),
    ChatMessage {
        author: Uuid,
        content: String,
    },
    ConnectionUpdate {
        uuid: Uuid,
        state: ConnectionUpdate,
    },
    ReadyPlayers {
        ready: Vec<Uuid>,
        countdown_update: Option<CountdownState>,
    },
    StartingCountdown {
        time_left: u8,
    },
    GameEnded {
        winner: Uuid,
        new_room_owner: Option<Uuid>,
    },

    // word bomb
    WordBombStarted {
        rejoin_token: Option<Uuid>,
        players: Vec<WordBombPlayerData>,
        prompt: String,
        turn: Uuid,
    },
    WordBombInput {
        uuid: Uuid,
        input: String,
    },
    WordBombInvalidGuess {
        uuid: Uuid,
        reason: word_bomb::GuessInfo,
    },
    WordBombPrompt {
        correct_guess: Option<String>,
        life_change: i8,
        prompt: String,
        turn: Uuid,
    },
}

#[derive(Serialize)]
pub struct RoomInfo {
    pub owner: Uuid,
    pub settings: RoomSettings,
    pub clients: Vec<ClientInfo>,
    pub state: RoomStateInfo,
}

#[derive(Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RoomStateInfo {
    Lobby {
        ready: Vec<Uuid>,
        starting_countdown: Option<u8>,
    },
    WordBomb {
        players: Vec<WordBombPlayerData>,
        turn: Uuid,
        prompt: String,
        used_letters: Option<HashSet<char>>,
    },
    Anagrams {
        players: Vec<AnagramsPlayerData>,
        prompt: String,
        used_words: Vec<String>,
    },
}

#[derive(Serialize)]
pub struct ClientInfo {
    pub uuid: Uuid,
    pub username: String,
}

#[derive(Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ConnectionUpdate {
    Connected { username: String },
    Reconnected { username: String },
    Disconnected { new_room_owner: Option<Uuid> },
}

#[derive(Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CountdownState {
    InProgress { time_left: u8 },
    Stopped,
}

#[derive(Serialize, Clone)]
pub struct WordBombPlayerData {
    pub uuid: Uuid,
    pub username: String,
    pub disconnected: bool,
    pub input: String,
    pub lives: u8,
}

#[derive(Serialize, Clone)]
pub struct AnagramsPlayerData {
    pub uuid: Uuid,
    pub username: String,
    pub disconnected: bool,
    pub used_words: Vec<String>,
}

impl ClientMessage {
    pub fn handle(self, state: &AppState, room: &str, uuid: Uuid) {
        let result = match self {
            ClientMessage::RoomSettings(room_settings) => {
                state.client_room_settings(room, uuid, room_settings)
            }
            ClientMessage::Ready => state.client_ready(room, uuid),
            ClientMessage::StartEarly => state.client_start_early(room, uuid),
            ClientMessage::Unready => state.client_unready(room, uuid),
            ClientMessage::ChatMessage { content } => {
                if content.len() > 250 {
                    Ok(())
                } else {
                    state.client_chat_message(room, uuid, content)
                }
            }
            ClientMessage::WordBombInput { input } => {
                if input.len() > 35 {
                    Ok(())
                } else {
                    state.client_input_update(room, uuid, input)
                }
            }
            ClientMessage::WordBombGuess { word } => {
                if word.len() > 35 {
                    Ok(())
                } else {
                    state.client_guess(
                        room,
                        uuid,
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
            state.send_error_msg(room, uuid, &e.to_string()).ok();
        }
    }
}

impl From<ServerMessage> for Message {
    fn from(msg: ServerMessage) -> Self {
        let serialized = serde_json::to_string(&msg).unwrap();
        Self::Text(serialized)
    }
}
