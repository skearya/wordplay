pub mod anagrams;
pub mod word_bomb;

pub mod messages {
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::game::{
        anagrams::messages::{
            AnagramsMessage, AnagramsPostGame, AnagramsState, ClientAnagrams, ServerAnagrams,
        },
        word_bomb::messages::{
            ClientWordBomb, ServerWordBomb, WordBombMessage, WordBombPostGame, WordBombState,
        },
    };

    #[derive(Deserialize, TS)]
    #[serde(tag = "kind", content = "data", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ClientGame {
        WordBomb(ClientWordBomb),
        Anagrams(ClientAnagrams),
        /// Request to end the game early. Starts a vote.
        EndRequest,
        /// Sent only by the room owner. Immediately ends the game.
        ForceEnd,
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
    pub enum ServerGame {
        WordBomb(ServerWordBomb),
        Anagrams(ServerAnagrams),
        /// Broadcasted when a player requests to end the game early.
        EndRequest {
            uuid: Uuid,
        },
        /// Broadcasted when the current game has ended.
        Ended {
            post_game_info: PostGameInfo,
            /// Is `Some` with a random client's uuid if the previous room owner
            /// left during game and hasn't come back.
            new_owner: Option<Uuid>,
        },
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS, Clone)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum PostGameInfo {
        WordBomb(WordBombPostGame),
        Anagrams(AnagramsPostGame),
    }

    pub enum GameMessage {
        WordBomb(WordBombMessage),
        Anagrams(AnagramsMessage),
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct GameState {
        // Specific game type state (ex: word bomb).
        pub state: GameVariantState,
        /// UUIDs of players requesting to end the current game.
        pub requesting_end: Vec<Uuid>,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum GameVariantState {
        WordBomb(WordBombState),
        Anagrams(AnagramsState),
    }
}

use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    game::{
        anagrams::{Anagrams, messages::ServerAnagrams},
        messages::{
            ClientGame, GameMessage, GameState, GameVariantState, PostGameInfo, ServerGame,
        },
        word_bomb::{WordBomb, messages::ServerWordBomb},
    },
    messages::{GameType, RoomSettings},
    room::{
        StateChange,
        handler::{GameHandler, Handler},
        messenger::{ClientMessenger, ClientUtils, client_submessenger},
        sender::{AnagramsSender, GameSender, WordBombSender},
    },
};

enum State {
    WordBomb(WordBomb),
    Anagrams(Anagrams),
}

impl State {
    fn try_word_bomb(&mut self) -> anyhow::Result<&mut WordBomb> {
        if let Self::WordBomb(v) = self {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("expected word bomb"))
        }
    }

    fn try_anagrams(&mut self) -> anyhow::Result<&mut Anagrams> {
        if let Self::Anagrams(v) = self {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("expected anagrams"))
        }
    }

    fn state(&self) -> GameVariantState {
        match self {
            Self::WordBomb(word_bomb) => GameVariantState::WordBomb(word_bomb.state()),
            Self::Anagrams(anagrams) => GameVariantState::Anagrams(anagrams.state()),
        }
    }

    fn end(&mut self) {
        match self {
            Self::WordBomb(word_bomb) => word_bomb.abort(),
            Self::Anagrams(anagrams) => anagrams.abort(),
        }
    }
}

pub struct Game {
    room: GameSender,
    state: State,
    rejoin_tokens: HashMap<Uuid, Uuid>,
    requesting_end: Vec<Uuid>,
}

impl Game {
    pub fn new(room: GameSender, settings: &RoomSettings, players: &[Uuid]) -> Self {
        Self {
            state: match settings.game {
                GameType::WordBomb => State::WordBomb(WordBomb::new(
                    WordBombSender::new(room.clone()),
                    &settings.word_bomb,
                    players,
                )),
                GameType::Anagrams => State::Anagrams(Anagrams::new(
                    AnagramsSender::new(room.clone()),
                    &settings.anagrams,
                    players,
                )),
            },
            room,
            rejoin_tokens: players.iter().map(|&uuid| (uuid, Uuid::new_v4())).collect(),
            requesting_end: vec![],
        }
    }

    pub fn lookup_rejoin_token(&self, rejoin_token: Uuid) -> Option<Uuid> {
        self.rejoin_tokens.get(&rejoin_token).copied()
    }

    fn handle_post_game_info(
        settings: &mut RoomSettings,
        clients: &mut (impl ClientMessenger<ServerGame> + ClientUtils),
        post_game_info: Option<PostGameInfo>,
    ) -> StateChange {
        match post_game_info {
            Some(post_game_info) => {
                clients.keep_connected();

                let new_owner = if clients.get(settings.owner).is_none() {
                    let random = *clients.random().0;
                    settings.owner = random;

                    Some(random)
                } else {
                    None
                };

                clients.broadcast(ServerGame::Ended {
                    post_game_info: post_game_info.clone(),
                    new_owner,
                });

                StateChange::Lobby(Some(post_game_info))
            }
            None => StateChange::None,
        }
    }
}

impl Handler for Game {
    type ClientMessage = ClientGame;
    type ServerMessage = ServerGame;
    type RoomMessage = GameMessage;
    type StateMessage = GameState;

    fn client(
        &mut self,
        settings: &mut RoomSettings,
        mut clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils,
        (uuid, message): (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<StateChange> {
        let post_game_info = match message {
            ClientGame::WordBomb(message) => self
                .state
                .try_word_bomb()?
                .client(
                    client_submessenger!(&mut clients, ServerGame::WordBomb(ServerWordBomb)),
                    (uuid, message),
                )?
                .map(PostGameInfo::WordBomb),
            ClientGame::Anagrams(message) => self
                .state
                .try_anagrams()?
                .client(
                    client_submessenger!(&mut clients, ServerGame::Anagrams(ServerAnagrams)),
                    (uuid, message),
                )?
                .map(PostGameInfo::Anagrams),
            ClientGame::EndRequest => {
                self.requesting_end.push(uuid);

                // If everyone in game has requested to end early, end.
                if self.requesting_end.len() == self.rejoin_tokens.len() {
                    return Ok(StateChange::Lobby(None));
                } else {
                    clients.broadcast(ServerGame::EndRequest { uuid });
                    None
                }
            }
            ClientGame::ForceEnd => return Ok(StateChange::Lobby(None)),
        };

        Ok(Game::handle_post_game_info(
            settings,
            &mut clients,
            post_game_info,
        ))
    }

    fn room(
        &mut self,
        settings: &mut RoomSettings,
        mut clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils,
        message: Self::RoomMessage,
    ) -> anyhow::Result<StateChange> {
        let post_game_info = match message {
            GameMessage::WordBomb(message) => self
                .state
                .try_word_bomb()?
                .room(
                    client_submessenger!(&mut clients, ServerGame::WordBomb(ServerWordBomb)),
                    message,
                )?
                .map(PostGameInfo::WordBomb),
            GameMessage::Anagrams(message) => self
                .state
                .try_anagrams()?
                .room(
                    client_submessenger!(&mut clients, ServerGame::Anagrams(ServerAnagrams)),
                    message,
                )?
                .map(PostGameInfo::Anagrams),
        };

        Ok(Game::handle_post_game_info(
            settings,
            &mut clients,
            post_game_info,
        ))
    }

    fn state(&self) -> Self::StateMessage {
        GameState {
            state: self.state.state(),
            requesting_end: self.requesting_end.clone(),
        }
    }

    fn abort(&mut self) {
        self.state.end();
    }
}
