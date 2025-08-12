pub mod clients;
pub mod handler;
pub mod messenger;
pub mod sender;
pub mod state;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    game::messages::{GameMessage, ServerGame},
    general::{General, messages::ServerGeneral},
    lobby::messages::{LobbyMessage, ServerLobby},
    messages::{ClientMessage, RoomMessage, RoomSettings, ServerMessage},
    room::{
        clients::Clients,
        handler::Handler,
        messenger::{ClientMessenger, RoomMessenger, client_submessenger, room_submessenger},
        sender::RoomSender,
        state::State,
    },
    task,
};

pub struct Room {
    state: State,
    settings: RoomSettings,
    clients: Clients,
    /// Sender to our own room task.
    sender: RoomSender,
    /// Reciever of `RoomMessages`.
    reciever: mpsc::UnboundedReceiver<RoomMessage>,
}

impl Room {
    pub fn new(sender: RoomSender, reciever: mpsc::UnboundedReceiver<RoomMessage>) -> Self {
        Self {
            state: State::default(),
            // Warning: settings.owner is initialized to `Uuid::default()` (nil).
            // Should immediately be overwritten by the owner joining.
            settings: RoomSettings::default(),
            clients: Clients::new(),
            sender,
            reciever,
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
                Ok(Some(State::Ended)) => {
                    self.state.end();
                    break;
                }
                Ok(Some(state)) => self.state = state,
                Ok(None) => (),
                Err(err) => tracing::error!(?err),
            }
        }

        Ok(())
    }

    fn handle_message(&mut self, message: RoomMessage) -> anyhow::Result<Option<State>> {
        match message {
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
            RoomMessage::General(message) => General::new(&mut self.state).handle_message(
                &mut self.settings,
                client_submessenger!(&mut self.clients, ServerMessage::General(ServerGeneral)),
                message,
            ),
            RoomMessage::Lobby(message) => self.state.try_lobby()?.handle_message(
                &mut self.settings,
                client_submessenger!(&mut self.clients, ServerMessage::Lobby(ServerLobby)),
                room_submessenger!(self.sender.clone(), RoomMessage::Lobby(LobbyMessage)),
                message,
            ),
            RoomMessage::Game(message) => self.state.try_in_game()?.handle_message(
                &mut self.settings,
                client_submessenger!(&mut self.clients, ServerMessage::Game(ServerGame)),
                room_submessenger!(self.sender.clone(), RoomMessage::Game(GameMessage)),
                message,
            ),
        }
    }

    fn handle_client(
        &mut self,
        uuid: Uuid,
        message: ClientMessage,
    ) -> anyhow::Result<Option<State>> {
        match message {
            ClientMessage::General(message) => General::new(&mut self.state).handle_client(
                &mut self.settings,
                client_submessenger!(&mut self.clients, ServerMessage::General(ServerGeneral)),
                (uuid, message),
            ),
            ClientMessage::Lobby(message) => self.state.try_lobby()?.handle_client(
                &mut self.settings,
                client_submessenger!(&mut self.clients, ServerMessage::Lobby(ServerLobby)),
                room_submessenger!(self.sender.clone(), RoomMessage::Lobby(LobbyMessage)),
                (uuid, message),
            ),
            ClientMessage::Game(message) => self.state.try_in_game()?.handle_client(
                &mut self.settings,
                client_submessenger!(&mut self.clients, ServerMessage::Game(ServerGame)),
                room_submessenger!(self.sender.clone(), RoomMessage::Game(GameMessage)),
                (uuid, message),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use axum::extract::ws;
    use tokio::sync::mpsc::UnboundedReceiver;

    use crate::{
        general::messages::{GeneralMessage, ServerClient, ServerState},
        lobby::messages::LobbyState,
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

        room.send(RoomMessage::General(GeneralMessage::Join {
            uuid: one.uuid,
            client: one_client,
        }));

        assert_eq!(
            one.recv().await?,
            ServerMessage::General(ServerGeneral::Info {
                uuid: one.uuid,
                settings: RoomSettings {
                    owner: one.uuid,
                    ..RoomSettings::default()
                },
                clients: HashMap::from([(
                    one.uuid,
                    ServerClient {
                        username: "Client 1".to_owned(),
                        avatar_url: None
                    }
                )]),
                state: ServerState::Lobby(LobbyState {
                    ready: vec![],
                    timer_start: None
                })
            })
        );

        Ok(())
    }

    #[tokio::test(start_paused = true)]
    async fn multiple() -> anyhow::Result<()> {
        let room = Room::spawn();

        let mut id = 0;
        let (mut one, one_client) = FakeClient::new(&mut id);
        let (mut two, two_client) = FakeClient::new(&mut id);

        room.send(RoomMessage::General(GeneralMessage::Join {
            uuid: one.uuid,
            client: one_client,
        }));

        assert_eq!(
            one.recv().await?,
            ServerMessage::General(ServerGeneral::Info {
                uuid: one.uuid,
                settings: RoomSettings {
                    owner: one.uuid,
                    ..RoomSettings::default()
                },
                clients: HashMap::from([(
                    one.uuid,
                    ServerClient {
                        username: "Client 1".to_owned(),
                        avatar_url: None
                    }
                )]),
                state: ServerState::Lobby(LobbyState {
                    ready: vec![],
                    timer_start: None
                })
            })
        );

        room.send(RoomMessage::General(GeneralMessage::Join {
            uuid: two.uuid,
            client: two_client,
        }));

        assert_eq!(
            two.recv().await?,
            ServerMessage::General(ServerGeneral::Info {
                uuid: two.uuid,
                settings: RoomSettings {
                    owner: one.uuid,
                    ..RoomSettings::default()
                },
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
                state: ServerState::Lobby(LobbyState {
                    ready: vec![],
                    timer_start: None
                })
            })
        );

        Ok(())
    }
}
