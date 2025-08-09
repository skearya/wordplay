pub mod messages {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use tokio::sync::oneshot;
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::{
        game::messages::GameState, lobby::messages::LobbyState, messages::RoomSettings,
        room::clients::Client,
    };

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
        /// First message sent after establishing connection, sent only once.
        Info {
            /// Joined client's designated UUID.
            uuid: Uuid,
            /// Room and game settings.
            settings: RoomSettings,
            /// Room clients.
            clients: HashMap<Uuid, ServerClient>,
            /// State of the room (lobby | type of game).
            /// TODO: Box to reduce variant size?.
            state: ServerState,
        },
        /// Broadcasted when a client joins/rejoins.
        Join { uuid: Uuid, client: ServerClient },
        /// Broadcasted when a client leaves.
        /// `new_owner` will only be some if the owner leaves in lobby, if the owner
        /// leaves in game, they will still be owner and have the chance to rejoin.
        /// If they don't rejoin before the game ends, the game ending message will
        /// broadcast the new owner.
        Leave { uuid: Uuid, new_owner: Option<Uuid> },
        /// Sent back when a client sends a `ClientGeneral::Ping`.
        Pong { timestamp: u64 },
        /// Used to broadcast a chat message.
        Chat { author: Uuid, content: String },
        /// Broadcasted when the room owner has updated room/game settings.
        Settings(RoomSettings),
        /// Sent when the server encounters an error processing a client's message.
        Error { message: String },
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct ServerClient {
        /// Client username.
        pub username: String,
        /// URL to account avatar.
        pub avatar_url: Option<String>,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
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
            client: Client,
        },
        JoinWithRejoinToken {
            rejoin_token: Uuid,
            client: Client,
            /// Response to the socket task that tried joining containing the client's designated UUID.
            /// If the `rejoin_token` was valid, the client will given the previously associated UUID.
            /// Otherwise, the client will be given a randomly generated UUID.
            response: oneshot::Sender<Uuid>,
        },
        Leave {
            uuid: Uuid,
            socket: Uuid,
        },
    }
}

use std::mem;

use uuid::Uuid;

use crate::{
    general::messages::{ClientGeneral, GeneralMessage, ServerClient, ServerGeneral},
    messages::RoomSettings,
    room::{
        clients::Client,
        messenger::{ClientMessenger, ClientUtils},
        state::State,
    },
};

pub struct General<'a> {
    state: &'a mut State,
}

impl<'a> General<'a> {
    pub fn new(state: &'a mut State) -> Self {
        Self { state }
    }

    pub fn handle_client(
        &mut self,
        settings: &mut RoomSettings,
        clients: impl ClientMessenger<ServerGeneral> + ClientUtils,
        (uuid, message): (Uuid, ClientGeneral),
    ) -> anyhow::Result<Option<State>> {
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
                    return Ok(None);
                }

                *settings = new;

                clients.broadcast(ServerGeneral::Settings(*settings));
            }
        }

        Ok(None)
    }

    pub fn handle_message(
        &mut self,
        settings: &mut RoomSettings,
        mut clients: impl ClientMessenger<ServerGeneral> + ClientUtils,
        message: GeneralMessage,
    ) -> anyhow::Result<Option<State>> {
        match message {
            GeneralMessage::Join { uuid, client } => {
                if settings.owner == Uuid::default() {
                    settings.owner = uuid;
                }

                self.new_client(settings, &mut clients, uuid, client);
            }
            // TODO: Rework join/leave system?
            GeneralMessage::JoinWithRejoinToken {
                rejoin_token,
                client,
                response,
            } => {
                let uuid = if let Some(player) = self
                    .state
                    .try_in_game()
                    .ok()
                    .and_then(|game| game.lookup_rejoin_token(rejoin_token))
                {
                    match clients.get_mut(player) {
                        Some(old) => {
                            let old = mem::replace(old, client);
                            old.close("Reconnected on another client.");

                            clients.send(player, self.info(player, settings, &clients));
                        }
                        None => {
                            self.new_client(settings, &mut clients, player, client);
                        }
                    }

                    player
                } else {
                    let uuid = Uuid::new_v4();
                    self.new_client(settings, &mut clients, uuid, client);

                    uuid
                };

                response.send(uuid).ok();
            }
            GeneralMessage::Leave { uuid, socket } => {
                if !clients
                    .get(uuid)
                    .is_some_and(|client| client.socket_uuid_eq(socket))
                {
                    return Ok(None);
                }

                match self.state {
                    State::Lobby(_) => clients.remove(uuid),
                    State::InGame(_) => clients.disconnect(uuid),
                    State::Ended => unreachable!(),
                }

                if clients.is_empty() {
                    return Ok(Some(State::Ended));
                }

                let new_owner = if uuid == settings.owner {
                    let random = *clients.random().0;
                    settings.owner = random;

                    Some(random)
                } else {
                    None
                };

                clients.broadcast(ServerGeneral::Leave { uuid, new_owner });
            }
        }

        Ok(None)
    }

    fn new_client(
        &self,
        settings: &RoomSettings,
        clients: &mut (impl ClientMessenger<ServerGeneral> + ClientUtils),
        uuid: Uuid,
        client: Client,
    ) {
        let data = (&client).into();

        clients.add(uuid, client);

        clients.send(uuid, self.info(uuid, settings, clients));
        clients.broadcast_except(uuid, ServerGeneral::Join { uuid, client: data });
    }

    fn info(
        &self,
        uuid: Uuid,
        settings: &RoomSettings,
        clients: &impl ClientUtils,
    ) -> ServerGeneral {
        ServerGeneral::Info {
            uuid,
            settings: *settings,
            clients: clients
                .iter()
                .map(|(uuid, client)| (*uuid, client.into()))
                .collect(),
            state: self.state.state(),
        }
    }
}

impl From<&Client> for ServerClient {
    fn from(client: &Client) -> Self {
        Self {
            username: client.username.clone(),
            avatar_url: None,
        }
    }
}
