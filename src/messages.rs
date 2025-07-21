use axum::extract::ws;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    games::word_bomb::messages::WordBombSettings,
    in_game::messages::{ClientInGame, ServerInGame},
    lobby::messages::{ClientLobby, LobbyMessage, ServerLobby},
    room::general::messages::{ClientGeneral, ServerGeneral},
};

#[derive(Deserialize, TS)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
#[ts(export)]
pub enum ClientMessage {
    General(ClientGeneral),
    Lobby(ClientLobby),
    InGame(ClientInGame),
}

#[derive(Serialize, TS)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
#[ts(export)]
pub enum ServerMessage {
    /// Can be sent from any state.
    General(ServerGeneral),
    /// All lobby messages.
    Lobby(ServerLobby),
    /// All in-game messages.
    InGame(ServerInGame),
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
    Lobby(LobbyMessage),
}

#[derive(Serialize, Deserialize, TS, Clone, Copy)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoomSettings {
    pub public: bool,
    pub word_bomb: WordBombSettings,
}
