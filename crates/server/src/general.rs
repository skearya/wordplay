pub mod messages {
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
        Chat {
            content: String,
        },
        /// Only sendable by the room owner. Can't be used to change game settings mid-game.
        Settings(RoomSettings),
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(
        tag = "kind",
        rename_all = "camelCase",
        rename_all_fields = "camelCase"
    )]
    #[ts(export)]
    pub enum ServerGeneral {
        /// Sent back when a client sends a `ClientGeneral::Ping`.
        Pong { timestamp: u64 },
        /// Used to broadcast a chat message.
        Chat { author: Uuid, content: String },
        /// Broadcasted when the room owner has updated room/game settings.
        Settings(RoomSettings),
        /// Sent when the server encounters an error processing a client's message.
        Error { message: String },
    }
}

use uuid::Uuid;

use crate::{
    general::messages::{ClientGeneral, ServerGeneral},
    messages::RoomSettings,
    room::{
        StateChange,
        handler::Handler,
        messenger::{ClientMessenger, ClientUtils, RoomMessenger},
    },
};

pub struct General;

impl Handler for General {
    type ClientMessage = ClientGeneral;
    type ServerMessage = ServerGeneral;
    type RoomMessage = ();
    type StateMessage = ();

    fn state(&self) -> Self::StateMessage {}

    fn handle_client(
        &mut self,
        settings: &mut RoomSettings,
        clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils,
        _room: impl RoomMessenger<Self::RoomMessage>,
        (uuid, message): (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<StateChange> {
        match message {
            ClientGeneral::Ping { timestamp } => {
                clients.send(uuid, ServerGeneral::Pong { timestamp });
            }
            ClientGeneral::Chat { content } => {
                clients.broadcast(ServerGeneral::Chat {
                    author: uuid,
                    content,
                });
            }
            ClientGeneral::Settings(new) => {
                if uuid != settings.owner {
                    return Ok(StateChange::None);
                }

                *settings = new;

                clients.broadcast(ServerGeneral::Settings(*settings));
            }
        }

        Ok(StateChange::None)
    }

    fn handle_message(
        &mut self,
        _settings: &mut RoomSettings,
        _clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils,
        _room: impl RoomMessenger<Self::RoomMessage>,
        _message: Self::RoomMessage,
    ) -> anyhow::Result<StateChange> {
        Ok(StateChange::None)
    }

    fn abort(&mut self) {}
}
