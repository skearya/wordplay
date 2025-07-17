use std::{collections::HashMap, time::Duration};

use axum::extract::ws::{self, Utf8Bytes};
use tokio::{sync::mpsc, task::AbortHandle};
use uuid::Uuid;

use crate::{
    messages::{
        client::{ClientGeneral, ClientLobby, ClientMessage},
        server::{ServerGeneral, ServerLobby, ServerMessage},
    },
    task,
};

pub enum RoomMessage {
    Joined {
        uuid: Uuid,
        sender: mpsc::UnboundedSender<ws::Message>,
    },
    Left {
        uuid: Uuid,
    },
    Client {
        uuid: Uuid,
        message: ClientMessage,
    },
    CloseCheck,
}

pub struct Room {
    /// Sender to our own room task.
    sender: mpsc::UnboundedSender<RoomMessage>,
    reciever: mpsc::UnboundedReceiver<RoomMessage>,
    clients: Clients,
}

struct Clients {
    room: mpsc::UnboundedSender<RoomMessage>,
    clients: HashMap<Uuid, Client>,
}

struct Client {
    sender: mpsc::UnboundedSender<ws::Message>,
}

struct Lobby {
    ready: Vec<Uuid>,
    countdown: Option<Countdown>,
}

struct Countdown {
    timer_handle: AbortHandle,
}

impl Lobby {
    fn new() -> Self {
        Self {
            ready: vec![],
            countdown: None,
        }
    }

    fn handle(&mut self, uuid: Uuid, message: ClientLobby) -> anyhow::Result<()> {
        match message {
            ClientLobby::RoomSettings(room_settings) => todo!(),
            ClientLobby::Ready => todo!(),
            ClientLobby::StartEarly => todo!(),
            ClientLobby::Unready => todo!(),
            ClientLobby::PracticeRequest => todo!(),
            ClientLobby::PracticeSubmission { prompt, input } => todo!(),
        }

        Ok(())
    }
}

enum State {
    Lobby(Lobby),
}

impl State {
    fn name(&self) -> &'static str {
        match self {
            State::Lobby(_) => "lobby",
        }
    }

    fn try_lobby(&mut self) -> anyhow::Result<&mut Lobby> {
        if let Self::Lobby(v) = self {
            Ok(v)
        } else {
            Err(anyhow::anyhow!(
                "expected to be in lobby, in {}",
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
        let mut state = State::Lobby(Lobby::new());

        while let Some(message) = self.reciever.recv().await {
            match message {
                RoomMessage::Joined { uuid, sender } => {
                    self.clients.add(uuid, Client { sender });
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
            ClientMessage::General(client_general) => self.general(uuid, client_general),
            ClientMessage::Lobby(client_lobby) => {
                let lobby = state.try_lobby()?;

                lobby.handle(uuid, client_lobby)
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

    fn general(&self, uuid: Uuid, message: ClientGeneral) -> anyhow::Result<()> {
        match message {
            ClientGeneral::Ping { timestamp } => todo!(),
            ClientGeneral::ChatMessage { content } => {
                self.clients
                    .broadcast(&ServerMessage::General(ServerGeneral::Chat {
                        author: uuid,
                        content,
                    }));
            }
        }

        Ok(())
    }
}

impl Clients {
    fn new(room: mpsc::UnboundedSender<RoomMessage>) -> Self {
        Self {
            room,
            clients: HashMap::new(),
        }
    }

    fn add(&mut self, uuid: Uuid, client: Client) {
        self.clients.insert(uuid, client);

        self.broadcast(&ServerMessage::General(ServerGeneral::Join { uuid }));
    }

    fn remove(&mut self, uuid: Uuid) {
        if self.clients.remove(&uuid).is_some() {
            self.broadcast(&ServerMessage::General(ServerGeneral::Leave { uuid }));

            if self.clients.is_empty() {
                let room = self.room.clone();

                task::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(5)).await;

                    room.send(RoomMessage::CloseCheck)?;

                    Ok(())
                });
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.clients.is_empty()
    }

    fn send(&self, uuid: Uuid, message: &ServerMessage) {
        let text = serde_json::to_string(&message)
            .expect("ServerMessage serialization shouldn't ever fail?");

        let message = ws::Message::Text(Utf8Bytes::from(text));

        self.clients[&uuid]
            .sender
            .send(message)
            .expect("client sender shouldn't be closed");
    }

    fn broadcast(&self, message: &ServerMessage) {
        let text = serde_json::to_string(&message)
            .expect("ServerMessage serialization shouldn't ever fail?");

        let message = ws::Message::Text(Utf8Bytes::from(text));

        for client in self.clients.values() {
            client
                .sender
                .send(message.clone())
                .expect("client sender shouldn't be closed");
        }
    }
}
