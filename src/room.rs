pub mod clients;
pub mod general;
pub mod handler;
pub mod messenger;
pub mod sender;
pub mod state;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    game::messages::{GameMessage, ServerGame},
    lobby::messages::{LobbyMessage, ServerLobby},
    messages::{ClientMessage, RoomMessage, RoomSettings, ServerMessage},
    room::{
        clients::{Client, Clients},
        general::{
            General,
            messages::{GeneralMessage, ServerGeneral},
        },
        handler::{Handler, HandlerMut},
        messenger::{
            ClientMessenger, RoomMessenger, client_submessenger, client_submessenger_mut,
            room_submessenger,
        },
        sender::RoomSender,
        state::State,
    },
    task,
};

pub struct Room {
    state: State,
    clients: Clients,
    settings: RoomSettings,
    /// Sender to our own room task.
    sender: RoomSender,
    /// Reciever of `RoomMessages`.
    reciever: mpsc::UnboundedReceiver<RoomMessage>,
    /// Indicator that the room task should end.
    close: bool,
}

impl Room {
    pub fn new(
        owner: (Uuid, Client),
        sender: RoomSender,
        reciever: mpsc::UnboundedReceiver<RoomMessage>,
    ) -> Self {
        Self {
            state: State::default(),
            settings: RoomSettings {
                owner: owner.0,
                ..RoomSettings::default()
            },
            clients: Clients::new(sender.clone(), owner),
            sender,
            reciever,
            close: false,
        }
    }

    pub fn spawn(owner: (Uuid, Client)) -> RoomSender {
        let (sender, reciever) = mpsc::unbounded_channel::<RoomMessage>();
        let sender = RoomSender::new(sender);

        task::spawn(Self::new(owner, sender.clone(), reciever).run());

        sender
    }

    async fn run(mut self) -> anyhow::Result<()> {
        while let Some(message) = self.reciever.recv().await {
            match self.handle(message) {
                Ok(Some(state)) => self.state = state,
                Ok(None) => (),
                Err(err) => tracing::error!(?err),
            }

            if self.close {
                self.state.end();
                break;
            }
        }

        Ok(())
    }

    fn handle(&mut self, message: RoomMessage) -> anyhow::Result<Option<State>> {
        match message {
            RoomMessage::Client { uuid, message } => match self.client(uuid, message) {
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
            RoomMessage::General(message) => General::new(&mut self.state, &mut self.close)
                .handle_message(
                    &mut self.settings,
                    client_submessenger_mut!(
                        &mut self.clients,
                        ServerMessage::General(ServerGeneral)
                    ),
                    room_submessenger!(self.sender.clone(), RoomMessage::General(GeneralMessage)),
                    message,
                ),
            RoomMessage::Lobby(message) => self.state.try_lobby()?.handle_message(
                &self.settings,
                client_submessenger!(&self.clients, ServerMessage::Lobby(ServerLobby)),
                room_submessenger!(self.sender.clone(), RoomMessage::Lobby(LobbyMessage)),
                message,
            ),
            RoomMessage::InGame(message) => self.state.try_in_game()?.handle_message(
                &self.settings,
                client_submessenger!(&self.clients, ServerMessage::InGame(ServerGame)),
                room_submessenger!(self.sender.clone(), RoomMessage::InGame(GameMessage)),
                message,
            ),
        }
    }

    fn client(&mut self, uuid: Uuid, message: ClientMessage) -> anyhow::Result<Option<State>> {
        match message {
            ClientMessage::General(message) => General::new(&mut self.state, &mut self.close)
                .handle_client(
                    &mut self.settings,
                    client_submessenger_mut!(
                        &mut self.clients,
                        ServerMessage::General(ServerGeneral)
                    ),
                    room_submessenger!(self.sender.clone(), RoomMessage::General(GeneralMessage)),
                    (uuid, message),
                ),
            ClientMessage::Lobby(message) => self.state.try_lobby()?.handle_client(
                &self.settings,
                client_submessenger!(&self.clients, ServerMessage::Lobby(ServerLobby)),
                room_submessenger!(self.sender.clone(), RoomMessage::Lobby(LobbyMessage)),
                (uuid, message),
            ),
            ClientMessage::InGame(message) => self.state.try_in_game()?.handle_client(
                &self.settings,
                client_submessenger!(&self.clients, ServerMessage::InGame(ServerGame)),
                room_submessenger!(self.sender.clone(), RoomMessage::InGame(GameMessage)),
                (uuid, message),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::extract::ws;
    use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
    use uuid::Uuid;

    use crate::{
        messages::{ClientMessage, RoomMessage},
        room::{
            Room,
            clients::Client,
            general::messages::{ClientGeneral, GeneralMessage},
            messenger::RoomMessenger,
        },
    };

    struct FakeClient {
        uuid: Uuid,
        socket: Uuid,
        username: String,
        sender: UnboundedSender<ws::Message>,
        reciever: UnboundedReceiver<ws::Message>,
    }

    impl FakeClient {
        fn new(id: &mut u32) -> Self {
            *id += 1;

            let (sender, reciever) = mpsc::unbounded_channel();

            Self {
                uuid: Uuid::new_v4(),
                socket: Uuid::new_v4(),
                username: format!("Client {id}"),
                sender,
                reciever,
            }
        }
    }

    #[tokio::test(start_paused = true)]
    async fn chatting() -> anyhow::Result<()> {
        let mut id = 0;

        let one = FakeClient::new(&mut id);
        let room = Room::spawn((one.uuid, Client::new(one.socket, one.sender, one.username)));
        let mut two = FakeClient::new(&mut id);

        room.send(RoomMessage::General(GeneralMessage::Join {
            uuid: two.uuid,
            socket: two.socket,
            sender: two.sender,
            username: two.username,
        }));

        room.send(RoomMessage::Client {
            uuid: one.uuid,
            message: ClientMessage::General(ClientGeneral::Chat {
                content: "hi".to_owned(),
            }),
        });

        assert!(one.reciever.is_empty());

        let res = serde_json::from_str::<serde_json::Value>(
            two.reciever.recv().await.unwrap().into_text()?.as_str(),
        )?;

        assert_eq!(res["kind"], "general");
        assert_eq!(res["data"]["kind"], "chat");
        assert_eq!(res["data"]["author"], one.uuid.to_string());
        assert_eq!(res["data"]["content"], "hi");

        Ok(())
    }
}
