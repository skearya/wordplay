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
    room::{StateChange, context::Context},
};

pub struct General;

impl General {
    pub fn on_client_message(
        &mut self,
        ctx: Context,
        (uuid, message): (Uuid, ClientGeneral),
    ) -> anyhow::Result<StateChange> {
        match message {
            ClientGeneral::Ping { timestamp } => {
                ctx.clients.send(uuid, ServerGeneral::Pong { timestamp });
            }
            ClientGeneral::Chat { content } => {
                ctx.clients.broadcast(ServerGeneral::Chat {
                    author: uuid,
                    content,
                });
            }
            ClientGeneral::Settings(new) => {
                if uuid != ctx.settings.owner {
                    return Ok(StateChange::None);
                }

                *ctx.settings = new;

                ctx.clients
                    .broadcast(ServerGeneral::Settings(*ctx.settings));
            }
        }

        Ok(StateChange::None)
    }
}
