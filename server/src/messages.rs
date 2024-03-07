use std::collections::HashSet;

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
        rejoin_token: Uuid,
        prompt: String,
        turn: Uuid,
        players: Vec<PlayerData>,
    },
    // send chat messages for join/leave
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
    },
    NewPrompt {
        life_change: i8,
        prompt: String,
        turn: Uuid,
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
    InProgress(u8),
    Stopped,
}

impl ClientMessage {
    pub fn handle(self, state: &AppState, room: &str, uuid: Uuid) {
        match self {
            ClientMessage::Ready => {
                state.client_ready(room, uuid);
            }
            ClientMessage::Unready => todo!(),
            ClientMessage::ChatMessage { content } => {
                state.client_chat_message(room, uuid, content);
            }
            ClientMessage::Input { input } => state.client_input_update(room, uuid, input),
            ClientMessage::Guess { word } => {
                // maybe also get rid of non letters
                state.client_guess(room, uuid, word.to_lowercase().trim());
            }
        }
    }
}
