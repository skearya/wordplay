pub mod messages {
    use axum::extract::ws;
    use serde::{Deserialize, Serialize};
    use tokio::sync::{mpsc, oneshot};
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::{game::messages::GameState, lobby::messages::LobbyState, messages::RoomSettings};

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
        pub uuid: Uuid,
        pub username: String,
        /// URL to account avatar.
        pub avatar_url: Option<String>,
    }

    #[derive(Serialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    /// Room state sent to clients when they join.
    pub enum ServerState {
        Lobby(LobbyState),
        Game(GameState),
    }

    pub enum GeneralMessage {
        Join {
            uuid: Uuid,
            socket: Uuid,
            username: String,
            sender: mpsc::UnboundedSender<ws::Message>,
        },
        JoinWithRejoinToken {
            rejoin_token: Uuid,
            socket: Uuid,
            username: String,
            sender: mpsc::UnboundedSender<ws::Message>,
            /// Response to the socket task that tried joining containing the client's designated UUID.
            /// If the `rejoin_token` was valid, the client will given the previously associated UUID.
            /// Otherwise, the client will be given a randomly generated UUID.
            response: oneshot::Sender<Uuid>,
        },
        Leave {
            uuid: Uuid,
            socket: Uuid,
        },
        Close,
    }
}

use uuid::Uuid;

use crate::{
    messages::RoomSettings,
    room::{
        clients::Client,
        general::messages::{
            ClientGeneral, GeneralMessage, ServerClient, ServerGeneral, ServerState,
        },
        handler::HandlerMut,
        messenger::{ClientMessenger, ClientUtils, ClientUtilsMut, RoomMessenger},
        state::State,
    },
};

pub struct General<'a> {
    state: &'a mut State,
    close: &'a mut bool,
}

impl HandlerMut for General<'_> {
    type ClientMessage = ClientGeneral;
    type ServerMessage = ServerGeneral;
    type RoomMessage = GeneralMessage;
    type StateMessage = ServerState;

    fn handle_client(
        &mut self,
        settings: &mut RoomSettings,
        clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils + ClientUtilsMut,
        room: impl RoomMessenger<Self::RoomMessage>,
        (uuid, message): (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<State>> {
        match message {
            ClientGeneral::Ping { timestamp } => todo!(),
            ClientGeneral::Chat { content } => {
                clients.broadcast(ServerGeneral::Chat {
                    author: uuid,
                    content,
                });
            }
            ClientGeneral::Settings(new) => {
                *settings = new;

                clients.broadcast(ServerGeneral::Settings(settings.clone()));
            }
        }

        Ok(None)
    }

    fn handle_message(
        &mut self,
        settings: &mut RoomSettings,
        mut clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils + ClientUtilsMut,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<State>> {
        match message {
            GeneralMessage::Join {
                uuid,
                socket,
                username,
                sender,
            } => {
                clients.add(uuid, Client::new(socket, sender, username));

                clients.send(uuid, self.info(uuid, &settings, &clients));
            }
            GeneralMessage::JoinWithRejoinToken {
                rejoin_token,
                socket,
                sender,
                username,
                response,
            } => {
                let uuid = if let Some(player) = self
                    .state
                    .try_in_game()
                    .ok()
                    .and_then(|game| game.lookup_rejoin_token(rejoin_token))
                {
                    match clients.get_mut(player) {
                        Some(client) => {
                            client.close("Reconnected on another client.");
                            *client = Client::new(socket, sender, username);
                        }
                        None => {
                            clients.add(player, Client::new(socket, sender, username));
                        }
                    }

                    player
                } else {
                    let uuid = Uuid::new_v4();
                    clients.add(uuid, Client::new(socket, sender, username));

                    uuid
                };

                response.send(uuid).ok();

                clients.send(uuid, self.info(uuid, &settings, &clients));
            }
            GeneralMessage::Leave { uuid, socket } => {
                clients.remove(uuid, socket);
            }
            GeneralMessage::Close => {
                if clients.is_empty() {
                    *self.close = true;
                }
            }
        }

        Ok(None)
    }

    fn state(&self) -> Self::StateMessage {
        self.state.state()
    }

    fn end(&mut self) {}
}

impl<'a> General<'a> {
    pub fn new(state: &'a mut State, close: &'a mut bool) -> Self {
        Self { state, close }
    }

    fn info(
        &self,
        uuid: Uuid,
        settings: &RoomSettings,
        clients: &impl ClientUtils,
    ) -> ServerGeneral {
        ServerGeneral::Info {
            uuid,
            settings: settings.clone(),
            clients: clients
                .iter()
                .map(|(&uuid, client)| ServerClient {
                    uuid,
                    username: client.username.clone(),
                    avatar_url: None,
                })
                .collect(),
            state: self.state(),
        }
    }
}
