use crate::models::AppState;

use std::collections::HashSet;

use axum::extract::ws::Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ClientMessage {
    Ready,
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
    RoomInfo {
        uuid: Uuid,
        state: RoomState,
    },
    ServerMessage {
        content: String,
    },
    ChatMessage {
        author: String,
        content: String,
    },
    ReadyPlayers {
        players: Vec<PlayerInfo>,
    },
    StartingCountdown {
        state: CountdownState,
    },
    GameStarted {
        rejoin_token: Option<Uuid>,
        prompt: String,
        turn: Uuid,
        players: Vec<PlayerData>,
    },
    PlayerUpdate {
        uuid: Uuid,
        state: PlayerUpdate,
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
    },
}

#[derive(Serialize, Clone, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum RoomState {
    Lobby {
        ready: Vec<PlayerInfo>,
        starting_countdown: Option<u8>,
    },
    InGame {
        prompt: String,
        turn: Uuid,
        players: Vec<PlayerData>,
        used_letters: Option<HashSet<char>>,
    },
}

#[derive(Serialize, Clone, Debug)]
pub struct PlayerInfo {
    pub uuid: Uuid,
    pub username: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct PlayerData {
    pub uuid: Uuid,
    pub username: String,
    pub input: String,
    pub lives: u8,
    pub disconnected: bool,
}

#[derive(Serialize, Clone, Debug)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum PlayerUpdate {
    Disconnected,
    Reconnected { username: String },
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
        match self {
            ClientMessage::Ready => {
                state.client_ready(room, uuid);
            }
            ClientMessage::Unready => todo!(),
            ClientMessage::ChatMessage { content } => {
                if content.len() > 250 {
                    return;
                }

                state.client_chat_message(room, uuid, content);
            }
            ClientMessage::Input { input } => {
                if input.len() > 35 {
                    return;
                }

                state.client_input_update(room, uuid, input)
            }
            ClientMessage::Guess { word } => {
                if word.len() > 35 {
                    return;
                }

                state.client_guess(
                    room,
                    uuid,
                    &word
                        .to_ascii_lowercase()
                        .chars()
                        .filter(|char| char.is_alphabetic())
                        .collect::<String>(),
                );
            }
        }
    }
}

impl From<ServerMessage> for Message {
    fn from(msg: ServerMessage) -> Self {
        let serialized = serde_json::to_string(&msg).unwrap();

        Self::Text(serialized)
    }
}
