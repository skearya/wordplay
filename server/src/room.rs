pub mod messages {
    #[allow(unused_imports)]
    use serde::{Deserialize, Serialize};
    use tokio::sync::oneshot;
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::{
        game::messages::{GameState, PostGameInfo},
        messages::{ClientMessage, ServerClient},
        room::clients::Client,
    };

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(
        tag = "kind",
        content = "data",
        rename_all = "camelCase",
        rename_all_fields = "camelCase"
    )]
    #[ts(export)]
    pub enum ServerCore {
        /// Broadcasted when a client joins/rejoins.
        Join { uuid: Uuid, client: ServerClient },
        /// Broadcasted when a client leaves.
        /// `new_owner` will only be some if the owner leaves in lobby, if the owner
        /// leaves in game, they will still be owner and have the chance to rejoin.
        /// If they don't rejoin before the game ends, the game ending message will
        /// broadcast the new owner.
        Leave { uuid: Uuid, new_owner: Option<Uuid> },
        /// Sent when the game (based on room settings) has started.
        GameStart {
            /// Contains the player's rejoin token. Is `None` if client is spectating.
            rejoin_token: Option<Uuid>,
            state: GameState,
        },
        /// Broadcasted when the current game has ended.
        GameEnd {
            post_game_info: Option<PostGameInfo>,
            /// Is `Some` with a random client's uuid if the previous room owner
            /// left during game and hasn't come back.
            new_owner: Option<Uuid>,
        },
    }

    pub enum CoreMessage {
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
        /// Rooms also recieve client messages through the same channel as other room messages.
        Client {
            uuid: Uuid,
            message: ClientMessage,
        },
    }
}

pub mod clients;
pub mod context;
pub mod sender;

use std::{mem, num::NonZero};

use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
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
        messages::{CoreMessage, ServerCore},
        sender::RoomSender,
    },
    task,
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

        task::spawn(Self::new(sender.clone(), reciever).run());

        sender
    }

    async fn run(mut self) -> anyhow::Result<()> {
        while let Some(message) = self.reciever.recv().await {
            match self.on_self_message(message) {
                Ok(change) => match change {
                    StateChange::None => (),
                    StateChange::Lobby(prev_game_info) => {
                        self.clients.keep_connected();

                        let new_owner = self.maybe_new_owner();

                        self.clients.broadcast(ServerCore::GameEnd {
                            post_game_info: prev_game_info.clone(),
                            new_owner,
                        });

                        self.state = State::Lobby(Lobby::new(prev_game_info));
                    }
                    StateChange::Game(players) => {
                        let game = Game::new(
                            Context::new(&self.sender, &mut self.clients, &mut self.settings),
                            &players,
                        );

                        let state = game.snapshot();

                        for (&uuid, client) in self.clients.iter() {
                            client.send(ServerCore::GameStart {
                                rejoin_token: game.rejoin_tokens().get(&uuid).copied(),
                                state: state.clone(),
                            });
                        }

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
            CoreMessage::Client { uuid, message } => self.on_client_message(uuid, message),
            CoreMessage::Join { uuid, client } => {
                self.add_client(uuid, client);

                Ok(StateChange::None)
            }
            CoreMessage::JoinWithRejoinToken {
                rejoin_token,
                client,
                response,
            } => {
                // Previous player UUID, retrieved from rejoin token.
                let player = self.state.try_in_game().ok().and_then(|game| {
                    game.rejoin_tokens().iter().find_map(|(uuid, token)| {
                        if rejoin_token == *token {
                            Some(*uuid)
                        } else {
                            None
                        }
                    })
                });

                let uuid = if let Some(player) = player {
                    // If we try using a rejoin token while they still seem to be connected, end the old connection.
                    if let Some(old) = self.clients.get_mut(player) {
                        let old = mem::replace(old, client);
                        old.close("Reconnected on another client");

                        self.clients.send(player, self.make_info_message(player));
                        response.send(player).ok();

                        return Ok(StateChange::None);
                    }

                    player
                } else {
                    Uuid::new_v4()
                };

                self.add_client(uuid, client);
                response.send(uuid).ok();

                Ok(StateChange::None)
            }
            CoreMessage::Leave { uuid, socket } => {
                if !self
                    .clients
                    .get(uuid)
                    .is_some_and(|client| client.socket_uuid_eq(socket))
                {
                    return Ok(StateChange::None);
                }

                self.state.on_client_leave(
                    Context::new(&self.sender, &mut self.clients, &mut self.settings),
                    uuid,
                );

                if self.clients.is_empty() {
                    return Ok(StateChange::End);
                }

                let new_owner = self.maybe_new_owner();

                self.clients
                    .broadcast(ServerCore::Leave { uuid, new_owner });

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
            self.clients.send(
                uuid,
                ServerMessage::General(ServerGeneral::Error {
                    message: "Rate limited, you're sending messages too fast".to_string(),
                }),
            );

            return Ok(StateChange::None);
        }

        let result = match message {
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
        };

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

    fn make_info_message(&self, uuid: Uuid) -> ServerMessage {
        ServerMessage::Info {
            uuid,
            clients: self
                .clients
                .iter()
                .map(|(&uuid, client)| (uuid, client.into()))
                .collect(),
            settings: self.settings,
            state: Box::new(self.state.snapshot()),
        }
    }

    fn add_client(&mut self, uuid: Uuid, client: Client) {
        if self.settings.owner == Uuid::default() {
            self.settings.owner = uuid;
        }

        let data = (&client).into();

        self.clients.insert(uuid, client);
        self.clients.send(uuid, self.make_info_message(uuid));
        self.clients
            .broadcast_except(uuid, ServerCore::Join { uuid, client: data });
    }

    fn maybe_new_owner(&mut self) -> Option<Uuid> {
        if self.clients.get(self.settings.owner).is_none() {
            let random = *self.clients.random().0;
            self.settings.owner = random;

            Some(random)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use axum::extract::ws;
    use tokio::sync::mpsc::UnboundedReceiver;

    use crate::{
        lobby::messages::LobbyState,
        messages::{ServerClient, ServerState},
        room::clients::{Client, SocketRef},
    };

    use super::*;

    struct FakeClient {
        uuid: Uuid,
        reciever: UnboundedReceiver<ws::Message>,
    }

    impl FakeClient {
        fn new(id: &mut u32) -> (Self, Client) {
            *id += 1;

            let (sender, reciever) = mpsc::unbounded_channel();

            (
                Self {
                    uuid: Uuid::new_v4(),
                    reciever,
                },
                Client::new(
                    SocketRef::new(Uuid::new_v4(), sender),
                    format!("Client {id}"),
                ),
            )
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
        let room = Room::spawn();

        let mut id = 0;
        let (mut one, one_client) = FakeClient::new(&mut id);

        room.send(CoreMessage::Join {
            uuid: one.uuid,
            client: one_client,
        });

        assert_eq!(
            one.recv().await?,
            ServerMessage::Info {
                uuid: one.uuid,
                clients: HashMap::from([(
                    one.uuid,
                    ServerClient {
                        username: "Client 1".to_owned(),
                        avatar_url: None
                    }
                )]),
                settings: RoomSettings {
                    owner: one.uuid,
                    ..RoomSettings::default()
                },
                state: Box::new(ServerState::Lobby(LobbyState {
                    ready: vec![],
                    timer_start: None,
                    prev_game: None
                }))
            }
        );

        Ok(())
    }

    #[tokio::test(start_paused = true)]
    async fn multiple() -> anyhow::Result<()> {
        let room = Room::spawn();

        let mut id = 0;
        let (mut one, one_client) = FakeClient::new(&mut id);
        let (mut two, two_client) = FakeClient::new(&mut id);

        room.send(CoreMessage::Join {
            uuid: one.uuid,
            client: one_client,
        });

        assert_eq!(
            one.recv().await?,
            ServerMessage::Info {
                uuid: one.uuid,
                clients: HashMap::from([(
                    one.uuid,
                    ServerClient {
                        username: "Client 1".to_owned(),
                        avatar_url: None
                    }
                )]),
                settings: RoomSettings {
                    owner: one.uuid,
                    ..RoomSettings::default()
                },
                state: Box::new(ServerState::Lobby(LobbyState {
                    ready: vec![],
                    timer_start: None,
                    prev_game: None
                }))
            }
        );

        room.send(CoreMessage::Join {
            uuid: two.uuid,
            client: two_client,
        });

        assert_eq!(
            two.recv().await?,
            ServerMessage::Info {
                uuid: two.uuid,
                clients: HashMap::from([
                    (
                        one.uuid,
                        ServerClient {
                            username: "Client 1".to_owned(),
                            avatar_url: None
                        }
                    ),
                    (
                        two.uuid,
                        ServerClient {
                            username: "Client 2".to_owned(),
                            avatar_url: None
                        }
                    )
                ]),
                settings: RoomSettings {
                    owner: one.uuid,
                    ..RoomSettings::default()
                },
                state: Box::new(ServerState::Lobby(LobbyState {
                    ready: vec![],
                    timer_start: None,
                    prev_game: None
                }))
            }
        );

        Ok(())
    }
}
