pub mod messages {
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

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
            reason: &'static str,
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

    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct WordBombPostGameInfo {
        winner: Uuid,
        mins_elapsed: f32,
        words_used: usize,
        fastest_guesses: Vec<(Uuid, f32)>,
        longest_words: Vec<(Uuid, String)>,
        avg_wpms: Vec<(Uuid, f32)>,
        avg_word_lengths: Vec<(Uuid, f32)>,
    }
}

use crate::{
    games::word_bomb::messages::{ClientWordBomb, ServerWordBomb, WordBombMessage},
    in_game::messages::PostGameInfo,
    messages::RoomSettings,
    room::{
        handler::Handler,
        messenger::{ClientMessenger, RoomMessenger},
    },
};

pub struct WordBomb;

impl Handler<PostGameInfo> for WordBomb {
    type ClientMessage = ClientWordBomb;
    type ServerMessage = ServerWordBomb;
    type RoomMessage = WordBombMessage;

    fn new(settings: &RoomSettings) -> Self {
        todo!()
    }

    fn handle_client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: (uuid::Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<PostGameInfo>> {
        todo!()
    }

    fn handle(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<PostGameInfo>> {
        todo!()
    }
}
