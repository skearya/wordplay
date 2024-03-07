use crate::{
    game::GameState,
    global::GLOBAL,
    messages::{PlayerData, PlayerInfo, RoomState, ServerMessage},
    Params,
};

use futures::{future::BoxFuture, FutureExt};
use std::{
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
    pub tx: UnboundedSender<ServerMessage>,
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

    pub fn broadcast(&self, room: &str, message: ServerMessage) {
        let lock = self.inner.lock().unwrap();
        let Room { clients, .. } = lock.rooms.get(room).unwrap();

        clients.broadcast(message);
    }

    pub fn add_client(
        &self,
        room: String,
        params: Params,
        tx: UnboundedSender<ServerMessage>,
    ) -> (Uuid, String) {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.entry(room).or_default();

        let prev_uuid = params
            .rejoin_token
            .and_then(|token| {
                if let GameState::InGame(game) = state {
                    game.players
                        .iter()
                        .find(|player| token == player.rejoin_token)
                } else {
                    None
                }
            })
            .map(|player| player.uuid);

        let (uuid, client) = match prev_uuid {
            Some(uuid) => {
                clients.entry(uuid).and_modify(|client| {
                    client.tx = tx;
                    client.disconnected = false;
                });

                clients.get_key_value(&uuid).unwrap()
            }
            None => {
                let uuid = Uuid::new_v4();

                clients.insert(
                    uuid,
                    Client {
                        tx,
                        disconnected: false,
                        username: params.username.clone(),
                    },
                );

                clients.get_key_value(&uuid).unwrap()
            }
        };

        let room_state = match state {
            GameState::Lobby(lobby) => RoomState::Lobby {
                ready_players: lobby
                    .ready
                    .iter()
                    .map(|(uuid, username)| PlayerInfo {
                        uuid: *uuid,
                        username: username.clone(),
                    })
                    .collect(),
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
                    })
                    .collect(),
                used_letters: prev_uuid.map(|uuid| {
                    game.players
                        .iter()
                        .find(|player| uuid == player.uuid)
                        .unwrap()
                        .used_letters
                        .clone()
                }),
            },
        };

        client
            .tx
            .send(ServerMessage::RoomInfo {
                uuid: *uuid,
                state: room_state,
            })
            .ok();

        clients.broadcast(ServerMessage::ServerMessage {
            content: format!("{} has joined", params.username),
        });

        (*uuid, client.username.clone())
    }

    pub fn remove_client(&self, room: &str, uuid: Uuid) {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(room).unwrap();

        let client = clients.get_mut(&uuid).unwrap();
        client.disconnected = true;

        let username = client.username.clone();

        if clients
            .values()
            .filter(|client| !client.disconnected)
            .count()
            == 0
        {
            lock.rooms.remove(room);
        } else {
            if let GameState::Lobby(lobby) = state {
                lobby.ready.remove(&(uuid, username.clone()));
                clients.remove(&uuid);
            }

            clients.broadcast(ServerMessage::ServerMessage {
                content: format!("{} has left", username),
            });
        }
    }

    pub fn client_ready(&self, room: &str, client: (Uuid, String)) {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(room).unwrap();

        let GameState::Lobby(lobby) = state else {
            return;
        };

        lobby.ready.insert(client);

        if lobby.ready.len() >= 2 {
            if let Some(start_handle) = &lobby.start_handle {
                start_handle.abort();
            }

            let app_state = self.clone();
            let room = room.to_owned();

            lobby.start_handle = Some(Arc::new(
                tokio::task::spawn(async move {
                    app_state.start_when_ready(room).await;
                })
                .abort_handle(),
            ));
        }

        clients.broadcast(ServerMessage::ReadyPlayers {
            players: lobby
                .ready
                .iter()
                .map(|(uuid, username)| PlayerInfo {
                    uuid: *uuid,
                    username: username.clone(),
                })
                .collect(),
            countdown: lobby.ready.len() >= 2,
        });
    }

    pub async fn start_when_ready(&self, room: String) {
        tokio::time::sleep(Duration::from_secs(10)).await;

        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(&room).unwrap();

        let GameState::Lobby(lobby) = state else {
            return;
        };

        if lobby.ready.len() >= 2 {
            let app_state = self.clone();
            let room = room.to_owned();

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
                })
                .collect();

            for (uuid, client) in clients {
                client
                    .tx
                    .send(ServerMessage::GameStarted {
                        rejoin_token: game
                            .players
                            .iter()
                            .find(|player| *uuid == player.uuid)
                            .unwrap()
                            .rejoin_token,
                        prompt: game.prompt.clone(),
                        turn: game.current_turn,
                        players: players.clone(),
                    })
                    .ok();
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

        if guess.contains(&game.prompt) && GLOBAL.get().unwrap().is_valid(guess) {
            let extra_life = game.check_for_extra_life(guess);
            game.new_prompt();
            game.update_turn();
            game.update_timer_len();

            clients.broadcast(ServerMessage::NewPrompt {
                life_change: if extra_life { 1 } else { 0 },
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
        } else {
            clients.broadcast(ServerMessage::InvalidWord { uuid });
        }
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
        for client in self.values().filter(|client| !client.disconnected) {
            client.tx.send(message.clone()).ok();
        }
    }
}
