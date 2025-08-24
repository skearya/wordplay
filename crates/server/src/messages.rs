use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    game::{
        anagrams::messages::AnagramsSettings,
        messages::{ClientGame, GameMessage, GameState, ServerGame},
        word_bomb::messages::WordBombSettings,
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
    /// Can be sent from any state.
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

pub enum RoomMessage {
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
