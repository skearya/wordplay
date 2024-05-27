use super::{
    games::{
        anagrams::{self, Anagrams},
        word_bomb::{self, WordBomb},
    },
    room::{check_for_new_room_owner, Client, ClientUtils, RoomSettings, State},
    AppState, Room, SenderInfo,
};
use crate::{
    global::GLOBAL,
    messages::{CountdownState, Games, RoomStateInfo, ServerMessage},
};

use anyhow::{Context, Result};
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::AbortHandle;
use uuid::Uuid;

pub struct Lobby {
    pub ready: HashSet<Uuid>,
    pub countdown: Option<Countdown>,
}

pub struct Countdown {
    pub time_left: u8,
    pub timer_handle: Arc<AbortHandle>,
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            ready: HashSet::new(),
            countdown: None,
        }
    }

    pub fn start_word_bomb(
        &self,
        timeout_task_handle: impl FnOnce(String, f32) -> Arc<AbortHandle>,
    ) -> State {
        let timer_len = thread_rng().gen_range(10.0..=30.0);
        let prompt = GLOBAL.get().unwrap().random_prompt().to_string();
        let mut players: Vec<word_bomb::Player> = self
            .ready
            .iter()
            .map(|uuid| word_bomb::Player::new(*uuid))
            .collect();
        players.shuffle(&mut thread_rng());

        State::WordBomb(WordBomb {
            started_at: Instant::now(),
            timer: word_bomb::Timer {
                task: timeout_task_handle(prompt.clone(), timer_len),
                start: Instant::now(),
                length: timer_len,
            },
            prompt,
            prompt_uses: 0,
            missed_prompts: Vec::new(),
            turn: players[0].uuid,
            players,
        })
    }

    pub fn start_anagrams(&self, app_state: AppState, room: String) -> State {
        let timer = Arc::new(
            tokio::spawn(async move {
                app_state.anagrams_timer(room).await.ok();
            })
            .abort_handle(),
        );

        State::Anagrams(Anagrams {
            timer,
            prompt: GLOBAL.get().unwrap().random_anagram(),
            players: self
                .ready
                .iter()
                .map(|uuid| anagrams::Player::new(*uuid))
                .collect(),
        })
    }
}

impl AppState {
    pub fn client_ready(&self, SenderInfo { uuid, room }: SenderInfo) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state, .. } = lock.room_mut(room)?;
        let lobby = state.try_lobby()?;
        if !lobby.ready.insert(uuid) {
            return Ok(());
        }

        let countdown_update = check_for_countdown_update(self.clone(), room.to_string(), lobby);

        clients.broadcast(ServerMessage::ReadyPlayers {
            ready: lobby.ready.iter().copied().collect(),
            countdown_update,
        });

        Ok(())
    }

    pub fn client_start_early(&self, SenderInfo { uuid, room }: SenderInfo) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room {
            clients,
            state,
            settings,
            owner,
        } = lock.room_mut(room)?;
        let lobby = state.try_lobby()?;

        if uuid == *owner && lobby.ready.len() >= 2 {
            if let Some(countdown) = &lobby.countdown {
                countdown.timer_handle.abort();
            }

            start_game(self.clone(), room.to_owned(), state, clients, settings)?;
        }

        Ok(())
    }

    pub fn client_unready(&self, SenderInfo { uuid, room }: SenderInfo) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state, .. } = lock.room_mut(room)?;
        let lobby = state.try_lobby()?;
        if !lobby.ready.remove(&uuid) {
            return Ok(());
        }

        let countdown_update = check_for_countdown_update(self.clone(), room.to_string(), lobby);

        clients.broadcast(ServerMessage::ReadyPlayers {
            ready: lobby.ready.iter().copied().collect(),
            countdown_update,
        });

        Ok(())
    }

    pub async fn start_when_ready(&self, room: String) -> Result<()> {
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let mut lock = self.inner.lock().unwrap();
            let Room {
                clients,
                state,
                settings,
                ..
            } = lock.room_mut(&room)?;
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

                start_game(self.clone(), room.clone(), state, clients, settings)?;
            } else {
                clients.broadcast(ServerMessage::StartingCountdown {
                    time_left: countdown.time_left,
                });
            }
        }

        Ok(())
    }

    pub fn client_room_settings(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        settings_update: RoomSettings,
    ) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room {
            clients,
            state,
            owner,
            settings,
        } = lock.room_mut(room)?;

        if state.try_lobby().is_ok() && uuid == *owner {
            *settings = settings_update.clone();
            clients.broadcast(ServerMessage::RoomSettings(settings_update));
        }

        Ok(())
    }
}

pub fn check_for_countdown_update(
    app_state: AppState,
    room: String,
    lobby: &mut Lobby,
) -> Option<CountdownState> {
    match &lobby.countdown {
        Some(countdown) if lobby.ready.len() < 2 => {
            countdown.timer_handle.abort();
            lobby.countdown = None;

            Some(CountdownState::Stopped)
        }
        None if lobby.ready.len() >= 2 => {
            lobby.countdown = Some(Countdown {
                timer_handle: Arc::new(
                    tokio::spawn(async move {
                        app_state.start_when_ready(room).await.ok();
                    })
                    .abort_handle(),
                ),
                time_left: 10,
            });

            Some(CountdownState::InProgress { time_left: 10 })
        }
        _ => None,
    }
}

fn start_game(
    app_state: AppState,
    room: String,
    state: &mut State,
    clients: &mut HashMap<Uuid, Client>,
    settings: &RoomSettings,
) -> Result<()> {
    let lobby = state.try_lobby()?;

    for uuid in &lobby.ready {
        clients.get_mut(uuid).unwrap().rejoin_token = Some(Uuid::new_v4());
    }

    let game = match settings.game {
        Games::WordBomb => {
            *state = lobby.start_word_bomb(|prompt, timer_len| {
                Arc::new(
                    tokio::spawn(async move {
                        app_state
                            .word_bomb_timer(room, timer_len, prompt)
                            .await
                            .ok();
                    })
                    .abort_handle(),
                )
            });

            let game = state.try_word_bomb()?;

            RoomStateInfo::WordBomb {
                turn: game.turn,
                players: game.players.clone(),
                prompt: game.prompt.clone(),
                used_letters: None,
            }
        }
        Games::Anagrams => {
            *state = lobby.start_anagrams(app_state, room);

            let game = state.try_anagrams()?;

            RoomStateInfo::Anagrams {
                players: game.players.clone(),
                prompt: game.prompt.clone(),
            }
        }
    };

    clients.send_each(|_uuid, client| ServerMessage::GameStarted {
        rejoin_token: client.rejoin_token,
        game: game.clone(),
    });

    Ok(())
}

pub fn end_game(
    state: &mut State,
    clients: &mut HashMap<Uuid, Client>,
    owner: &mut Uuid,
    winner: Uuid,
) {
    clients.retain(|_uuid, client| client.socket.is_some());

    for client in clients.values_mut() {
        client.rejoin_token = None;
    }

    let new_room_owner = check_for_new_room_owner(clients, owner);

    clients.broadcast(ServerMessage::GameEnded {
        winner,
        new_room_owner,
    });

    *state = State::Lobby(Lobby::new());
}
