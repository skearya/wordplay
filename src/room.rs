pub mod clients;

use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    games::word_bomb::WordBomb,
    general::{self, messages::ServerGeneral},
    lobby::Lobby,
    messages::{ClientMessage, RoomMessage, ServerMessage},
    room::clients::{Clients, Messenger, submessenger},
    task,
};

pub struct Room {
    /// Sender to our own room task.
    sender: mpsc::UnboundedSender<RoomMessage>,
    reciever: mpsc::UnboundedReceiver<RoomMessage>,
    clients: Clients,
}

enum State {
    Lobby(Lobby),
    WordBomb(WordBomb),
}

// TODO: Reuse message?
struct Settings {
    public: bool,
}

impl State {
    fn name(&self) -> &'static str {
        match self {
            State::Lobby(_) => "lobby",
            State::WordBomb(_) => "word bomb",
        }
    }

    fn try_lobby(&mut self) -> anyhow::Result<&mut Lobby> {
        if let Self::Lobby(lobby) = self {
            Ok(lobby)
        } else {
            Err(anyhow::anyhow!(
                "expected to be in lobby, in {}",
                self.name()
            ))
        }
    }

    fn try_word_bomb(&mut self) -> anyhow::Result<&mut WordBomb> {
        if let Self::WordBomb(word_bomb) = self {
            Ok(word_bomb)
        } else {
            Err(anyhow::anyhow!(
                "expected to be in word bomb, in {}",
                self.name()
            ))
        }
    }
}

impl Room {
    pub fn new(
        sender: mpsc::UnboundedSender<RoomMessage>,
        reciever: mpsc::UnboundedReceiver<RoomMessage>,
    ) -> Self {
        Self {
            sender: sender.clone(),
            reciever,
            clients: Clients::new(sender),
        }
    }

    pub fn spawn() -> mpsc::UnboundedSender<RoomMessage> {
        let (sender, reciever) = mpsc::unbounded_channel::<RoomMessage>();

        task::spawn(Self::new(sender.clone(), reciever).run());

        sender
    }

    async fn run(mut self) -> anyhow::Result<()> {
        // TODO: Struct field?
        let mut state = State::Lobby(Lobby::new());

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
                    let _ = self.client(&mut state, uuid, message);
                }
                RoomMessage::CloseCheck => {
                    if self.clients.is_empty() {
                        break;
                    }
                }
            };
        }

        Ok(())
    }

    fn client(&self, state: &mut State, uuid: Uuid, message: ClientMessage) -> anyhow::Result<()> {
        let res = match message {
            ClientMessage::General(client_general) => general::handle_client(
                submessenger!(&self.clients, ServerMessage::General(ServerGeneral)),
                (uuid, client_general),
            ),
            ClientMessage::Lobby(client_lobby) => {
                let lobby = state.try_lobby()?;

                // lobby.handle(
                //     submessenger!(&self.clients, ServerMessage::Lobby(ServerLobby)),
                //     uuid,
                //     client_lobby,
                // )

                Ok(())
            }
            ClientMessage::InGame(client_in_game) => todo!(),
            ClientMessage::WordBomb(client_word_bomb) => todo!(),
        };

        if let Err(err) = &res {
            self.clients.send(
                uuid,
                &ServerMessage::General(ServerGeneral::Error {
                    message: err.to_string(),
                }),
            );
        }

        res
    }
}
