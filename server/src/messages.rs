use crate::state::{
    games::{anagrams, word_bomb},
    room::RoomSettings,
};

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
    Ready,
    StartEarly,
    Unready,
    RoomSettings(RoomSettings),
    ChatMessage { content: String },
    WordBombInput { input: String },
    WordBombGuess { word: String },
    AnagramsGuess { word: String },
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
    GameStarted {
        rejoin_token: Option<Uuid>,
        game: RoomStateInfo,
    },
    GameEnded {
        winner: Uuid,
        new_room_owner: Option<Uuid>,
    },

    // word bomb
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

    // anagrams
    AnagramsInvalidGuess {
        reason: anagrams::GuessInfo,
    },
    AnagramsCorrectGuess {
        uuid: Uuid,
        guess: String,
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
        used_words: Option<Vec<String>>,
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

impl From<ServerMessage> for Message {
    fn from(msg: ServerMessage) -> Self {
        let serialized = serde_json::to_string(&msg).unwrap();
        Self::Text(serialized)
    }
}
