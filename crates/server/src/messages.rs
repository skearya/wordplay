use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    game::{
        anagrams::messages::{AnagramsMessage, AnagramsSettings, ServerAnagrams},
        messages::{ClientGame, GameMessage, GameState, PostGameInfo, ServerGame},
        word_bomb::messages::{ServerWordBomb, WordBombMessage, WordBombSettings},
    },
    general::messages::{ClientGeneral, ServerGeneral},
    lobby::messages::{ClientLobby, LobbyMessage, LobbyState, ServerLobby},
    room::clients::Client,
};

#[derive(Deserialize, TS)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
#[ts(export)]
pub enum ClientMessage {
    General(ClientGeneral),
    Lobby(ClientLobby),
    Game(ClientGame),
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
    /// Broadcasted when a client joins/rejoins.
    Join { uuid: Uuid, client: ServerClient },
    /// Broadcasted when a client leaves.
    /// `new_owner` will only be some if the owner leaves in lobby, if the owner
    /// leaves in game, they will still be owner and have the chance to rejoin.
    /// If they don't rejoin before the game ends, the game ending message will
    /// broadcast the new owner.
    Leave { uuid: Uuid, new_owner: Option<Uuid> },
    /// Sent when the game (based on room settings) has started.
    GameStart {
        /// Contains the player's rejoin token. Is `None` if client is spectating.
        rejoin_token: Option<Uuid>,
        state: GameState,
    },
    /// Broadcasted when the current game has ended.
    GameEnd {
        post_game_info: Option<PostGameInfo>,
        /// Is `Some` with a random client's uuid if the previous room owner
        /// left during game and hasn't come back.
        new_owner: Option<Uuid>,
    },
    /// General messages. Can be sent from any state.
    General(ServerGeneral),
    /// All lobby messages.
    Lobby(ServerLobby),
    /// All in-game messages.
    Game(ServerGame),
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

pub enum CoreMessage {
    Join {
        uuid: Uuid,
        client: Client,
    },
    JoinWithRejoinToken {
        rejoin_token: Uuid,
        client: Client,
        /// Response to the socket task that tried joining containing the client's designated UUID.
        /// If the `rejoin_token` was valid, the client will given the previously associated UUID.
        /// Otherwise, the client will be given a randomly generated UUID.
        response: oneshot::Sender<Uuid>,
    },
    Leave {
        uuid: Uuid,
        socket: Uuid,
    },
    /// Rooms also recieve client messages through the same channel as other room messages.
    Client {
        uuid: Uuid,
        message: ClientMessage,
    },
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
        }
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
        ServerGame::WordBomb(value).into()
    }
}

impl From<ServerAnagrams> for ServerMessage {
    fn from(value: ServerAnagrams) -> Self {
        ServerGame::Anagrams(value).into()
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
        GameMessage::WordBomb(value).into()
    }
}

impl From<AnagramsMessage> for RoomMessage {
    fn from(value: AnagramsMessage) -> Self {
        GameMessage::Anagrams(value).into()
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
