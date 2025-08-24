pub mod messages {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    #[cfg_attr(test, derive(Debug, PartialEq))]
    #[derive(Serialize, Deserialize, TS, Clone, Copy)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AnagramsSettings {}

    impl Default for AnagramsSettings {
        fn default() -> Self {
            Self {}
        }
    }

    #[derive(Deserialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ClientAnagrams {
        Guess { word: String },
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ServerAnagrams {
        Valid {
            uuid: Uuid,
            points: u32,
        },
        Invalid {
            /// Reason for invalid guess (ex: "guess doesn't include prompt")
            reason: String,
        },
    }

    pub enum AnagramsMessage {}

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AnagramsState {
        players: HashMap<Uuid, AnagramsPlayer>,
        anagram: String,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AnagramsPlayer {
        used_words: Vec<String>,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS, Clone)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AnagramsPostGame {
        original_word: String,
        leaderboard: Vec<(Uuid, u32)>,
        used_words: Vec<(Uuid, Vec<String>)>,
        total_guesses: Vec<(Uuid, u32)>,
    }
}

use uuid::Uuid;

use crate::{
    game::anagrams::messages::{
        AnagramsMessage, AnagramsPostGame, AnagramsSettings, AnagramsState, ClientAnagrams,
        ServerAnagrams,
    },
    room::{
        handler::GameHandler,
        messenger::{ClientMessenger, RoomMessenger},
    },
};

pub struct Anagrams;

impl GameHandler for Anagrams {
    type GameSettings = AnagramsSettings;
    type ClientMessage = ClientAnagrams;
    type ServerMessage = ServerAnagrams;
    type RoomMessage = AnagramsMessage;
    type StateMessage = AnagramsState;
    type PostGameMessage = AnagramsPostGame;

    fn new(
        settings: &Self::GameSettings,
        players: &[Uuid],
        room: impl RoomMessenger<Self::RoomMessage>,
    ) -> Self {
        todo!()
    }

    fn state(&self) -> Self::StateMessage {
        todo!()
    }

    fn handle_client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        message: (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<Self::PostGameMessage>> {
        todo!()
    }

    fn handle_message(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<Self::PostGameMessage>> {
        todo!()
    }

    fn abort(&mut self) {
        todo!()
    }
}
