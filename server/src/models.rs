use crate::{
    game::{Countdown, GameState, GuessInfo},
    messages::{
        ClientInfo, ConnectionUpdate, CountdownState, InvalidWordReason, PlayerData, RoomState,
        ServerMessage,
    },
    Params,
};

use anyhow::{Context, Result};
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

    pub fn add_client(
        &self,
        room: &str,
        params: Params,
        socket_uuid: Uuid,
        tx: UnboundedSender<Message>,
    ) -> Uuid {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state } = lock.rooms.entry(room.to_string()).or_default();

        let uuid = if let Some(uuid) = params.rejoin_token.and_then(|rejoin_token| {
            if let GameState::InGame(game) = state {
                game.players
                    .iter()
                    .find(|player| rejoin_token == player.rejoin_token)
                    .map(|player| player.uuid)
            } else {
                None
            }
        }) {
            clients.broadcast(ServerMessage::ConnectionUpdate {
                uuid,
                state: ConnectionUpdate::Reconnected {
                    username: params.username.clone(),
                },
            });

            uuid
        } else {
            let uuid = Uuid::new_v4();

            clients.broadcast(ServerMessage::ConnectionUpdate {
                uuid,
                state: ConnectionUpdate::Connected {
                    username: params.username.clone(),
                },
            });

            uuid
        };

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
                ready: lobby.ready.iter().copied().collect(),
                starting_countdown: lobby
                    .countdown
                    .as_ref()
                    .map(|countdown| countdown.time_left),
            },
            GameState::InGame(game) => RoomState::InGame {
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
                ServerMessage::RoomInfo {
                    clients: clients
                        .iter()
                        .filter(|client| client.1.socket.is_some())
                        .map(|(uuid, client)| ClientInfo {
                            uuid: *uuid,
                            username: client.username.clone(),
                        })
                        .collect(),
                    uuid,
                    state: room_state,
                }
                .into(),
            )
            .ok();

        uuid
    }

    pub fn remove_client(&self, room: &str, uuid: Uuid, socket_uuid: Uuid) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let (clients, state) = lock.room_mut(room)?;

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
        } else {
            if let GameState::Lobby(lobby) = state {
                clients.remove(&uuid);

                if lobby.ready.remove(&uuid) {
                    clients.broadcast(ServerMessage::ReadyPlayers {
                        ready: lobby.ready.iter().copied().collect(),
                    });

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
            }

            clients.broadcast(ServerMessage::ConnectionUpdate {
                uuid,
                state: ConnectionUpdate::Disconnected,
            });
        }

        Ok(())
    }

    pub fn client_chat_message(&self, room: &str, uuid: Uuid, content: String) -> Result<()> {
        let lock = self.inner.lock().unwrap();
        let (clients, _) = lock.room(room)?;

        clients.broadcast(ServerMessage::ChatMessage {
            author: uuid,
            content,
        });

        Ok(())
    }

    pub fn client_ready(&self, room: &str, uuid: Uuid) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let (clients, state) = lock.room_mut(room)?;

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
                        app_state.start_when_ready(room).await.ok();
                    })
                    .abort_handle(),
                ),
                time_left: 10,
            });
        }

        clients.broadcast(ServerMessage::ReadyPlayers {
            ready: lobby.ready.iter().copied().collect(),
        });

        Ok(())
    }

    pub async fn start_when_ready(&self, room: String) -> Result<()> {
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let mut lock = self.inner.lock().unwrap();
            let (clients, state) = lock.room_mut(&room)?;

            let lobby = state.try_lobby()?;
            let countdown = lobby
                .countdown
                .as_mut()
                .context("Somehow not counting down")?;

            countdown.time_left -= 1;

            if countdown.time_left == 0 {
                if lobby.ready.len() < 2 {
                    return Ok(());
                }

                let app_state = self.clone();
                let room = room.clone();

                *state = lobby.start_game(move |prompt, timer_len| {
                    Arc::new(
                        tokio::task::spawn(async move {
                            app_state
                                .check_for_timeout(room, timer_len, prompt)
                                .await
                                .ok();
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
                        disconnected: false,
                        input: player.input.clone(),
                        lives: player.lives,
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

        Ok(())
    }

    pub fn client_input_update(&self, room: &str, uuid: Uuid, new_input: String) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let (clients, state) = lock.room_mut(room)?;

        let game = state.try_in_game()?;
        let player = game
            .players
            .iter_mut()
            .find(|player| player.uuid == uuid)
            .context("Player not found")?;

        player.input = new_input.clone();

        clients.broadcast(ServerMessage::InputUpdate {
            uuid,
            input: new_input,
        });

        Ok(())
    }

    pub fn client_guess(&self, room: &str, uuid: Uuid, guess: &str) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let (clients, state) = lock.room_mut(room)?;

        let game = state.try_in_game()?;

        if game.current_turn != uuid {
            return Ok(());
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
                            .await
                            .ok();
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

        Ok(())
    }

    fn check_for_timeout(
        &self,
        room: String,
        timer_len: u8,
        original_prompt: String,
    ) -> BoxFuture<'_, Result<()>> {
        async move {
            tokio::time::sleep(Duration::from_secs(timer_len.into())).await;

            let mut lock = self.inner.lock().unwrap();
            let (clients, state) = lock.room_mut(&room)?;

            let game = state.try_in_game()?;

            if original_prompt == game.prompt {
                game.player_timed_out();

                if game.alive_players().len() == 1 {
                    clients.broadcast(ServerMessage::GameEnded {
                        winner: game.alive_players().first().unwrap().uuid,
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
                                .await
                                .ok();
                        })
                        .abort_handle(),
                    );
                }
            }

            Ok(())
        }
        .boxed()
    }

    pub fn errored(&self, room: &str, uuid: Uuid) -> Result<()> {
        let lock = self.inner.lock().unwrap();
        let (clients, _) = lock.room(room)?;

        clients[&uuid]
            .tx
            .send(
                ServerMessage::ServerMessage {
                    content: "something went wrong...".into(),
                }
                .into(),
            )
            .ok();

        Ok(())
    }
}

impl AppStateInner {
    fn room(&self, room: &str) -> Result<(&HashMap<Uuid, Client>, &GameState)> {
        let room = self.rooms.get(room).context("Room not found")?;
        Ok((&room.clients, &room.state))
    }
    fn room_mut(&mut self, room: &str) -> Result<(&mut HashMap<Uuid, Client>, &mut GameState)> {
        let room = self.rooms.get_mut(room).context("Room not found")?;
        Ok((&mut room.clients, &mut room.state))
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
