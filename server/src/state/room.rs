use super::{
    games::{anagrams::Anagrams, word_bomb::WordBomb},
    lobby::{check_for_countdown_update, Lobby},
    AppState, SenderInfo,
};
use crate::{
    messages::{ClientInfo, ConnectionUpdate, Games, RoomInfo, RoomStateInfo, ServerMessage},
    Params,
};

use anyhow::{anyhow, Context, Result};
use axum::extract::ws::{close_code, CloseFrame, Message};
use rand::{seq::IteratorRandom, thread_rng};
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Default)]
pub struct Room {
    pub owner: Uuid,
    pub settings: RoomSettings,
    pub clients: HashMap<Uuid, Client>,
    pub state: State,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RoomSettings {
    pub public: bool,
    pub game: Games,
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            public: false,
            game: Games::WordBomb,
        }
    }
}

pub struct Client {
    pub socket: Option<Uuid>,
    pub tx: UnboundedSender<Message>,
    pub username: String,
    pub rejoin_token: Option<Uuid>,
}

pub enum State {
    Lobby(Lobby),
    WordBomb(WordBomb),
    Anagrams(Anagrams),
}

impl Default for State {
    fn default() -> Self {
        Self::Lobby(Lobby::new())
    }
}

impl State {
    pub fn try_lobby(&mut self) -> Result<&mut Lobby> {
        match self {
            State::Lobby(lobby) => Ok(lobby),
            _ => Err(anyhow!("Not in lobby")),
        }
    }

    pub fn try_word_bomb(&mut self) -> Result<&mut WordBomb> {
        match self {
            State::WordBomb(game) => Ok(game),
            _ => Err(anyhow!("Not in word bomb")),
        }
    }

    pub fn try_anagrams(&mut self) -> Result<&mut Anagrams> {
        match self {
            State::Anagrams(game) => Ok(game),
            _ => Err(anyhow!("Not in anagrams")),
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
            settings,
        } = lock.rooms.entry(room.to_string()).or_default();

        let prev_client = params.rejoin_token.and_then(|rejoin_token| {
            clients.iter_mut().find(|(_uuid, client)| {
                client
                    .rejoin_token
                    .is_some_and(|client_token| client_token == rejoin_token)
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

            let uuid = *prev_uuid;

            clients.broadcast(ServerMessage::ConnectionUpdate {
                uuid,
                state: ConnectionUpdate::Reconnected {
                    username: params.username,
                },
            });

            uuid
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
                    rejoin_token: None,
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

        clients[&uuid]
            .tx
            .send(
                ServerMessage::Info {
                    uuid,
                    room: RoomInfo {
                        owner: *owner,
                        settings: settings.clone(),
                        clients: clients
                            .iter()
                            .map(|(uuid, client)| ClientInfo {
                                uuid: *uuid,
                                username: client.username.clone(),
                                disconnected: client.socket.is_none(),
                            })
                            .collect(),
                        state: room_state_info(state, uuid),
                    },
                }
                .into(),
            )
            .ok();

        uuid
    }

    pub fn remove_client(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        socket_uuid: Uuid,
    ) -> Result<()> {
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

        if clients.connected().count() == 0 {
            match state {
                State::WordBomb(game) => game.timer.task.abort(),
                State::Anagrams(game) => game.timer.abort(),
                _ => {}
            }

            lock.rooms.remove(room);
            return Ok(());
        }

        match state {
            State::Lobby(lobby) => {
                clients.remove(&uuid);

                if lobby.ready.remove(&uuid) {
                    let countdown_update =
                        check_for_countdown_update(self.clone(), room.to_string(), lobby);

                    clients.broadcast(ServerMessage::ReadyPlayers {
                        ready: lobby.ready.iter().copied().collect(),
                        countdown_update,
                    });
                }

                let new_room_owner = check_for_new_room_owner(clients, owner);

                clients.broadcast(ServerMessage::ConnectionUpdate {
                    uuid,
                    state: ConnectionUpdate::Disconnected { new_room_owner },
                });
            }
            _ => {
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

    pub fn client_chat_message(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        content: String,
    ) -> Result<()> {
        let lock = self.inner.lock().unwrap();
        let Room { clients, .. } = lock.room(room)?;

        clients.broadcast(ServerMessage::ChatMessage {
            author: uuid,
            content,
        });

        Ok(())
    }

    pub fn send_error_msg(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        message: &str,
    ) -> Result<()> {
        let lock = self.inner.lock().unwrap();
        let Room { clients, .. } = lock.room(room)?;

        clients[&uuid]
            .tx
            .send(
                ServerMessage::Error {
                    content: format!("something went wrong: {message}"),
                }
                .into(),
            )
            .ok();

        Ok(())
    }
}

fn room_state_info(state: &State, uuid: Uuid) -> RoomStateInfo {
    match state {
        State::Lobby(lobby) => RoomStateInfo::Lobby {
            ready: lobby.ready.iter().copied().collect(),
            starting_countdown: lobby
                .countdown
                .as_ref()
                .map(|countdown| countdown.time_left),
        },
        State::WordBomb(game) => RoomStateInfo::WordBomb {
            players: game.players.clone(),
            turn: game.turn,
            prompt: game.prompt.clone(),
            used_letters: game
                .players
                .iter()
                .find(|player| uuid == player.uuid)
                .map(|player| player.used_letters.clone()),
        },
        State::Anagrams(game) => RoomStateInfo::Anagrams {
            players: game.players.clone(),
            anagram: game.anagram.clone(),
        },
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

pub trait ClientUtils {
    fn connected(&self) -> impl Iterator<Item = (&Uuid, &Client)>;
    fn send_each(&self, f: impl Fn(&Uuid, &Client) -> ServerMessage);
    fn broadcast(&self, message: ServerMessage);
}

impl ClientUtils for HashMap<Uuid, Client> {
    fn connected(&self) -> impl Iterator<Item = (&Uuid, &Client)> {
        self.iter().filter(|client| client.1.socket.is_some())
    }

    fn send_each(&self, f: impl Fn(&Uuid, &Client) -> ServerMessage) {
        for (uuid, client) in self.connected() {
            client.tx.send(f(uuid, client).into()).ok();
        }
    }

    fn broadcast(&self, message: ServerMessage) {
        let serialized: Message = message.into();

        for (_uuid, client) in self.connected() {
            client.tx.send(serialized.clone()).ok();
        }
    }
}
