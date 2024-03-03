use crate::{
    game::GameState,
    global::GLOBAL,
    messages::{PlayerData, PlayerInfo, RoomState, ServerMessage},
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

#[derive(Default, Clone)]
pub struct Room {
    pub clients: Vec<Client>,
    pub state: GameState,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub uuid: Uuid,
    pub tx: UnboundedSender<ServerMessage>,
    pub username: String,
    // TODO: disconnected: bool
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

    pub fn add_client(&self, room: String, client: Client) {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.entry(room).or_default();

        clients.push(client.clone());

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
                        // TODO: this is bad cause if a user leaves they arent in the clients vec anymore sooo
                        // TODO: THIS IS REALLY BAD BECAUSE WE POISON THE GLOBAL STATE!!!!
                        username: clients
                            .iter()
                            .find(|client| client.uuid == player.uuid)
                            .unwrap()
                            .username
                            .clone(),
                        input: player.input.clone(),
                        lives: player.lives,
                    })
                    .collect(),
            },
        };

        client
            .tx
            .send(ServerMessage::RoomInfo {
                uuid: client.uuid,
                state: room_state,
            })
            .ok();

        clients.broadcast(ServerMessage::ServerMessage {
            content: format!("{} has joined", client.username),
        });
    }

    pub fn remove_client(&self, room: &str, uuid: Uuid) {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, .. } = lock.rooms.get_mut(room).unwrap();

        if let Some(index) = clients.iter().position(|client| client.uuid == uuid) {
            let removed = clients.remove(index);

            if clients.is_empty() {
                lock.rooms.remove(room);
            } else {
                clients.broadcast(ServerMessage::ServerMessage {
                    content: format!("{} has left", removed.username),
                });
            }
        }

        // TODO: Handle player leaving & rejoining?

        // match state {
        //     GameState::Lobby(lobby) => {
        //         lobby.ready.remove(&(uuid, username));
        //     }
        //     GameState::InGame(game) => {
        //         game.players.retain(|player| player.uuid != uuid);
        //     }
        // };

        // clients.broadcast(ServerMessage::Players {
        //     players: clients
        //         .iter()
        //         .map(|client| PlayerInfo {
        //             uuid: client.uuid,
        //             username: client.username.clone(),
        //         })
        //         .collect::<Vec<PlayerInfo>>(),
        // });
    }

    pub fn client_ready(&self, room: &str, client: (Uuid, String)) {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(room).unwrap();

        let GameState::Lobby(lobby) = state else {
            return;
        };

        lobby.ready.insert(client);

        if lobby.ready.len() >= 2 && clients.len() == lobby.ready.len() {
            *state = lobby.start_game();

            let GameState::InGame(game) = state else {
                unreachable!();
            };

            clients.broadcast(ServerMessage::GameStarted {
                prompt: game.prompt.clone(),
                turn: game.current_turn,
                players: game
                    .players
                    .iter()
                    .map(|player| PlayerData {
                        uuid: player.uuid,
                        username: clients
                            .iter()
                            .find(|client| client.uuid == player.uuid)
                            .unwrap()
                            .clone()
                            .username,
                        input: player.input.clone(),
                        lives: player.lives,
                    })
                    .collect(),
            });

            let app_state = self.clone();
            let room = room.to_owned();
            let timer_len = game.timer_len;
            let current_prompt = game.prompt.clone();

            tokio::task::spawn(async move {
                app_state
                    .check_for_timeout(room, timer_len, current_prompt)
                    .await;
            });
        } else {
            clients.broadcast(ServerMessage::ReadyPlayers {
                players: lobby
                    .ready
                    .iter()
                    .map(|client| PlayerInfo {
                        uuid: client.0,
                        username: client.1.clone(),
                    })
                    .collect(),
            });
        }
    }

    pub fn client_input_update(&self, room: &str, uuid: Uuid, new_input: String) {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.get_mut(room).unwrap();

        let GameState::InGame(game) = state else {
            return;
        };

        let player = game
            .players
            .iter_mut()
            .find(|player| player.uuid == uuid)
            .unwrap();

        player.input = new_input;

        clients.broadcast(ServerMessage::InputUpdate {
            uuid,
            input: player.input.clone(),
        })
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
            game.progress();

            clients.broadcast(ServerMessage::NewPrompt {
                timed_out: false,
                prompt: game.prompt.clone(),
                turn: game.current_turn,
            });

            if let Some(abort_handle) = &game.timeout_task {
                abort_handle.abort();
            }

            let app_state = self.clone();
            let room = room.to_owned();
            let timer_len = game.timer_len;
            let current_prompt = game.prompt.clone();

            tokio::task::spawn(async move {
                app_state
                    .check_for_timeout(room, timer_len, current_prompt)
                    .await;
            });
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
            if let Some(Room { clients, state }) = lock.rooms.get_mut(&room) {
                let GameState::InGame(game) = state else {
                    return;
                };

                if original_prompt == game.prompt {
                    game.player_timed_out();

                    let alive_players = game.alive_players();

                    if alive_players.len() == 1 {
                        clients.broadcast(ServerMessage::GameEnded {
                            winner: alive_players.first().unwrap().uuid,
                        });

                        *state = game.end();
                    } else {
                        clients.broadcast(ServerMessage::NewPrompt {
                            timed_out: true,
                            prompt: game.prompt.clone(),
                            turn: game.current_turn,
                        });

                        let app_state = self.clone();
                        let timer_len = game.timer_len;
                        let current_prompt = game.prompt.clone();

                        game.timeout_task = Some(Arc::new(
                            tokio::task::spawn(async move {
                                app_state
                                    .check_for_timeout(room, timer_len, current_prompt)
                                    .await;
                            })
                            .abort_handle(),
                        ));
                    }
                }
            }
        }
        .boxed()
    }
}

pub trait Broadcast {
    fn broadcast(&self, message: ServerMessage);
}

impl Broadcast for Vec<Client> {
    fn broadcast(&self, message: ServerMessage) {
        for client in self {
            client.tx.send(message.clone()).ok();
        }
    }
}
