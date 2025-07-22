pub mod messages {
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::games::{
        anagrams::messages::{
            AnagramsMessage, AnagramsPostGameInfo, ClientAnagrams, ServerAnagrams,
        },
        word_bomb::messages::{
            ClientWordBomb, ServerWordBomb, WordBombMessage, WordBombPostGameInfo,
        },
    };

    #[derive(Deserialize, TS)]
    #[serde(tag = "kind", content = "data", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ClientInGame {
        WordBomb(ClientWordBomb),
        Anagrams(ClientAnagrams),
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

    #[derive(Serialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum PostGameInfo {
        WordBomb(WordBombPostGameInfo),
        Anagrams(AnagramsPostGameInfo),
    }

    pub enum InGameMessage {
        WordBomb(WordBombMessage),
        Anagrams(AnagramsMessage),
    }
}

use uuid::Uuid;

use crate::{
    games::{
        anagrams::{
            Anagrams,
            messages::{AnagramsMessage, ServerAnagrams},
        },
        word_bomb::{
            WordBomb,
            messages::{ClientWordBomb, ServerWordBomb, WordBombMessage},
        },
    },
    in_game::messages::{ClientInGame, InGameMessage, ServerInGame},
    messages::RoomSettings,
    room::{
        handler::Handler,
        messenger::{ClientMessenger, RoomMessenger, client_submessenger, room_submessenger},
        state::State,
    },
};

enum Game {
    WordBomb(WordBomb),
    Anagrams(Anagrams),
}

impl Game {
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
}

pub struct InGame {
    game: Game,
    requesting_end: Vec<Uuid>,
}

impl Handler<State> for InGame {
    type ClientMessage = ClientInGame;
    type ServerMessage = ServerInGame;
    type RoomMessage = InGameMessage;

    fn new(settings: &RoomSettings) -> Self {
        Self {
            game: todo!(),
            requesting_end: vec![],
        }
    }

    fn handle_client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        (uuid, message): (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<State>> {
        match message {
            ClientInGame::WordBomb(message) => {
                let word_bomb = self.game.try_word_bomb()?;

                word_bomb.handle_client(
                    client_submessenger!(clients, ServerInGame::WordBomb(ServerWordBomb)),
                    room_submessenger!(room, InGameMessage::WordBomb(WordBombMessage)),
                    (uuid, message),
                )?;
            }
            ClientInGame::Anagrams(message) => {
                let anagrams = self.game.try_anagrams()?;

                anagrams.handle_client(
                    client_submessenger!(clients, ServerInGame::Anagrams(ServerAnagrams)),
                    room_submessenger!(room, InGameMessage::Anagrams(AnagramsMessage)),
                    (uuid, message),
                )?;
            }
            ClientInGame::EndRequest => todo!(),
            ClientInGame::ForceEnd => todo!(),
        }

        Ok(None)
    }

    fn handle(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<State>> {
        match message {
            InGameMessage::WordBomb(message) => {
                let word_bomb = self.game.try_word_bomb()?;

                word_bomb.handle(
                    client_submessenger!(clients, ServerInGame::WordBomb(ServerWordBomb)),
                    room_submessenger!(room, InGameMessage::WordBomb(WordBombMessage)),
                    message,
                )?;
            }
            InGameMessage::Anagrams(message) => {
                let anagrams = self.game.try_anagrams()?;

                anagrams.handle(
                    client_submessenger!(clients, ServerInGame::Anagrams(ServerAnagrams)),
                    room_submessenger!(room, InGameMessage::Anagrams(AnagramsMessage)),
                    message,
                )?;
            }
        }
        todo!()
    }
}
