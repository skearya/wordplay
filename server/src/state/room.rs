use crate::{
    routes::game::Params,
    state::{
        error::{GameError, Result, RoomError},
        games::{
            anagrams::Anagrams,
            word_bomb::{WordBomb, WordBombSettings},
        },
        lobby::{check_for_countdown_update, Lobby},
        messages::{ClientInfo, ConnectionUpdate, Games, RoomInfo, RoomStateInfo, ServerMessage},
        SenderInfo,
    },
    utils::ClientUtils,
    AppState,
};
use axum::extract::ws::{close_code, CloseFrame, Message};
use rand::{seq::IteratorRandom, thread_rng};
use rustrict::CensorStr;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct Room {
    pub owner: Uuid,
    pub settings: RoomSettings,
    pub clients: HashMap<Uuid, Client>,
    pub state: State,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomSettings {
    pub game: Games,
    pub public: bool,
    pub word_bomb: WordBombSettings,
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            public: false,
            game: Games::WordBomb,
            word_bomb: WordBombSettings { min_wpm: 500 },
        }
    }
}

#[derive(Debug)]
pub struct Client {
    pub socket: Option<Uuid>,
    pub tx: UnboundedSender<Message>,
    pub username: String,
    pub rejoin_token: Option<Uuid>,
}

impl Client {
    pub fn send(&self, message: ServerMessage) {
        self.tx.send(message.into()).ok();
    }

    pub fn close(&self, close_frame: Option<CloseFrame<'static>>) {
        self.tx.send(Message::Close(close_frame)).ok();
    }
}

#[derive(Debug)]
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
    pub fn try_lobby(&mut self) -> Result<&mut Lobby, GameError> {
        match self {
            State::Lobby(lobby) => Ok(lobby),
            _ => Err(GameError::InvalidState { state: "lobby" }),
        }
    }

    pub fn try_word_bomb(&mut self) -> Result<&mut WordBomb, GameError> {
        match self {
            State::WordBomb(game) => Ok(game),
            _ => Err(GameError::InvalidState { state: "word bomb" }),
        }
    }

    pub fn try_anagrams(&mut self) -> Result<&mut Anagrams, GameError> {
        match self {
            State::Anagrams(game) => Ok(game),
            _ => Err(GameError::InvalidState { state: "anagrams" }),
        }
    }
}

impl AppState {
    pub fn room_full(&self, room: &str) -> bool {
        self.room(room).is_ok_and(|room| room.clients.len() >= 8)
    }

    pub fn add_client(
        &self,
        room: &str,
        params: Params,
        socket_uuid: Uuid,
        tx: UnboundedSender<Message>,
    ) -> Uuid {
        let mut lock = self.rooms.entry(room.to_string()).or_default();
        let Room {
            clients,
            state,
            owner,
            settings,
        } = lock.value_mut();

        let prev_client = params.rejoin_token.and_then(|rejoin_token| {
            clients.iter_mut().find(|(_uuid, client)| {
                client
                    .rejoin_token
                    .is_some_and(|client_token| client_token == rejoin_token)
            })
        });

        let (uuid, connection_update) = if let Some((prev_uuid, client)) = prev_client {
            if client.socket.is_some() {
                client.close(Some(CloseFrame {
                    code: close_code::ABNORMAL,
                    reason: Cow::from("connected on another client?"),
                }));
            }

            client.socket = Some(socket_uuid);
            client.tx = tx;
            client.username.clone_from(&params.username);

            let uuid = *prev_uuid;

            (
                uuid,
                ConnectionUpdate::Reconnected {
                    username: params.username,
                },
            )
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

            (
                uuid,
                ConnectionUpdate::Connected {
                    username: params.username.clone(),
                },
            )
        };

        clients.broadcast_except(
            ServerMessage::ConnectionUpdate {
                uuid,
                state: connection_update,
            },
            &[uuid],
        );

        clients[&uuid].send(ServerMessage::Info {
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
        });

        uuid
    }

    pub fn remove_client(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        socket_id: Uuid,
    ) -> Result<()> {
        let mut lock = self.room_mut(room)?;
        let Room {
            owner,
            clients,
            state,
            ..
        } = lock.value_mut();

        let client = clients
            .get_mut(&uuid)
            .ok_or(RoomError::CouldntFindClientToRemove)?;

        if !client
            .socket
            .is_some_and(|client_socket_id| client_socket_id == socket_id)
        {
            return Err(RoomError::SocketUuidMismatchWhileRemoving)?;
        }

        client.socket = None;

        if clients.connected().count() == 0 {
            match state {
                State::WordBomb(game) => game.timer.task.abort(),
                State::Anagrams(game) => game.timer.abort(),
                State::Lobby(_) => {}
            }

            drop(lock);
            self.rooms.remove(room);
            return Ok(());
        }

        if let State::Lobby(lobby) = state {
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
        } else {
            clients.broadcast(ServerMessage::ConnectionUpdate {
                uuid,
                state: ConnectionUpdate::Disconnected {
                    new_room_owner: None,
                },
            });
        }

        Ok(())
    }

    pub fn client_ping(&self, SenderInfo { uuid, room }: SenderInfo, timestamp: u64) -> Result<()> {
        let lock = self.room(room)?;
        let Room { clients, .. } = lock.value();

        clients[&uuid].send(ServerMessage::Pong { timestamp });

        Ok(())
    }

    pub fn client_chat_message(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        mut content: String,
    ) -> Result<()> {
        if content.len() > 250 {
            return Err(RoomError::ChatMessageTooLong)?;
        }

        let lock = self.room(room)?;
        let Room {
            settings, clients, ..
        } = lock.value();

        if settings.public {
            content = content.censor();
        }

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
        let lock = self.room(room)?;
        let Room { clients, .. } = lock.value();

        clients[&uuid].send(ServerMessage::Error {
            content: format!("server error: {message}"),
        });

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
            prompt: game.prompt.to_string(),
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
