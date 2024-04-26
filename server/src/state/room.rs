use super::{
    games::word_bomb::WordBomb,
    lobby::{check_for_countdown_update, Lobby},
    AppState, ClientUtils,
};
use crate::{
    messages::{ClientInfo, ConnectionUpdate, PlayerData, RoomInfo, RoomStateInfo, ServerMessage},
    Params,
};

use anyhow::{anyhow, Context, Result};
use axum::extract::ws::{close_code, CloseFrame, Message};
use rand::{seq::IteratorRandom, thread_rng};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Default)]
pub struct Room {
    pub public: bool,
    pub owner: Uuid,
    pub clients: HashMap<Uuid, Client>,
    pub state: State,
}

pub struct Client {
    pub socket: Option<Uuid>,
    pub tx: UnboundedSender<Message>,
    pub username: String,
}

pub enum State {
    Lobby(Lobby),
    WordBomb(WordBomb),
}

impl Default for State {
    fn default() -> Self {
        State::Lobby(Lobby {
            ready: HashSet::new(),
            countdown: None,
        })
    }
}

impl State {
    pub fn try_lobby(&mut self) -> Result<&mut Lobby> {
        match self {
            State::Lobby(lobby) => Ok(lobby),
            State::WordBomb(_) => Err(anyhow!("Not in lobby")),
        }
    }

    pub fn try_word_bomb(&mut self) -> Result<&mut WordBomb> {
        match self {
            State::Lobby(_) => Err(anyhow!("Not in game")),
            State::WordBomb(game) => Ok(game),
        }
    }
}

impl AppState {
    pub fn add_client(
        &self,
        room: &str,
        params: Params,
        socket_uuid: Uuid,
        tx: UnboundedSender<Message>,
    ) -> Uuid {
        let mut lock = self.inner.lock().unwrap();
        let Room {
            clients,
            state,
            owner,
            public,
        } = lock.rooms.entry(room.to_string()).or_default();

        let prev_client = params.rejoin_token.and_then(|rejoin_token| {
            state.try_word_bomb().ok().and_then(|game| {
                game.players
                    .iter()
                    .find(|player| player.rejoin_token == rejoin_token)
                    .map(|player| (player.uuid, clients.get_mut(&player.uuid).unwrap()))
            })
        });

        let uuid = if let Some((prev_uuid, client)) = prev_client {
            if client.socket.is_some() {
                client
                    .tx
                    .send(Message::Close(Some(CloseFrame {
                        code: close_code::ABNORMAL,
                        reason: Cow::from("Connected on another client?"),
                    })))
                    .ok();
            }

            client.socket = Some(socket_uuid);
            client.tx = tx;
            client.username = params.username.clone();

            clients.broadcast(ServerMessage::ConnectionUpdate {
                uuid: prev_uuid,
                state: ConnectionUpdate::Reconnected {
                    username: params.username,
                },
            });

            prev_uuid
        } else {
            let uuid = Uuid::new_v4();

            if clients.is_empty() {
                *owner = uuid;
            }

            clients.insert(
                uuid,
                Client {
                    socket: Some(socket_uuid),
                    tx,
                    username: params.username.clone(),
                },
            );

            clients.broadcast(ServerMessage::ConnectionUpdate {
                uuid,
                state: ConnectionUpdate::Connected {
                    username: params.username.clone(),
                },
            });

            uuid
        };

        let room_state = match state {
            State::Lobby(lobby) => RoomStateInfo::Lobby {
                ready: lobby.ready.iter().copied().collect(),
                starting_countdown: lobby
                    .countdown
                    .as_ref()
                    .map(|countdown| countdown.time_left),
            },
            State::WordBomb(game) => RoomStateInfo::WordBomb {
                players: game
                    .players
                    .iter()
                    .map(|player| PlayerData {
                        uuid: player.uuid,
                        username: clients[&player.uuid].username.clone(),
                        input: player.input.clone(),
                        lives: player.lives,
                        disconnected: clients[&player.uuid].socket.is_none(),
                    })
                    .collect(),
                turn: game.current_turn,
                prompt: game.prompt.clone(),
                used_letters: game
                    .players
                    .iter()
                    .find(|player| uuid == player.uuid)
                    .map(|player| player.used_letters.clone()),
            },
        };

        clients[&uuid]
            .tx
            .send(
                ServerMessage::Info {
                    uuid,
                    room: RoomInfo {
                        public: *public,
                        owner: *owner,
                        clients: clients
                            .iter()
                            .filter(|client| client.1.socket.is_some())
                            .map(|(uuid, client)| ClientInfo {
                                uuid: *uuid,
                                username: client.username.clone(),
                            })
                            .collect(),
                        state: room_state,
                    },
                }
                .into(),
            )
            .ok();

        uuid
    }

    pub fn remove_client(&self, room: &str, uuid: Uuid, socket_uuid: Uuid) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room {
            clients,
            state,
            owner,
            ..
        } = lock.room_mut(room)?;

        let client = clients
            .get_mut(&uuid)
            .filter(|client| {
                client
                    .socket
                    .is_some_and(|client_socket| client_socket == socket_uuid)
            })
            .context("Couldn't remove client")?;

        client.socket = None;

        if clients
            .values()
            .filter(|client| client.socket.is_some())
            .count()
            == 0
        {
            lock.rooms.remove(room);
            return Ok(());
        }

        match state {
            State::Lobby(lobby) => {
                clients.remove(&uuid);

                let new_room_owner = check_for_new_room_owner(clients, owner);

                clients.broadcast(ServerMessage::ConnectionUpdate {
                    uuid,
                    state: ConnectionUpdate::Disconnected { new_room_owner },
                });

                if lobby.ready.remove(&uuid) {
                    let countdown_update =
                        check_for_countdown_update(self.clone(), room.to_string(), lobby);

                    clients.broadcast(ServerMessage::ReadyPlayers {
                        ready: lobby.ready.iter().copied().collect(),
                        countdown_update,
                    });
                }
            }
            State::WordBomb(_) => {
                clients.broadcast(ServerMessage::ConnectionUpdate {
                    uuid,
                    state: ConnectionUpdate::Disconnected {
                        new_room_owner: None,
                    },
                });
            }
        }

        Ok(())
    }

    pub fn client_chat_message(&self, room: &str, uuid: Uuid, content: String) -> Result<()> {
        let lock = self.inner.lock().unwrap();
        let Room { clients, .. } = lock.room(room)?;

        clients.broadcast(ServerMessage::ChatMessage {
            author: uuid,
            content,
        });

        Ok(())
    }

    pub fn send_error_msg(&self, room: &str, uuid: Uuid, message: &str) -> Result<()> {
        let lock = self.inner.lock().unwrap();
        let Room { clients, .. } = lock.room(room)?;

        clients[&uuid]
            .tx
            .send(
                ServerMessage::Text {
                    content: format!("something went wrong: {message}"),
                }
                .into(),
            )
            .ok();

        Ok(())
    }
}

pub fn check_for_new_room_owner(clients: &HashMap<Uuid, Client>, owner: &mut Uuid) -> Option<Uuid> {
    if clients.get(owner).is_some() {
        None
    } else {
        *owner = *clients.keys().choose(&mut thread_rng()).unwrap();
        Some(*owner)
    }
}
