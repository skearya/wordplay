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
        clients::Clients,
        general::messages::ServerGeneral,
        handler::Handler,
        messenger::{ClientMessenger, RoomMessenger, client_submessenger, room_submessenger},
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
        sender: mpsc::UnboundedSender<RoomMessage>,
        reciever: mpsc::UnboundedReceiver<RoomMessage>,
    ) -> Self {
        Self {
            state: State::default(),
            settings: RoomSettings::default(),
            clients: Clients::new(sender.clone()),
            sender: RoomSender::new(sender),
            reciever,
            close: false,
        }
    }

    pub fn spawn() -> mpsc::UnboundedSender<RoomMessage> {
        let (sender, reciever) = mpsc::unbounded_channel::<RoomMessage>();

        task::spawn(Self::new(sender.clone(), reciever).run());

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
            RoomMessage::General(message) => self.handle_message(message),
            RoomMessage::Lobby(message) => self.state.try_lobby()?.handle_message(
                client_submessenger!(&self.clients, ServerMessage::Lobby(ServerLobby)),
                room_submessenger!(self.sender.clone(), RoomMessage::Lobby(LobbyMessage)),
                message,
            ),
            RoomMessage::InGame(message) => self.state.try_in_game()?.handle_message(
                client_submessenger!(&self.clients, ServerMessage::InGame(ServerGame)),
                room_submessenger!(self.sender.clone(), RoomMessage::InGame(GameMessage)),
                message,
            ),
        }
    }

    fn client(&mut self, uuid: Uuid, message: ClientMessage) -> anyhow::Result<Option<State>> {
        match message {
            ClientMessage::General(message) => self.handle_client((uuid, message)),
            ClientMessage::Lobby(message) => self.state.try_lobby()?.handle_client(
                client_submessenger!(&self.clients, ServerMessage::Lobby(ServerLobby)),
                room_submessenger!(self.sender.clone(), RoomMessage::Lobby(LobbyMessage)),
                (uuid, message),
            ),
            ClientMessage::InGame(message) => self.state.try_in_game()?.handle_client(
                client_submessenger!(&self.clients, ServerMessage::InGame(ServerGame)),
                room_submessenger!(self.sender.clone(), RoomMessage::InGame(GameMessage)),
                (uuid, message),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;
    use uuid::Uuid;

    use crate::{
        messages::{ClientMessage, RoomMessage},
        room::{
            Room,
            general::messages::{ClientGeneral, GeneralMessage},
        },
    };

    #[tokio::test(start_paused = true)]
    async fn test() -> anyhow::Result<()> {
        let room = Room::spawn();

        let uuid1 = Uuid::new_v4();
        let (sender1, _reciever1) = mpsc::unbounded_channel();

        let uuid2 = Uuid::new_v4();
        let (sender2, mut reciever2) = mpsc::unbounded_channel();

        room.send(RoomMessage::General(GeneralMessage::Joined {
            uuid: uuid1,
            sender: sender1,
        }))?;

        room.send(RoomMessage::General(GeneralMessage::Joined {
            uuid: uuid2,
            sender: sender2,
        }))?;

        room.send(RoomMessage::Client {
            uuid: uuid1,
            message: ClientMessage::General(ClientGeneral::ChatMessage {
                content: "hi".to_owned(),
            }),
        })?;

        assert!(matches!(
            serde_json::from_str::<ClientMessage>(
                reciever2.recv().await.unwrap().into_text()?.as_str(),
            )?,
            ClientMessage::General(ClientGeneral::ChatMessage { content }) if content == "hi"
        ));

        Ok(())
    }
}
