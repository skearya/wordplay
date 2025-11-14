pub mod messages {
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::game::{
        anagrams::messages::{AnagramsMessage, AnagramsPostGame, AnagramsState, ClientAnagrams},
        word_bomb::messages::{ClientWordBomb, WordBombMessage, WordBombPostGame, WordBombState},
    };

    #[derive(Deserialize, TS)]
    #[serde(tag = "kind", content = "data", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ClientGame {
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
        /// Broadcasted when a player requests to end the game early.
        EndRequest { uuid: Uuid },
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
        ClientWordBomb((Uuid, ClientWordBomb)),
        ClientAnagrams((Uuid, ClientAnagrams)),
        WordBombMessage(WordBombMessage),
        AnagramsMessage(AnagramsMessage),
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS, Clone)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct GameState {
        // Specific game type state (ex: word bomb).
        pub state: GameVariantState,
        /// UUIDs of players requesting to end the current game.
        pub requesting_end: Vec<Uuid>,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS, Clone)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum GameVariantState {
        WordBomb(WordBombState),
        Anagrams(AnagramsState),
    }
}

pub mod anagrams;
pub mod word_bomb;

use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    game::{
        anagrams::Anagrams,
        messages::{ClientGame, GameMessage, GameState, GameVariantState, ServerGame},
        word_bomb::WordBomb,
    },
    messages::{GameType, RoomSettings},
    room::{StateChange, clients::Clients, context::Context, sender::RoomSender},
};

pub struct GameContext<'a> {
    pub room: &'a RoomSender,
    pub clients: &'a Clients,
    pub settings: &'a RoomSettings,
}

impl<'a> GameContext<'a> {
    pub fn new(room: &'a RoomSender, clients: &'a Clients, settings: &'a RoomSettings) -> Self {
        Self {
            room,
            clients,
            settings,
        }
    }
}

pub trait GameHandler {
    type ClientMessage;
    type SelfMessage;
    type Outcome: Into<StateChange>;
    type Snapshot: Into<GameVariantState>;

    fn on_client_message(
        &mut self,
        ctx: GameContext,
        message: (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Self::Outcome>;

    fn on_self_message(
        &mut self,
        ctx: GameContext,
        message: Self::SelfMessage,
    ) -> anyhow::Result<Self::Outcome>;

    fn on_abort(&mut self);

    fn snapshot(&self) -> Self::Snapshot;
}

enum State {
    WordBomb(WordBomb),
    Anagrams(Anagrams),
}

impl State {
    pub fn try_word_bomb(&mut self) -> anyhow::Result<&mut WordBomb> {
        if let Self::WordBomb(v) = self {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("expected word bomb"))
        }
    }

    pub fn try_anagrams(&mut self) -> anyhow::Result<&mut Anagrams> {
        if let Self::Anagrams(v) = self {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("expected anagrams"))
        }
    }

    fn on_abort(&mut self) {
        match self {
            Self::WordBomb(word_bomb) => word_bomb.on_abort(),
            Self::Anagrams(anagrams) => anagrams.on_abort(),
        }
    }

    fn snapshot(&self) -> GameVariantState {
        match self {
            Self::WordBomb(word_bomb) => GameVariantState::WordBomb(word_bomb.snapshot()),
            Self::Anagrams(anagrams) => GameVariantState::Anagrams(anagrams.snapshot()),
        }
    }
}

pub struct Game {
    state: State,
    rejoin_tokens: HashMap<Uuid, Uuid>,
    requesting_end: Vec<Uuid>,
}

impl Game {
    pub fn new(ctx: Context, players: &[Uuid]) -> Self {
        Self {
            state: match ctx.settings.game {
                GameType::WordBomb => State::WordBomb(WordBomb::new(
                    GameContext::new(ctx.room, ctx.clients, ctx.settings),
                    players,
                )),
                GameType::Anagrams => State::Anagrams(Anagrams::new(
                    GameContext::new(ctx.room, ctx.clients, ctx.settings),
                    players,
                )),
            },
            rejoin_tokens: players.iter().map(|&uuid| (uuid, Uuid::new_v4())).collect(),
            requesting_end: vec![],
        }
    }

    pub fn rejoin_tokens(&self) -> &HashMap<Uuid, Uuid> {
        &self.rejoin_tokens
    }
}

impl Game {
    pub fn on_client_message(
        &mut self,
        ctx: Context,
        (uuid, message): (Uuid, ClientGame),
    ) -> anyhow::Result<StateChange> {
        let outcome = match message {
            ClientGame::EndRequest => {
                if !self.rejoin_tokens.contains_key(&uuid) || self.requesting_end.contains(&uuid) {
                    return Ok(StateChange::None);
                }

                self.requesting_end.push(uuid);

                // If everyone in game has requested to end early, end.
                if self.requesting_end.len() == self.rejoin_tokens.len() {
                    return Ok(StateChange::Lobby(None));
                }

                ctx.clients.broadcast(ServerGame::EndRequest { uuid });

                None
            }
            ClientGame::ForceEnd => return Ok(StateChange::Lobby(None)),
        };

        Ok(outcome.map_or(StateChange::None, |info| StateChange::Lobby(Some(info))))
    }

    pub fn on_self_message(
        &mut self,
        ctx: Context,
        message: GameMessage,
    ) -> anyhow::Result<StateChange> {
        let outcome = match message {
            GameMessage::ClientWordBomb(message) => self
                .state
                .try_word_bomb()?
                .on_client_message(
                    GameContext::new(ctx.room, ctx.clients, ctx.settings),
                    message,
                )?
                .into(),
            GameMessage::ClientAnagrams(message) => self
                .state
                .try_anagrams()?
                .on_client_message(
                    GameContext::new(ctx.room, ctx.clients, ctx.settings),
                    message,
                )?
                .into(),
            GameMessage::WordBombMessage(message) => self
                .state
                .try_word_bomb()?
                .on_self_message(
                    GameContext::new(ctx.room, ctx.clients, ctx.settings),
                    message,
                )?
                .into(),
            GameMessage::AnagramsMessage(message) => self
                .state
                .try_anagrams()?
                .on_self_message(
                    GameContext::new(ctx.room, ctx.clients, ctx.settings),
                    message,
                )?
                .into(),
        };

        Ok(outcome)
    }

    pub fn on_client_leave(&mut self, ctx: Context, uuid: Uuid) {
        ctx.clients.disconnect(uuid);
    }

    pub fn on_abort(&mut self) {
        self.state.on_abort();
    }

    pub fn snapshot(&self) -> GameState {
        GameState {
            state: self.state.snapshot(),
            requesting_end: self.requesting_end.clone(),
        }
    }
}
