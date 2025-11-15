use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    game::{
        anagrams::messages::{AnagramsMessage, AnagramsSettings, ClientAnagrams, ServerAnagrams},
        messages::{ClientGame, GameMessage, GameState, ServerGame},
        word_bomb::messages::{ClientWordBomb, ServerWordBomb, WordBombMessage, WordBombSettings},
    },
    general::messages::{ClientGeneral, ServerGeneral},
    lobby::messages::{ClientLobby, LobbyMessage, LobbyState, ServerLobby},
    room::{
        clients::Client,
        messages::{CoreMessage, ServerCore},
    },
};

#[derive(Deserialize, TS)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
#[ts(export)]
pub enum ClientMessage {
    General(ClientGeneral),
    Lobby(ClientLobby),
    Game(ClientGame),
    WordBomb(ClientWordBomb),
    Anagrams(ClientAnagrams),
}

#[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
#[derive(Serialize, TS)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum ServerMessage {
    /// First message sent after establishing connection, sent only once.
    Info {
        /// Joined client's designated UUID.
        uuid: Uuid,
        /// Room clients.
        clients: HashMap<Uuid, ServerClient>,
        /// Room and game settings.
        settings: RoomSettings,
        /// State variant data of the room (lobby | game -> (game kind)).
        state: Box<ServerState>,
    },
    /// Core room functionality related messages.
    Core(ServerCore),
    /// General messages. Can be sent from any state.
    General(ServerGeneral),
    /// All lobby messages.
    Lobby(ServerLobby),
    /// All general in-game messages.
    Game(ServerGame),
    /// Word Bomb messages.
    WordBomb(ServerWordBomb),
    /// Anagrams messages.
    Anagrams(ServerAnagrams),
}

#[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ServerClient {
    /// Client username.
    pub username: String,
    /// URL to account avatar.
    pub avatar_url: Option<String>,
    /// Is client currently connected? (Possibly not in game).
    pub connected: bool,
}

#[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
#[derive(Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
/// Room variant state sent to clients when they join.
pub enum ServerState {
    Lobby(LobbyState),
    Game(GameState),
}

pub enum RoomMessage {
    Core(CoreMessage),
    Lobby(LobbyMessage),
    Game(GameMessage),
}

#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Serialize, Deserialize, TS, Clone, Copy, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoomSettings {
    pub public: bool,
    pub owner: Uuid,
    pub game: GameType,
    pub word_bomb: WordBombSettings,
    pub anagrams: AnagramsSettings,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Serialize, Deserialize, TS, Clone, Copy, Default)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum GameType {
    #[default]
    WordBomb,
    Anagrams,
}

impl From<&Client> for ServerClient {
    fn from(client: &Client) -> Self {
        Self {
            username: client.username.clone(),
            avatar_url: None,
            connected: client.connected(),
        }
    }
}

impl From<ServerCore> for ServerMessage {
    fn from(value: ServerCore) -> Self {
        Self::Core(value)
    }
}

impl From<ServerGeneral> for ServerMessage {
    fn from(value: ServerGeneral) -> Self {
        Self::General(value)
    }
}

impl From<ServerLobby> for ServerMessage {
    fn from(value: ServerLobby) -> Self {
        Self::Lobby(value)
    }
}

impl From<ServerGame> for ServerMessage {
    fn from(value: ServerGame) -> Self {
        Self::Game(value)
    }
}

impl From<ServerWordBomb> for ServerMessage {
    fn from(value: ServerWordBomb) -> Self {
        ServerMessage::WordBomb(value)
    }
}

impl From<ServerAnagrams> for ServerMessage {
    fn from(value: ServerAnagrams) -> Self {
        ServerMessage::Anagrams(value)
    }
}

impl From<CoreMessage> for RoomMessage {
    fn from(value: CoreMessage) -> Self {
        Self::Core(value)
    }
}

impl From<LobbyMessage> for RoomMessage {
    fn from(value: LobbyMessage) -> Self {
        Self::Lobby(value)
    }
}

impl From<GameMessage> for RoomMessage {
    fn from(value: GameMessage) -> Self {
        Self::Game(value)
    }
}

impl From<WordBombMessage> for RoomMessage {
    fn from(value: WordBombMessage) -> Self {
        RoomMessage::Game(GameMessage::WordBombMessage(value))
    }
}

impl From<AnagramsMessage> for RoomMessage {
    fn from(value: AnagramsMessage) -> Self {
        RoomMessage::Game(GameMessage::AnagramsMessage(value))
    }
}

impl From<LobbyState> for ServerState {
    fn from(value: LobbyState) -> Self {
        Self::Lobby(value)
    }
}

impl From<GameState> for ServerState {
    fn from(value: GameState) -> Self {
        Self::Game(value)
    }
}
