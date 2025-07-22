pub mod clients;
pub mod general;
pub mod handler;
pub mod messenger;
pub mod state;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    in_game::messages::{InGameMessage, ServerInGame},
    lobby::messages::{LobbyMessage, ServerLobby},
    messages::{ClientMessage, RoomMessage, RoomSettings, ServerMessage},
    room::{
        clients::{Clients, RoomSender},
        general::messages::ServerGeneral,
        handler::Handler,
        messenger::{ClientMessenger, RoomMessenger, client_submessenger, room_submessenger},
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
}

impl Room {
    pub fn new(
        sender: mpsc::UnboundedSender<RoomMessage>,
        reciever: mpsc::UnboundedReceiver<RoomMessage>,
    ) -> Self {
        Self {
            state: State::default(),
            settings: RoomSettings::default(),
            clients: Clients::new(sender.clone()),
            sender: RoomSender::new(sender),
            reciever,
        }
    }

    pub fn spawn() -> mpsc::UnboundedSender<RoomMessage> {
        let (sender, reciever) = mpsc::unbounded_channel::<RoomMessage>();

        task::spawn(Self::new(sender.clone(), reciever).run());

        sender
    }

    async fn run(mut self) -> anyhow::Result<()> {
        while let Some(message) = self.reciever.recv().await {
            match message {
                RoomMessage::Client { uuid, message } => {
                    if let Err(err) = self.client(uuid, message) {
                        self.clients.send(
                            uuid,
                            ServerMessage::General(ServerGeneral::Error {
                                message: err.to_string(),
                            }),
                        );
                    }
                }
                RoomMessage::General(message) => todo!(),
                RoomMessage::Lobby(message) => todo!(),
                RoomMessage::InGame(message) => todo!(),
            };
        }

        Ok(())
    }

    fn client(&mut self, uuid: Uuid, message: ClientMessage) -> anyhow::Result<Option<State>> {
        match message {
            ClientMessage::General(message) => todo!(),
            ClientMessage::Lobby(message) => self.state.try_lobby()?.handle_client(
                client_submessenger!(&self.clients, ServerMessage::Lobby(ServerLobby)),
                room_submessenger!(self.sender.clone(), RoomMessage::Lobby(LobbyMessage)),
                (uuid, message),
            ),
            ClientMessage::InGame(message) => self.state.try_in_game()?.handle_client(
                client_submessenger!(&self.clients, ServerMessage::InGame(ServerInGame)),
                room_submessenger!(self.sender.clone(), RoomMessage::InGame(InGameMessage)),
                (uuid, message),
            ),
        }
    }
}
