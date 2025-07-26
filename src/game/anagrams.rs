use crate::{
    game::{anagrams::messages::{AnagramsMessage, ClientAnagrams, ServerAnagrams}, messages::PostGameInfo},
    messages::RoomSettings,
    room::{
        handler::Handler,
        messenger::{ClientMessenger, RoomMessenger},
    },
};

pub mod messages {
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

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
            reason: &'static str,
        },
    }

    pub enum AnagramsMessage {}

    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AnagramsPostGameInfo {
        original_word: String,
        leaderboard: Vec<(Uuid, u32)>,
        used_words: Vec<(Uuid, Vec<String>)>,
        total_guesses: Vec<(Uuid, u32)>,
    }
}

pub struct Anagrams;

impl Handler<PostGameInfo> for Anagrams {
    type ClientMessage = ClientAnagrams;
    type ServerMessage = ServerAnagrams;
    type RoomMessage = AnagramsMessage;

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

    fn handle_message(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<PostGameInfo>> {
        todo!()
    }

    fn end(&mut self) {
        todo!()
    }
}
