pub mod clients;
pub mod messenger;
pub mod state;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    games::word_bomb::messages::WordBombSettings,
    general::{self, messages::ServerGeneral},
    lobby::{
        Lobby,
        messages::{LobbyMessage, ServerLobby},
    },
    messages::{ClientMessage, RoomMessage, RoomSettings, ServerMessage},
    room::{
        clients::Clients,
        messenger::{ClientMessenger, client_submessenger, submessenger},
        state::State,
    },
    task,
};

pub struct Room {
    state: State,
    clients: Clients,
    settings: RoomSettings,
    /// Sender to our own room task.
    sender: mpsc::UnboundedSender<RoomMessage>,
    /// Reciever of `RoomMessages`.
    reciever: mpsc::UnboundedReceiver<RoomMessage>,
}

impl Room {
    pub fn new(
        sender: mpsc::UnboundedSender<RoomMessage>,
        reciever: mpsc::UnboundedReceiver<RoomMessage>,
    ) -> Self {
        Self {
            state: State::Lobby(Lobby::new()),
            settings: RoomSettings {
                public: false,
                word_bomb: WordBombSettings {},
            },
            clients: Clients::new(sender.clone()),
            sender,
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
                RoomMessage::Joined { uuid, sender } => {
                    self.clients.add(uuid, sender);
                }
                RoomMessage::Left { uuid } => {
                    self.clients.remove(uuid);
                }
                RoomMessage::Client { uuid, message } => {
                    // TODO: Handle
                    let _ = self.client(uuid, message);
                }
                RoomMessage::CloseCheck => {
                    if self.clients.is_empty() {
                        break;
                    }
                }
                RoomMessage::Lobby(lobby_message) => todo!(),
            };
        }

        Ok(())
    }

    fn client(&mut self, uuid: Uuid, message: ClientMessage) -> anyhow::Result<()> {
        let res = match message {
            ClientMessage::General(client_general) => {
                general::handle_client(&mut self.clients, (uuid, client_general))
            }
            // ClientMessage::Lobby(client_lobby) => {
            //     let lobby = self.state.try_lobby()?;

            //     lobby.handle_client(
            //         &mut self.settings,
            //         self.sender.clone(),
            //         submessenger!(&self.clients, ServerMessage::Lobby(ServerLobby)),
            //         (uuid, client_lobby),
            //     )

            //     // let methods return a new state
            // }
            // ClientMessage::InGame(client_in_game) => todo!(),
            // ClientMessage::WordBomb(client_word_bomb) => todo!(),
            _ => panic!(),
        };

        // if let Err(err) = &res {
        //     self.clients.send(
        //         uuid,
        //         &ServerMessage::General(ServerGeneral::Error {
        //             message: err.to_string(),
        //         }),
        //     );
        // }

        // res

        todo!()
    }
}
