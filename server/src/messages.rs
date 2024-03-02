use crate::models::AppState;

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
        // rejoin_token
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
    GameStarted {
        prompt: String,
        turn: Uuid,
        players: Vec<PlayerData>,
    },
    InvalidWord {
        uuid: Uuid,
    },
    NewPrompt {
        timed_out: bool,
        prompt: String,
        turn: Uuid,
        // TODO: probably just send usernames too?
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
        ready_players: Vec<PlayerInfo>,
    },
    InGame {
        prompt: String,
        turn: Uuid,
        players: Vec<PlayerData>,
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
    pub lives: u8,
}

impl ClientMessage {
    pub fn handle(self, state: &AppState, room: &str, username: &str, uuid: Uuid) {
        match self {
            ClientMessage::Ready => {
                state.client_ready(room, (uuid, username.to_owned()));
            }
            ClientMessage::Unready => todo!(),
            ClientMessage::ChatMessage { content } => {
                state.broadcast(
                    room,
                    ServerMessage::ChatMessage {
                        author: username.to_owned(),
                        content,
                    },
                );
            }
            ClientMessage::Guess { word } => {
                state.client_guess(room, uuid, &word);
            }
        }
    }
}
