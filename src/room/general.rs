pub mod messages {
    use serde::{Deserialize, Serialize};
    use tokio::sync::mpsc;
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
            clients: Vec<ServerClient>,
            /// State of the room (lobby | type of game).
            /// TODO: Box to reduce variant size?.
            state: ServerState,
        },
        /// Broadcasted when a client joins/rejoins.
        Join { uuid: Uuid },
        /// Broadcasted when a client leaves.
        Leave { uuid: Uuid },
        /// Used to broadcast a chat message.
        Chat { author: Uuid, content: String },
        /// Broadcasted when the room owner has updated room/game settings.
        Settings(RoomSettings),
        /// Sent when the server encounters an error processing a client's message.
        Error { message: String },
    }

    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct ServerClient {
        uuid: Uuid,
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

    pub enum GeneralMessage {
        Joined {
            uuid: Uuid,
            sender: mpsc::UnboundedSender<axum::extract::ws::Message>,
        },
        Left {
            uuid: Uuid,
        },
        Close,
    }
}

use uuid::Uuid;

use crate::room::{
    Room,
    general::messages::{ClientGeneral, GeneralMessage},
    state::State,
};

// Special `Handler` implementation for "General" messages, requires a mutable reference to `Clients`
// and more mutable access to `Room` in order to operate which can't be provided in `Handler`.
impl Room {
    pub fn handle_client(
        &mut self,
        (uuid, message): (Uuid, ClientGeneral),
    ) -> anyhow::Result<Option<State>> {
        match message {
            ClientGeneral::Ping { timestamp } => todo!(),
            ClientGeneral::ChatMessage { content } => todo!(),
            ClientGeneral::Settings(room_settings) => todo!(),
        }
    }

    pub fn handle_message(&mut self, message: GeneralMessage) -> anyhow::Result<Option<State>> {
        match message {
            GeneralMessage::Joined { uuid, sender } => {
                self.clients.add(uuid, sender);
            }
            GeneralMessage::Left { uuid } => {
                self.clients.remove(uuid);
            }
            GeneralMessage::Close => {
                if self.clients.is_empty() {
                    self.state.end();
                    self.close = true;
                }
            }
        }

        Ok(None)
    }
}
