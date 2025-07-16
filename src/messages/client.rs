use serde::Deserialize;
use uuid::Uuid;

use crate::{games::word_bomb::messages::ClientWordBomb, messages::shared::RoomSettings};

#[derive(Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum ClientMessage {
    General(ClientGeneral),
    Lobby(ClientLobby),
    InGame(ClientInGame),
    WordBomb(ClientWordBomb),
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ClientGeneral {
    Ping { timestamp: u64 },
    ChatMessage { content: String },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ClientLobby {
    RoomSettings(RoomSettings),
    Ready,
    StartEarly,
    Unready,
    PracticeRequest,
    PracticeSubmission { prompt: String, input: String },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ClientInGame {}
