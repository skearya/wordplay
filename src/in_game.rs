pub mod messages {
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::games::word_bomb::messages::{ClientWordBomb, ServerWordBomb, WordBombPostGameInfo};

    #[derive(Deserialize, TS)]
    #[serde(tag = "kind", content = "data", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ClientInGame {
        WordBomb(ClientWordBomb),
        /// Request to end the game early. Starts a vote.
        EndRequest,
        /// Sent only by the room owner. Immediately ends the game.
        ForceEnd,
    }

    #[derive(Serialize, TS)]
    #[serde(
        tag = "kind",
        rename_all = "camelCase",
        rename_all_fields = "camelCase"
    )]
    #[ts(export)]
    pub enum ServerInGame {
        WordBomb(ServerWordBomb),
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

    #[derive(Serialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum PostGameInfo {
        WordBomb(WordBombPostGameInfo),
    }

    pub enum InGameMessage {}
}

use uuid::Uuid;

use crate::{
    games::word_bomb::WordBomb,
    in_game::messages::{ClientInGame, InGameMessage, ServerInGame},
    messages::RoomSettings,
    room::{
        handler::Handler,
        messenger::{ClientMessenger, RoomMessenger},
        state::State,
    },
};

enum Game {
    WordBomb(WordBomb),
}

pub struct InGame {
    game: Game,
    requesting_end: Vec<Uuid>,
}

impl InGame {
    pub fn new() -> Self {
        Self {
            game: todo!(),
            requesting_end: vec![],
        }
    }
}

impl Handler<State> for InGame {
    type ClientMessage = ClientInGame;
    type ServerMessage = ServerInGame;
    type RoomMessage = InGameMessage;

    fn new(settings: &RoomSettings) -> Self {
        Self::new()
    }

    fn client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        (uuid, message): (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<State>> {
        match message {
            ClientInGame::WordBomb(client_word_bomb) => todo!(),
            ClientInGame::EndRequest => todo!(),
            ClientInGame::ForceEnd => todo!(),
        }
    }

    fn room(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<State>> {
        todo!()
    }
}
