use crate::{
    game::{Countdown, GameState, GuessInfo},
    messages::{
        CountdownState, InvalidWordReason, PlayerData, PlayerInfo, PlayerUpdate, RoomState,
        ServerMessage,
    },
    Params,
};

use axum::extract::ws::{close_code, CloseFrame, Message};
use futures::{future::BoxFuture, FutureExt};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc::UnboundedSender;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<Mutex<AppStateInner>>,
}

struct AppStateInner {
    rooms: HashMap<String, Room>,
}

#[derive(Default)]
pub struct Room {
    pub clients: HashMap<Uuid, Client>,
    pub state: GameState,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub socket: Option<Uuid>,
    pub tx: UnboundedSender<Message>,
    pub username: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AppStateInner {
                rooms: HashMap::new(),
            })),
        }
    }

    pub fn clients_connected(&self) -> usize {
        let lock = self.inner.lock().unwrap();
        lock.rooms.values().map(|room| room.clients.len()).sum()
    }

    pub fn errored(&self, room: &str, uuid: Uuid) -> Option<()> {
        let lock = self.inner.lock().unwrap();
        let Room { clients, .. } = lock.rooms.get(room)?;

        clients[&uuid]
            .tx
            .send(
                ServerMessage::ServerMessage {
                    content: "something went wrong...".into(),
                }
                .into(),
            )
            .ok();

        Some(())
    }

    pub fn add_client(
        &self,
        room: &str,
        params: Params,
        socket_uuid: Uuid,
        tx: UnboundedSender<Message>,
    ) -> Uuid {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.entry(room.to_string()).or_default();

        let uuid = match params.rejoin_token.and_then(|rejoin_token| {
            if let GameState::InGame(game) = state {
                game.players
                    .iter()
                    .find(|player| rejoin_token == player.rejoin_token)
                    .map(|player| player.uuid)
            } else {
                None
            }
        }) {
            Some(uuid) => {
                clients.broadcast(ServerMessage::PlayerUpdate {
                    uuid,
                    state: PlayerUpdate::Reconnected {
                        username: params.username.clone(),
                    },
                });

                uuid
            }
            None => Uuid::new_v4(),
        };

        clients.broadcast(ServerMessage::ServerMessage {
            content: format!("{} has joined", params.username),
        });

        let old_client = clients.insert(
            uuid,
            Client {
                socket: Some(socket_uuid),
                tx,
                username: params.username,
            },
        );

        if let Some(old_client) = old_client.filter(|client| client.socket.is_some()) {
            old_client
                .tx
                .send(Message::Close(Some(CloseFrame {
                    code: close_code::ABNORMAL,
                    reason: Cow::from("Connected on another client?"),
                })))
                .ok();
        };

        let room_state = match state {
            GameState::Lobby(lobby) => RoomState::Lobby {
                ready: lobby
                    .ready
                    .iter()
                    .map(|uuid| PlayerInfo {
                        uuid: *uuid,
                        username: clients[uuid].username.clone(),
                    })
                    .collect(),
                starting_countdown: lobby
                    .countdown
                    .as_ref()
                    .map(|countdown| countdown.time_left),
            },
            GameState::InGame(game) => RoomState::InGame {
                prompt: game.prompt.clone(),
                turn: game.current_turn,
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
                ServerMessage::RoomInfo {
                    uuid,
                    state: room_state,
                }
                .into(),
            )
            .ok();

        uuid
    }

    pub fn remove_client(&self, room: &str, uuid: Uuid, socket_uuid: Uuid) -> Option<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(room)?;

        let client = clients.get_mut(&uuid).filter(|client| {
            client
                .socket
                .is_some_and(|client_socket| client_socket == socket_uuid)
        })?;

        let username = client.username.clone();

        client.socket = None;

        if clients
            .values()
            .filter(|client| client.socket.is_some())
            .count()
            == 0
        {
            lock.rooms.remove(room);
        } else {
            match state {
                GameState::Lobby(lobby) => {
                    clients.remove(&uuid);

                    if lobby.ready.remove(&uuid) {
                        clients.broadcast(ServerMessage::ReadyPlayers {
                            players: lobby
                                .ready
                                .iter()
                                .map(|uuid| PlayerInfo {
                                    uuid: *uuid,
                                    username: clients[uuid].username.clone(),
                                })
                                .collect(),
                        });
                    }

                    if let Some(countdown) = &mut lobby.countdown {
                        if lobby.ready.len() < 2 {
                            countdown.timer_handle.abort();
                            lobby.countdown = None;

                            clients.broadcast(ServerMessage::StartingCountdown {
                                state: CountdownState::Stopped,
                            });
                        }
                    }
                }
                GameState::InGame(_) => {
                    clients.broadcast(ServerMessage::PlayerUpdate {
                        uuid,
                        state: PlayerUpdate::Disconnected,
                    });
                }
            }

            clients.broadcast(ServerMessage::ServerMessage {
                content: format!("{username} has left"),
            });
        }

        Some(())
    }

    pub fn client_chat_message(&self, room: &str, uuid: Uuid, content: String) -> Option<()> {
        let lock = self.inner.lock().unwrap();
        let Room { clients, .. } = lock.rooms.get(room)?;

        clients.broadcast(ServerMessage::ChatMessage {
            author: clients[&uuid].username.clone(),
            content,
        });

        Some(())
    }

    pub fn client_ready(&self, room: &str, uuid: Uuid) -> Option<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(room)?;

        let lobby = state.try_lobby()?;

        lobby.ready.insert(uuid);

        if lobby.ready.len() >= 2 {
            if let Some(countdown) = &lobby.countdown {
                countdown.timer_handle.abort();
            }

            let app_state = self.clone();
            let room = room.to_string();

            lobby.countdown = Some(Countdown {
                timer_handle: Arc::new(
                    tokio::task::spawn(async move {
                        app_state.start_when_ready(room).await;
                    })
                    .abort_handle(),
                ),
                time_left: 10,
            });
        }

        clients.broadcast(ServerMessage::ReadyPlayers {
            players: lobby
                .ready
                .iter()
                .map(|uuid| PlayerInfo {
                    uuid: *uuid,
                    username: clients[uuid].username.clone(),
                })
                .collect(),
        });

        Some(())
    }

    pub async fn start_when_ready(&self, room: String) -> Option<()> {
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let mut lock = self.inner.lock().unwrap();
            let Room { clients, state } = lock.rooms.get_mut(&room)?;

            let lobby = state.try_lobby()?;
            let countdown = lobby.countdown.as_mut()?;

            countdown.time_left -= 1;

            if countdown.time_left == 0 {
                if lobby.ready.len() < 2 {
                    return None;
                }

                let app_state = self.clone();
                let room = room.clone();

                *state = lobby.start_game(move |prompt, timer_len| {
                    Arc::new(
                        tokio::task::spawn(async move {
                            app_state.check_for_timeout(room, timer_len, prompt).await;
                        })
                        .abort_handle(),
                    )
                });

                let game = state.try_in_game()?;

                let players: Vec<PlayerData> = game
                    .players
                    .iter()
                    .map(|player| PlayerData {
                        uuid: player.uuid,
                        username: clients[&player.uuid].username.clone(),
                        input: player.input.clone(),
                        lives: player.lives,
                        disconnected: !clients.contains_key(&player.uuid),
                    })
                    .collect();

                for (uuid, client) in clients {
                    client
                        .tx
                        .send(
                            ServerMessage::GameStarted {
                                rejoin_token: game
                                    .players
                                    .iter()
                                    .find(|player| *uuid == player.uuid)
                                    .map(|player| player.rejoin_token),
                                prompt: game.prompt.clone(),
                                turn: game.current_turn,
                                players: players.clone(),
                            }
                            .into(),
                        )
                        .ok();
                }
            } else {
                clients.broadcast(ServerMessage::StartingCountdown {
                    state: CountdownState::InProgress {
                        time_left: countdown.time_left,
                    },
                });
            }
        }

        Some(())
    }

    pub fn client_input_update(&self, room: &str, uuid: Uuid, new_input: String) -> Option<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(room)?;

        let game = state.try_in_game()?;
        let player = game.players.iter_mut().find(|player| player.uuid == uuid)?;

        player.input = new_input.clone();

        clients.broadcast(ServerMessage::InputUpdate {
            uuid,
            input: new_input,
        });

        Some(())
    }

    pub fn client_guess(&self, room: &str, uuid: Uuid, guess: &str) -> Option<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(room)?;

        let game = state.try_in_game()?;

        if game.current_turn != uuid {
            return None;
        }

        match game.parse_prompt(guess) {
            GuessInfo::Valid(life_change) => {
                game.new_prompt();
                game.update_turn();
                game.update_timer_len();

                clients.broadcast(ServerMessage::NewPrompt {
                    life_change,
                    word: Some(guess.to_string()),
                    new_prompt: game.prompt.clone(),
                    new_turn: game.current_turn,
                });

                game.timeout_task.abort();

                let app_state = self.clone();
                let room = room.to_string();
                let timer_len = game.timer_len;
                let current_prompt = game.prompt.clone();

                game.timeout_task = Arc::new(
                    tokio::task::spawn(async move {
                        app_state
                            .check_for_timeout(room, timer_len, current_prompt)
                            .await;
                    })
                    .abort_handle(),
                );
            }
            guess_info => {
                let reason = match guess_info {
                    GuessInfo::PromptNotIn => InvalidWordReason::PromptNotIn,
                    GuessInfo::NotEnglish => InvalidWordReason::NotEnglish,
                    GuessInfo::AlreadyUsed => InvalidWordReason::AlreadyUsed,
                    GuessInfo::Valid(_) => unreachable!(),
                };

                clients.broadcast(ServerMessage::InvalidWord { uuid, reason });
            }
        };

        Some(())
    }

    fn check_for_timeout(
        &self,
        room: String,
        timer_len: u8,
        original_prompt: String,
    ) -> BoxFuture<'_, Option<()>> {
        async move {
            tokio::time::sleep(Duration::from_secs(timer_len.into())).await;

            let mut lock = self.inner.lock().unwrap();
            let Room { clients, state } = lock.rooms.get_mut(&room)?;

            let game = state.try_in_game()?;

            if original_prompt == game.prompt {
                game.player_timed_out();

                if game.alive_players().len() == 1 {
                    clients.broadcast(ServerMessage::GameEnded {
                        winner: game.alive_players().first()?.uuid,
                    });

                    *state = game.end();

                    clients.retain(|_uuid, client| client.socket.is_some());
                } else {
                    clients.broadcast(ServerMessage::NewPrompt {
                        life_change: -1,
                        word: None,
                        new_prompt: game.prompt.clone(),
                        new_turn: game.current_turn,
                    });

                    let app_state = self.clone();
                    let timer_len = game.timer_len;
                    let current_prompt = game.prompt.clone();

                    game.timeout_task = Arc::new(
                        tokio::task::spawn(async move {
                            app_state
                                .check_for_timeout(room, timer_len, current_prompt)
                                .await;
                        })
                        .abort_handle(),
                    );
                }
            }

            Some(())
        }
        .boxed()
    }
}

pub trait Broadcast {
    fn broadcast(&self, message: ServerMessage);
}

impl Broadcast for HashMap<Uuid, Client> {
    fn broadcast(&self, message: ServerMessage) {
        let serialized = serde_json::to_string(&message).unwrap();

        for client in self.values().filter(|client| client.socket.is_some()) {
            client.tx.send(Message::Text(serialized.clone())).ok();
        }
    }
}
