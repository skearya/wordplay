pub mod messages {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::messages::RoomSettings;

    #[derive(Deserialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ClientGeneral {
        Ping {
            timestamp: u64,
        },
        ChatMessage {
            content: String,
        },
        /// Only sendable by the room owner. Can't be used to change game settings mid-game.
        Settings(RoomSettings),
    }

    #[derive(Serialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ServerGeneral {
        /// First message sent after establishing connection, sent only once.
        Info {
            /// Joined client's designated UUID.
            uuid: Uuid,
            /// Room owner's UUID.
            owner: Uuid,
            /// Room and game settings.
            settings: RoomSettings,
            /// Room clients.
            clients: HashMap<Uuid, ServerClient>,
            /// State of the room (lobby | type of game).
            /// TODO: Box to reduce variant size?.
            state: ServerState,
        },
        /// Broadcasted when a client joins/rejoins.
        Join {
            uuid: Uuid,
        },
        /// Broadcasted when a client leaves.
        Leave {
            uuid: Uuid,
        },
        /// Used to broadcast a chat message.
        Chat {
            author: Uuid,
            content: String,
        },
        /// Sent when the room owner has updated room/game settings.
        Settings(RoomSettings),
        Error {
            message: String,
        },
    }

    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct ServerClient {
        username: String,
        /// URL to account avatar.
        avatar_url: Option<String>,
        /// `true` if player disconnected in game only.
        disconnected: bool,
    }

    #[derive(Serialize, TS)]
    #[serde(
        tag = "kind",
        rename_all = "camelCase",
        rename_all_fields = "camelCase"
    )]
    #[ts(export)]
    /// Room state sent to clients when they join.
    pub enum ServerState {
        Lobby {
            ready: Vec<Uuid>,
            /// Unix timestamp of when the countdown timer started.
            timer_start: Option<u64>,
            // TODO: Show previous game info.
            // prev_game: Option<PostGameInfo>,
        },
        Game {
            // Specific game type state (ex: word bomb).
            state: ServerGameState,
            /// UUIDs of players requesting to end the current game.
            requesting_end: Vec<Uuid>,
        },
    }

    #[derive(Serialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ServerGameState {
        // TODO: Game state
        WordBomb(()),
    }
}

use uuid::Uuid;

use crate::room::{Room, general::messages::ClientGeneral};

impl Room {
    pub fn handle_client(&mut self, (uuid, message): (Uuid, ClientGeneral)) -> anyhow::Result<()> {
        match message {
            ClientGeneral::Ping { timestamp } => todo!(),
            ClientGeneral::ChatMessage { content } => todo!(),
            ClientGeneral::Settings(room_settings) => todo!(),
        }

        Ok(())
    }

    // pub fn handle_room(&mut self, message: Room) {}
}
