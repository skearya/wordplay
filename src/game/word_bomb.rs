pub mod messages {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    #[cfg_attr(test, derive(Debug, PartialEq))]
    #[derive(Serialize, Deserialize, TS, Clone, Copy)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct WordBombSettings {
        min_wpm: usize,
    }

    impl Default for WordBombSettings {
        fn default() -> Self {
            Self { min_wpm: 300 }
        }
    }

    #[derive(Deserialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ClientWordBomb {
        Input { input: String },
        Guess { word: String },
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ServerWordBomb {
        /// Broadcasted when the currently active player is typing.
        Input { input: String },
        Valid {
            /// Current player's guess that was valid.
            guess: String,
            /// True if the current player has used up all letters.
            life: bool,
            /// New prompt.
            prompt: String,
            /// Player UUID of new turn.
            turn: Uuid,
        },
        Invalid {
            /// Reason for invalid guess (ex: "guess doesn't include prompt")
            reason: String,
        },
        /// Broadcasted previously active player failed to come up with a valid guess.
        Exploded {
            /// New prompt.
            prompt: String,
            /// Player UUID of new turn.
            turn: Uuid,
        },
    }

    pub enum WordBombMessage {}

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct WordBombState {
        players: HashMap<Uuid, WordBombPlayer>,
        turn: Uuid,
        prompt: String,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct WordBombPlayer {
        input: String,
        lives: u8,
        used_letters: Vec<char>,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct WordBombPostGame {
        winner: Uuid,
        mins_elapsed: f32,
        words_used: usize,
        fastest_guesses: Vec<(Uuid, f32)>,
        longest_words: Vec<(Uuid, String)>,
        avg_wpms: Vec<(Uuid, f32)>,
        avg_word_lengths: Vec<(Uuid, f32)>,
    }
}

use uuid::Uuid;

use crate::{
    game::word_bomb::messages::{
        ClientWordBomb, ServerWordBomb, WordBombMessage, WordBombPostGame, WordBombSettings,
        WordBombState,
    },
    room::{
        handler::GameHandler,
        messenger::{ClientMessenger, RoomMessenger},
    },
};

pub struct WordBomb;

impl GameHandler for WordBomb {
    type Settings = WordBombSettings;
    type ClientMessage = ClientWordBomb;
    type ServerMessage = ServerWordBomb;
    type RoomMessage = WordBombMessage;
    type StateMessage = WordBombState;
    type PostGameMessage = WordBombPostGame;

    fn new(settings: &WordBombSettings, players: &[Uuid]) -> Self {
        todo!()
    }

    fn handle_client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<Self::PostGameMessage>> {
        todo!()
    }

    fn handle_message(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<Self::PostGameMessage>> {
        todo!()
    }

    fn state(&self) -> Self::StateMessage {
        todo!()
    }

    fn end(&mut self) -> Self::PostGameMessage {
        todo!()
    }
}
