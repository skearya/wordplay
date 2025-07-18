use axum::extract::ws;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    games::word_bomb::messages::{ClientWordBomb, ServerWordBomb, WordBombSettings},
    general::messages::{ClientGeneral, ServerGeneral},
    in_game::messages::{ClientInGame, ServerInGame},
    lobby::messages::{ClientLobby, ServerLobby},
};

#[derive(Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum ClientMessage {
    General(ClientGeneral),
    Lobby(ClientLobby),
    InGame(ClientInGame),
    WordBomb(ClientWordBomb),
}

#[derive(Serialize, TS)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
#[ts(export)]
pub enum ServerMessage {
    /// Can be sent from any state.
    General(ServerGeneral),
    /// All lobby messages.
    Lobby(ServerLobby),
    /// All general in-game messages.
    InGame(ServerInGame),
    /// All Word Bomb messages.
    WordBomb(ServerWordBomb),
}

pub enum RoomMessage {
    Joined {
        uuid: Uuid,
        sender: mpsc::UnboundedSender<ws::Message>,
    },
    Left {
        uuid: Uuid,
    },
    Client {
        uuid: Uuid,
        message: ClientMessage,
    },
    CloseCheck,
}

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoomSettings {
    word_bomb: WordBombSettings,
}
