use crate::models::AppState;

use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ClientMessage {
    GameSettings { public: bool },
    Ready,
    StartEarly,
    Unready,
    ChatMessage { content: String },
    Input { input: String },
    Guess { word: String },
}

#[derive(Serialize, Clone, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ServerMessage {
    Info {
        uuid: Uuid,
        room: RoomInfo,
    },
    GameSettings {
        public: bool,
    },
    ServerMessage {
        content: String,
    },
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
    GameStarted {
        rejoin_token: Option<Uuid>,
        players: Vec<PlayerData>,
        prompt: String,
        turn: Uuid,
    },
    InputUpdate {
        uuid: Uuid,
        input: String,
    },
    InvalidWord {
        uuid: Uuid,
        reason: InvalidWordReason,
    },
    NewPrompt {
        word: Option<String>,
        life_change: i8,
        new_prompt: String,
        new_turn: Uuid,
    },
    GameEnded {
        winner: Uuid,
        new_room_owner: Option<Uuid>,
    },
}

#[derive(Serialize, Clone, Debug)]
pub struct RoomInfo {
    pub public: bool,
    pub owner: Uuid,
    pub clients: Vec<ClientInfo>,
    pub state: RoomState,
}

#[derive(Serialize, Clone, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RoomState {
    Lobby {
        ready: Vec<Uuid>,
        starting_countdown: Option<u8>,
    },
    InGame {
        players: Vec<PlayerData>,
        turn: Uuid,
        prompt: String,
        used_letters: Option<HashSet<char>>,
    },
}

#[derive(Serialize, Clone, Debug)]
pub struct ClientInfo {
    pub uuid: Uuid,
    pub username: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct PlayerData {
    pub uuid: Uuid,
    pub username: String,
    pub disconnected: bool,
    pub input: String,
    pub lives: u8,
}

#[derive(Serialize, Clone, Debug)]
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

#[derive(Serialize, Clone, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum CountdownState {
    InProgress { time_left: u8 },
    Stopped,
}

#[derive(Serialize, Clone, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum InvalidWordReason {
    PromptNotIn,
    NotEnglish,
    AlreadyUsed,
}

impl ClientMessage {
    pub fn handle(self, state: &AppState, room: &str, uuid: Uuid) {
        let result = match self {
            ClientMessage::GameSettings { public } => {
                state.client_game_settings(room, uuid, public)
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
            ClientMessage::Input { input } => {
                if input.len() > 35 {
                    Ok(())
                } else {
                    state.client_input_update(room, uuid, input)
                }
            }
            ClientMessage::Guess { word } => {
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
            state.errored(room, uuid).ok();
        }
    }
}

impl From<ServerMessage> for Message {
    fn from(msg: ServerMessage) -> Self {
        let serialized = serde_json::to_string(&msg).unwrap();

        Self::Text(serialized)
    }
}
