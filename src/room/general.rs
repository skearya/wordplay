pub mod messages {
    use axum::extract::ws;
    use serde::{Deserialize, Serialize};
    use tokio::sync::{mpsc, oneshot};
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
        Join {
            uuid: Uuid,
            socket_uuid: Uuid,
            sender: mpsc::UnboundedSender<ws::Message>,
        },
        JoinWithRejoinToken {
            rejoin_token: Uuid,
            socket_uuid: Uuid,
            sender: mpsc::UnboundedSender<ws::Message>,
            /// Response to the socket task that tried joining containing the client's designated UUID.
            /// If the `rejoin_token` was valid, the client will given the previously associated UUID.
            /// Otherwise, the client will be given a randomly generated UUID.
            response: oneshot::Sender<Uuid>,
        },
        Leave {
            uuid: Uuid,
            socket_uuid: Uuid,
        },
        Close,
    }
}

use uuid::Uuid;

use crate::{
    messages::ServerMessage,
    room::{
        Room,
        clients::Client,
        general::messages::{ClientGeneral, GeneralMessage, ServerGeneral},
        messenger::ClientMessenger,
        state::State,
    },
};

// Special `Handler` implementation for "General" messages which requires a mutable reference to `Clients`
// and more mutable access to `Room` in order to operate which can't be provided in `Handler`.
impl Room {
    pub fn handle_client(
        &mut self,
        (uuid, message): (Uuid, ClientGeneral),
    ) -> anyhow::Result<Option<State>> {
        match message {
            ClientGeneral::Ping { timestamp } => todo!(),
            ClientGeneral::ChatMessage { content } => {
                self.clients
                    .broadcast(ServerMessage::General(ServerGeneral::Chat {
                        author: uuid,
                        content,
                    }));
            }
            ClientGeneral::Settings(room_settings) => todo!(),
        }

        Ok(None)
    }

    pub fn handle_message(&mut self, message: GeneralMessage) -> anyhow::Result<Option<State>> {
        match message {
            GeneralMessage::Join {
                uuid,
                socket_uuid,
                sender,
            } => {
                self.clients.add(uuid, socket_uuid, sender);
            }
            GeneralMessage::JoinWithRejoinToken {
                rejoin_token,
                socket_uuid,
                sender,
                response,
            } => {
                let uuid = if let Some(player) = self
                    .state
                    .try_in_game()
                    .ok()
                    .and_then(|game| game.lookup_rejoin_token(rejoin_token))
                {
                    match self.clients.get_mut(&player) {
                        Some(client) => {
                            client.close("Reconnected on another client.");
                            *client = Client::new(socket_uuid, sender);
                        }
                        None => {
                            self.clients.add(player, socket_uuid, sender);
                        }
                    }

                    player
                } else {
                    let uuid = Uuid::new_v4();
                    self.clients.add(uuid, socket_uuid, sender);

                    uuid
                };

                response.send(uuid).ok();
            }
            GeneralMessage::Leave { uuid, socket_uuid } => {
                self.clients.remove(uuid, socket_uuid);
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

// struct General2<'a> {
//     settings: &'a mut RoomSettings,
// }

// impl Handler<State> for General2<'_> {
//     type ClientMessage = ClientGeneral;
//     type ServerMessage = ServerGeneral;
//     type RoomMessage = GeneralMessage;

//     fn new(settings: &RoomSettings) -> Self {
//         panic!()
//     }

//     fn handle_client(
//         &mut self,
//         clients: impl ClientMessenger<Self::ServerMessage>,
//         room: impl RoomMessenger<Self::RoomMessage>,
//         message: (Uuid, Self::ClientMessage),
//     ) -> anyhow::Result<Option<State>> {
//         todo!()
//     }

//     fn handle_message(
//         &mut self,
//         clients: impl ClientMessenger<Self::ServerMessage>,
//         room: impl RoomMessenger<Self::RoomMessage>,
//         message: Self::RoomMessage,
//     ) -> anyhow::Result<Option<State>> {
//         todo!()
//     }

//     fn end(&mut self) {
//         panic!()
//     }
// }
