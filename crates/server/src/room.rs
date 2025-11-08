pub mod clients;
pub mod handler;
pub mod messenger;
pub mod sender;

use std::{mem, num::NonZero};

use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    game::{
        Game,
        messages::{PostGameInfo, ServerGame},
    },
    general::{General, messages::ServerGeneral},
    lobby::{Lobby, messages::ServerLobby},
    messages::{ClientMessage, RoomMessage, RoomSettings, ServerMessage, ServerState},
    room::{
        clients::{Client, Clients},
        handler::Handler,
        messenger::{ClientMessenger, client_submessenger},
        sender::{GameSender, LobbySender, RoomSender},
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
            Err(anyhow::anyhow!("expected lobby",))
        }
    }

    pub fn try_in_game(&mut self) -> anyhow::Result<&mut Game> {
        if let Self::InGame(v) = self {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("expected in game"))
        }
    }

    pub fn state(&self) -> ServerState {
        match self {
            Self::Lobby(lobby) => ServerState::Lobby(lobby.state()),
            Self::InGame(in_game) => ServerState::Game(in_game.state()),
        }
    }

    pub fn end(&mut self) {
        match self {
            Self::Lobby(lobby) => lobby.abort(),
            Self::InGame(in_game) => in_game.abort(),
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
            state: State::Lobby(Lobby::new(LobbySender::new(sender.clone()), None)),
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
            match self.handle_message(message) {
                Ok(change) => match change {
                    StateChange::None => (),
                    StateChange::Lobby(prev_game_info) => {
                        self.clients.keep_connected();

                        let new_owner = self.new_owner();

                        self.clients.broadcast(ServerMessage::GameEnd {
                            post_game_info: prev_game_info.clone(),
                            new_owner,
                        });

                        self.state = State::Lobby(Lobby::new(
                            LobbySender::new(self.sender.clone()),
                            prev_game_info,
                        ));
                    }
                    StateChange::Game(players) => {
                        let game = Game::new(
                            GameSender::new(self.sender.clone()),
                            &self.settings,
                            &players,
                        );

                        let state = game.state();

                        for (uuid, _) in self.clients.iter() {
                            self.clients.send(
                                *uuid,
                                ServerMessage::GameStart {
                                    rejoin_token: game.rejoin_tokens().get(uuid).copied(),
                                    state: state.clone(),
                                },
                            );
                        }

                        self.state = State::InGame(game);
                    }
                    StateChange::End => {
                        self.state.end();
                        break;
                    }
                },
                Err(err) => tracing::error!(?err),
            }
        }

        Ok(())
    }

    fn handle_message(&mut self, message: RoomMessage) -> anyhow::Result<StateChange> {
        match message {
            RoomMessage::Join { uuid, client } => {
                self.add_client(uuid, client);

                Ok(StateChange::None)
            }
            RoomMessage::JoinWithRejoinToken {
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
                        old.close("Reconnected on another client.");

                        self.clients.send(player, self.info_message(player));
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
            RoomMessage::Leave { uuid, socket } => {
                if !self
                    .clients
                    .get(uuid)
                    .is_some_and(|client| client.socket_uuid_eq(socket))
                {
                    return Ok(StateChange::None);
                }

                match self.state {
                    State::Lobby(_) => self.clients.remove(uuid),
                    State::InGame(_) => self.clients.disconnect(uuid),
                }

                if self.clients.is_empty() {
                    return Ok(StateChange::End);
                }

                let new_owner = self.new_owner();

                self.clients
                    .broadcast(ServerMessage::Leave { uuid, new_owner });

                Ok(StateChange::None)
            }
            RoomMessage::Client { uuid, message } => match self.handle_client(uuid, message) {
                Ok(state) => Ok(state),
                Err(err) => {
                    self.clients.send(
                        uuid,
                        ServerMessage::General(ServerGeneral::Error {
                            message: err.to_string(),
                        }),
                    );

                    Err(err)
                }
            },
            RoomMessage::Lobby(message) => self.state.try_lobby()?.room(
                &mut self.settings,
                client_submessenger!(&mut self.clients, ServerMessage::Lobby(ServerLobby)),
                message,
            ),
            RoomMessage::Game(message) => self.state.try_in_game()?.room(
                &mut self.settings,
                client_submessenger!(&mut self.clients, ServerMessage::Game(ServerGame)),
                message,
            ),
        }
    }

    fn handle_client(&mut self, uuid: Uuid, message: ClientMessage) -> anyhow::Result<StateChange> {
        if self.limiter.check_key(&uuid).is_err() {
            return Err(anyhow::anyhow!(
                "rate limited, you're sending messages too fast"
            ));
        }

        match message {
            ClientMessage::General(message) => General.client(
                &mut self.settings,
                client_submessenger!(&mut self.clients, ServerMessage::General(ServerGeneral)),
                (uuid, message),
            ),
            ClientMessage::Lobby(message) => self.state.try_lobby()?.client(
                &mut self.settings,
                client_submessenger!(&mut self.clients, ServerMessage::Lobby(ServerLobby)),
                (uuid, message),
            ),
            ClientMessage::Game(message) => self.state.try_in_game()?.client(
                &mut self.settings,
                client_submessenger!(&mut self.clients, ServerMessage::Game(ServerGame)),
                (uuid, message),
            ),
        }
    }

    fn info_message(&self, uuid: Uuid) -> ServerMessage {
        ServerMessage::Info {
            uuid,
            clients: self
                .clients
                .iter()
                .map(|(uuid, client)| (*uuid, client.into()))
                .collect(),
            settings: self.settings,
            state: Box::new(self.state.state()),
        }
    }

    fn add_client(&mut self, uuid: Uuid, client: Client) {
        if self.settings.owner == Uuid::default() {
            self.settings.owner = uuid;
        }

        let client_data = (&client).into();

        self.clients.add(uuid, client);

        self.clients.send(uuid, self.info_message(uuid));
        self.clients.broadcast_except(
            uuid,
            ServerMessage::Join {
                uuid,
                client: client_data,
            },
        );
    }

    fn new_owner(&mut self) -> Option<Uuid> {
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
        room::clients::Client,
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
                Client::new(Uuid::new_v4(), sender, format!("Client {id}")),
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

        room.send(RoomMessage::Join {
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

        room.send(RoomMessage::Join {
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

        room.send(RoomMessage::Join {
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
