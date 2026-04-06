pub mod messages {
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};
    use tokio::sync::oneshot;
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::{
        game::messages::{GameState, PostGameInfo},
        messages::{ClientMessage, RoomSettings, ServerClient},
        room::clients::SocketRef,
        socket::SocketParams,
    };

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(
        tag = "kind",
        rename_all = "camelCase",
        rename_all_fields = "camelCase"
    )]
    #[ts(export)]
    pub enum ServerCore {
        /// Broadcasted when a client joins.
        Join { uuid: Uuid, client: ServerClient },
        /// Broadcasted when a client rejoins.
        Rejoin { uuid: Uuid },
        /// Broadcasted when a client leaves.
        Leave { uuid: Uuid },
        /// Sent when the game (based on room settings) has started.
        GameStart { state: GameState },
        /// Broadcasted when the current game has ended.
        GameEnd {
            post_game_info: Option<PostGameInfo>,
        },
    }

    pub enum CoreMessage {
        Join {
            socket: SocketRef,
            params: SocketParams,
            /// Response to the socket task containing the client's designated UUID.
            /// If the `rejoin_token` was valid, the client will given the previously associated UUID.
            /// Otherwise, the client will be given a randomly generated UUID.
            sender: oneshot::Sender<Option<Uuid>>,
        },
        Leave {
            uuid: Uuid,
            socket: Uuid,
        },
        /// Rooms also recieve client messages through the same channel as other room messages.
        Client {
            uuid: Uuid,
            message: ClientMessage,
        },
        /// Request to the room task for basic room info.
        /// This will be used on the homepage (if game is public) and room join page.
        InfoRequest {
            sender: oneshot::Sender<RoomInfo>,
        },
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct RoomInfo {
        pub settings: RoomSettings,
        pub clients: Vec<ServerClient>,
    }
}

pub mod clients;
pub mod context;
pub mod sender;

use std::num::NonZero;

use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use rustrict::CensorStr;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    game::{
        Game,
        messages::{GameMessage, PostGameInfo},
    },
    general::{General, messages::ServerGeneral},
    lobby::Lobby,
    messages::{ClientMessage, RoomMessage, RoomSettings, ServerMessage, ServerState},
    room::{
        clients::{Client, Clients},
        context::Context,
        messages::{CoreMessage, RoomInfo, ServerCore},
        sender::RoomSender,
    },
};

pub enum State {
    Lobby(Lobby),
    InGame(Game),
}

impl State {
    pub fn try_lobby(&mut self) -> anyhow::Result<&mut Lobby> {
        if let Self::Lobby(v) = self {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("expected lobby"))
        }
    }

    pub fn try_in_game(&mut self) -> anyhow::Result<&mut Game> {
        if let Self::InGame(v) = self {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("expected in game"))
        }
    }

    pub fn on_abort(&mut self) {
        match self {
            Self::Lobby(lobby) => lobby.on_abort(),
            Self::InGame(game) => game.on_abort(),
        }
    }

    pub fn on_client_leave(&mut self, ctx: Context, uuid: Uuid) {
        match self {
            Self::Lobby(lobby) => lobby.on_client_leave(ctx, uuid),
            Self::InGame(game) => game.on_client_leave(ctx, uuid),
        }
    }

    pub fn snapshot(&self) -> ServerState {
        match self {
            Self::Lobby(lobby) => ServerState::Lobby(lobby.snapshot()),
            Self::InGame(game) => ServerState::Game(game.snapshot()),
        }
    }
}

pub enum StateChange {
    Lobby(Option<PostGameInfo>),
    Game(Vec<Uuid>),
    End,
    None,
}

pub struct Room {
    state: State,
    settings: RoomSettings,
    clients: Clients,
    /// Sender to our own room task.
    sender: RoomSender,
    /// Reciever of `RoomMessages`.
    reciever: mpsc::UnboundedReceiver<RoomMessage>,
    /// Rate limiter of client messages.
    limiter: DefaultKeyedRateLimiter<Uuid>,
}

impl Room {
    pub fn new(sender: RoomSender, reciever: mpsc::UnboundedReceiver<RoomMessage>) -> Self {
        Self {
            state: State::Lobby(Lobby::new(None)),
            // Warning: settings.owner is initialized to `Uuid::default()` (nil).
            // Currently should immediately be overwritten by the owner joining.
            settings: RoomSettings::default(),
            clients: Clients::new(),
            sender,
            reciever,
            limiter: RateLimiter::keyed(Quota::per_second(NonZero::new(20).unwrap())),
        }
    }

    pub fn spawn() -> RoomSender {
        let (sender, reciever) = mpsc::unbounded_channel::<RoomMessage>();
        let sender = RoomSender::new(sender);

        tokio::spawn(Self::new(sender.clone(), reciever).run());

        sender
    }

    async fn run(mut self) -> anyhow::Result<()> {
        while let Some(message) = self.reciever.recv().await {
            match self.on_self_message(message) {
                Ok(change) => match change {
                    StateChange::None => (),
                    StateChange::Lobby(prev_game_info) => {
                        self.clients.broadcast(ServerCore::GameEnd {
                            post_game_info: prev_game_info.clone(),
                        });

                        self.state = State::Lobby(Lobby::new(prev_game_info));
                    }
                    StateChange::Game(players) => {
                        let game = Game::new(
                            Context::new(&self.sender, &mut self.clients, &mut self.settings),
                            &players,
                        );

                        self.clients.broadcast(ServerCore::GameStart {
                            state: game.snapshot(),
                        });

                        self.state = State::InGame(game);
                    }
                    StateChange::End => {
                        self.state.on_abort();
                        break;
                    }
                },
                Err(err) => tracing::error!(?err),
            }
        }

        Ok(())
    }

    fn on_self_message(&mut self, message: RoomMessage) -> anyhow::Result<StateChange> {
        match message {
            RoomMessage::Core(message) => self.on_core_message(message),
            RoomMessage::Lobby(message) => self.state.try_lobby()?.on_self_message(
                Context::new(&self.sender, &mut self.clients, &mut self.settings),
                message,
            ),
            RoomMessage::Game(message) => self.state.try_in_game()?.on_self_message(
                Context::new(&self.sender, &mut self.clients, &mut self.settings),
                message,
            ),
        }
    }

    fn on_core_message(&mut self, message: CoreMessage) -> anyhow::Result<StateChange> {
        match message {
            CoreMessage::Client { uuid, message } => {
                let result = self.on_client_message(uuid, message);

                if let Err(err) = &result {
                    self.clients.send(
                        uuid,
                        ServerMessage::General(ServerGeneral::Error {
                            message: err.to_string(),
                        }),
                    );
                }

                result
            }
            CoreMessage::Join {
                socket,
                params,
                sender,
            } => {
                let error = if self.clients.len() > self.settings.size as usize {
                    Some("room full")
                } else if params.username.is_empty() {
                    Some("username cannot be empty")
                } else if params.username.len() > 20 {
                    Some("username too long (max 20 characters)")
                } else if params.username.is_inappropriate() {
                    Some("username likely contains inappropriate content")
                } else {
                    None
                };

                if let Some(err) = error {
                    socket.close(err);
                    sender.send(None).ok();

                    return Ok(StateChange::None);
                }

                let (uuid, rejoined) = if let Some(rejoin_token) = params.rejoin_token
                    && let Some((&uuid, client)) = self.clients.get_by_rejoin_token(rejoin_token)
                {
                    // If we try using a rejoin token while they still seem to be connected, end the old connection.
                    if client.connected() {
                        client.close("Reconnected on another client");
                    }

                    client.socket = Some(socket);

                    (uuid, true)
                } else {
                    let uuid = Uuid::new_v4();

                    if self.settings.owner == Uuid::default() {
                        self.settings.owner = uuid;
                    }

                    self.clients
                        .insert(uuid, Client::new(socket, params.username));

                    (uuid, false)
                };

                let client = self
                    .clients
                    .get(&uuid)
                    .expect("uuid should point to existing client?");

                client.send(ServerMessage::Info {
                    uuid,
                    rejoin_token: client.rejoin_token,
                    clients: self
                        .clients
                        .iter()
                        .map(|(&uuid, client)| (uuid, client.into()))
                        .collect(),
                    settings: self.settings,
                    state: Box::new(self.state.snapshot()),
                });

                self.clients.broadcast(if rejoined {
                    ServerCore::Rejoin { uuid }
                } else {
                    ServerCore::Join {
                        uuid,
                        client: client.into(),
                    }
                });

                sender.send(Some(uuid)).ok();

                Ok(StateChange::None)
            }
            CoreMessage::Leave { uuid, socket } => {
                if !self
                    .clients
                    .get(&uuid)
                    .is_some_and(|client| client.socket_uuid_eq(socket))
                {
                    return Ok(StateChange::None);
                }

                self.state.on_client_leave(
                    Context::new(&self.sender, &mut self.clients, &mut self.settings),
                    uuid,
                );

                self.clients.disconnect(uuid);

                if self.clients.is_empty() {
                    return Ok(StateChange::End);
                }

                self.clients.broadcast(ServerCore::Leave { uuid });

                Ok(StateChange::None)
            }
            CoreMessage::InfoRequest { sender } => {
                sender
                    .send(RoomInfo {
                        settings: self.settings,
                        clients: self.clients.values().map(|client| client.into()).collect(),
                    })
                    .ok();

                Ok(StateChange::None)
            }
        }
    }

    fn on_client_message(
        &mut self,
        uuid: Uuid,
        message: ClientMessage,
    ) -> anyhow::Result<StateChange> {
        if self.limiter.check_key(&uuid).is_err() {
            return Err(anyhow::anyhow!(
                "Rate limited, you're sending messages too fast"
            ));
        }

        match message {
            ClientMessage::General(message) => General.on_client_message(
                Context::new(&self.sender, &mut self.clients, &mut self.settings),
                (uuid, message),
            ),
            ClientMessage::Lobby(message) => self.state.try_lobby()?.on_client_message(
                Context::new(&self.sender, &mut self.clients, &mut self.settings),
                (uuid, message),
            ),
            ClientMessage::Game(message) => self.state.try_in_game()?.on_client_message(
                Context::new(&self.sender, &mut self.clients, &mut self.settings),
                (uuid, message),
            ),
            ClientMessage::WordBomb(message) => self.state.try_in_game()?.on_self_message(
                Context::new(&self.sender, &mut self.clients, &mut self.settings),
                GameMessage::ClientWordBomb((uuid, message)),
            ),
            ClientMessage::Anagrams(message) => self.state.try_in_game()?.on_self_message(
                Context::new(&self.sender, &mut self.clients, &mut self.settings),
                GameMessage::ClientAnagrams((uuid, message)),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use axum::extract::ws;
    use tokio::sync::{mpsc::UnboundedReceiver, oneshot};

    use crate::{
        lobby::messages::LobbyState,
        messages::{ServerClient, ServerState},
        room::clients::SocketRef,
        socket::SocketParams,
    };

    use super::*;

    struct FakeClient {
        uuid: Uuid,
        reciever: UnboundedReceiver<ws::Message>,
    }

    impl FakeClient {
        async fn new(id: &mut u32, room: &RoomSender) -> anyhow::Result<Self> {
            *id += 1;

            let (socket_sender, socket_reciever) = mpsc::unbounded_channel();
            let (uuid_sender, uuid_reciever) = oneshot::channel();

            room.send(CoreMessage::Join {
                socket: SocketRef::new(Uuid::new_v4(), socket_sender),
                params: SocketParams {
                    username: format!("Client {id}"),
                    rejoin_token: None,
                },
                sender: uuid_sender,
            });

            Ok(Self {
                uuid: uuid_reciever
                    .await?
                    .expect("should've been able to join room"),
                reciever: socket_reciever,
            })
        }

        async fn recv(&mut self) -> anyhow::Result<ServerMessage> {
            let msg = serde_json::from_str(
                self.reciever
                    .recv()
                    .await
                    .ok_or(anyhow::anyhow!("room shouldn't be closed"))?
                    .to_text()?,
            )?;

            Ok(msg)
        }
    }

    #[tokio::test(start_paused = true)]
    async fn joining() -> anyhow::Result<()> {
        let mut id = 0;

        let room = Room::spawn();
        let mut one = FakeClient::new(&mut id, &room).await?;

        assert!(matches!(one.recv().await?,
            ServerMessage::Info {
                uuid,
                clients,
                settings,
                state,
                ..
            } if uuid == one.uuid
                && clients
                    == HashMap::from([(
                        one.uuid,
                        ServerClient {
                            username: "Client 1".to_owned(),
                            avatar_url: None,
                            connected: true,
                        },
                    )])
                && settings
                    == RoomSettings {
                        owner: one.uuid,
                        ..RoomSettings::default()
                    }
                && *state
                    == ServerState::Lobby(LobbyState {
                        ready: vec![],
                        timer_start: None,
                        prev_game: None,
                    })
        ));

        Ok(())
    }

    #[tokio::test(start_paused = true)]
    async fn multiple_joining() -> anyhow::Result<()> {
        let mut id = 0;
        let room = Room::spawn();

        let mut one = FakeClient::new(&mut id, &room).await?;

        assert!(matches!(one.recv().await?,
            ServerMessage::Info {
                uuid,
                clients,
                settings,
                state,
                ..
            } if uuid == one.uuid
                && clients
                    == HashMap::from([(
                        one.uuid,
                        ServerClient {
                            username: "Client 1".to_owned(),
                            avatar_url: None,
                            connected: true,
                        },
                    )])
                && settings
                    == RoomSettings {
                        owner: one.uuid,
                        ..RoomSettings::default()
                    }
                && *state
                    == ServerState::Lobby(LobbyState {
                        ready: vec![],
                        timer_start: None,
                        prev_game: None,
                    })
        ));

        let mut two = FakeClient::new(&mut id, &room).await?;

        assert!(matches!(two.recv().await?,
            ServerMessage::Info {
                uuid,
                clients,
                settings,
                state,
                ..
            } if uuid == two.uuid
                && clients
                    == HashMap::from([
                        (
                            one.uuid,
                            ServerClient {
                                username: "Client 1".to_owned(),
                                avatar_url: None,
                                connected: true
                            }
                        ),
                        (
                            two.uuid,
                            ServerClient {
                                username: "Client 2".to_owned(),
                                avatar_url: None,
                                connected: true
                            }
                        )
                    ])
                && settings
                    == RoomSettings {
                        owner: one.uuid,
                        ..RoomSettings::default()
                    }
                && *state
                    == ServerState::Lobby(LobbyState {
                        ready: vec![],
                        timer_start: None,
                        prev_game: None,
                    })
        ));

        Ok(())
    }
}
