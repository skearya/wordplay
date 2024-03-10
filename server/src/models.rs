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
    // note: "disconnected" might not be needed cause its true when socket_id is None
    pub disconnected: bool,
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

    pub fn add_client(
        &self,
        room: String,
        params: Params,
        socket_uuid: Uuid,
        tx: UnboundedSender<Message>,
    ) -> Uuid {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.entry(room.clone()).or_default();

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
            content: format!("{} has joined", params.username.clone()),
        });

        let old_client = clients.insert(
            uuid,
            Client {
                socket: Some(socket_uuid),
                tx,
                disconnected: false,
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
                        disconnected: clients[&player.uuid].disconnected,
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

    pub fn remove_client(&self, room: &str, uuid: Uuid, socket_uuid: Uuid) {
        let mut lock = self.inner.lock().unwrap();
        let Some(Room { clients, state }) = lock.rooms.get_mut(room) else {
            return;
        };

        let Some(client) = clients.get_mut(&uuid).filter(|client| {
            client
                .socket
                .is_some_and(|client_socket| client_socket == socket_uuid)
        }) else {
            return;
        };

        let username = client.username.clone();

        client.socket = None;
        client.disconnected = true;

        if clients
            .values()
            .filter(|client| !client.disconnected)
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
                content: format!("{} has left", username),
            });
        }
    }

    pub fn client_chat_message(&self, room: &str, uuid: Uuid, content: String) {
        let lock = self.inner.lock().unwrap();
        let Room { clients, .. } = lock.rooms.get(room).unwrap();

        clients.broadcast(ServerMessage::ChatMessage {
            author: clients[&uuid].username.clone(),
            content,
        });
    }

    pub fn client_ready(&self, room: &str, uuid: Uuid) {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(room).unwrap();

        let GameState::Lobby(lobby) = state else {
            return;
        };

        lobby.ready.insert(uuid);

        if lobby.ready.len() >= 2 {
            if let Some(countdown) = &lobby.countdown {
                countdown.timer_handle.abort();
            }

            let app_state = self.clone();
            let room = room.to_owned();

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
    }

    pub async fn start_when_ready(&self, room: String) {
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let mut lock = self.inner.lock().unwrap();
            let Room { clients, state } = lock.rooms.get_mut(&room).unwrap();

            let GameState::Lobby(lobby) = state else {
                return;
            };

            let Some(countdown) = &mut lobby.countdown else {
                return;
            };

            countdown.time_left -= 1;

            if countdown.time_left == 0 {
                if lobby.ready.len() < 2 {
                    return;
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

                let GameState::InGame(game) = state else {
                    unreachable!();
                };

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

                // TODO: spectators stuck in lobby
                for player in &game.players {
                    clients[&player.uuid]
                        .tx
                        .send(
                            ServerMessage::GameStarted {
                                rejoin_token: player.rejoin_token,
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
    }

    pub fn client_input_update(&self, room: &str, uuid: Uuid, new_input: String) {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(room).unwrap();

        let GameState::InGame(game) = state else {
            return;
        };

        game.players
            .iter_mut()
            .find(|player| player.uuid == uuid)
            .unwrap()
            .input = new_input.clone();

        clients.broadcast(ServerMessage::InputUpdate {
            uuid,
            input: new_input,
        });
    }

    pub fn client_guess(&self, room: &str, uuid: Uuid, guess: &str) {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(room).unwrap();

        let GameState::InGame(game) = state else {
            return;
        };

        if game.current_turn != uuid {
            return;
        }

        match game.parse_prompt(guess) {
            GuessInfo::Valid(life_change) => {
                game.new_prompt();
                game.update_turn();
                game.update_timer_len();

                clients.broadcast(ServerMessage::NewPrompt {
                    life_change,
                    prompt: game.prompt.clone(),
                    turn: game.current_turn,
                });

                game.timeout_task.abort();

                let app_state = self.clone();
                let room = room.to_owned();
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
    }

    fn check_for_timeout(
        &self,
        room: String,
        timer_len: u8,
        original_prompt: String,
    ) -> BoxFuture<'_, ()> {
        async move {
            tokio::time::sleep(Duration::from_secs(timer_len.into())).await;

            let mut lock = self.inner.lock().unwrap();
            let Some(Room { clients, state }) = lock.rooms.get_mut(&room) else {
                return;
            };

            let GameState::InGame(game) = state else {
                return;
            };

            if original_prompt == game.prompt {
                game.player_timed_out();

                if game.alive_players().len() == 1 {
                    clients.broadcast(ServerMessage::GameEnded {
                        winner: game.alive_players().first().unwrap().uuid,
                    });

                    *state = game.end();

                    clients.retain(|_uuid, client| !client.disconnected);
                } else {
                    clients.broadcast(ServerMessage::NewPrompt {
                        life_change: -1,
                        prompt: game.prompt.clone(),
                        turn: game.current_turn,
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

        for client in self.values().filter(|client| !client.disconnected) {
            client.tx.send(Message::Text(serialized.clone())).ok();
        }
    }
}
