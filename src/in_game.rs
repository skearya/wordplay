pub mod messages {
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::games::word_bomb::messages::WordBombPostGameInfo;

    #[derive(Deserialize)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    pub enum ClientInGame {
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
        /// Broadcasted when a player requests to end the game early.
        EndRequest { uuid: Uuid },
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
}
