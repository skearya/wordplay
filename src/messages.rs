use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    game::{
        anagrams::messages::AnagramsSettings,
        messages::{ClientGame, GameMessage, ServerGame},
        word_bomb::messages::WordBombSettings,
    },
    lobby::messages::{ClientLobby, LobbyMessage, ServerLobby},
    room::general::messages::{ClientGeneral, GeneralMessage, ServerGeneral},
};

#[derive(Deserialize, TS)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
#[ts(export)]
pub enum ClientMessage {
    General(ClientGeneral),
    Lobby(ClientLobby),
    InGame(ClientGame),
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
    InGame(ServerGame),
}

pub enum RoomMessage {
    /// Rooms also recieve client messages through the same channel as other room messages.
    Client {
        uuid: Uuid,
        message: ClientMessage,
    },
    /// Core room functionality.
    General(GeneralMessage),
    Lobby(LobbyMessage),
    InGame(GameMessage),
}

#[derive(Serialize, Deserialize, TS, Clone, Copy, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoomSettings {
    pub public: bool,
    pub word_bomb: WordBombSettings,
    pub anagrams: AnagramsSettings,
}
