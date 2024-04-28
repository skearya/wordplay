use super::{
    games::{
        anagrams::{self, Anagrams},
        word_bomb::{self, WordBomb},
    },
    room::{Client, ClientUtils, RoomSettings, State},
    AppState, Room,
};
use crate::{
    global::GLOBAL,
    messages::{CountdownState, Games, ServerMessage, WordBombPlayerData},
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
    pub fn start_word_bomb(
        &self,
        timeout_task_handle: impl FnOnce(String, u8) -> Arc<AbortHandle>,
    ) -> State {
        let timer_len = thread_rng().gen_range(10..=30);
        let prompt = GLOBAL.get().unwrap().random_prompt();
        let mut players: Vec<word_bomb::Player> = self
            .ready
            .iter()
            .map(|uuid| word_bomb::Player::new(*uuid))
            .collect();
        players.shuffle(&mut thread_rng());

        State::WordBomb(WordBomb {
            timeout_task: timeout_task_handle(prompt.clone(), timer_len),
            timer_len,
            starting_time: Instant::now(),
            prompt,
            prompt_uses: 0,
            used_words: HashSet::new(),
            current_turn: players[0].uuid,
            players,
        })
    }

    pub fn start_anagrams(&self) -> State {
        todo!()
    }
}

impl AppState {
    pub fn client_ready(&self, room: &str, uuid: Uuid) -> Result<()> {
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

    pub fn client_start_early(&self, room: &str, uuid: Uuid) -> Result<()> {
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

    pub fn client_unready(&self, room: &str, uuid: Uuid) -> Result<()> {
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
        room: &str,
        uuid: Uuid,
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
    clients: &HashMap<Uuid, Client>,
    settings: &RoomSettings,
) -> Result<()> {
    match settings.game {
        Games::WordBomb => {
            *state = state.try_lobby()?.start_word_bomb(|prompt, timer_len| {
                Arc::new(
                    tokio::spawn(async move {
                        app_state
                            .check_for_timeout(room, timer_len, prompt)
                            .await
                            .ok();
                    })
                    .abort_handle(),
                )
            });

            let game = state.try_word_bomb()?;

            let players: Vec<WordBombPlayerData> = game
                .players
                .iter()
                .map(|player| WordBombPlayerData {
                    uuid: player.uuid,
                    username: clients[&player.uuid].username.clone(),
                    input: player.input.clone(),
                    lives: player.lives,
                    disconnected: false,
                })
                .collect();

            clients.send_each(|uuid, _client| ServerMessage::WordBombStarted {
                rejoin_token: game
                    .players
                    .iter()
                    .find(|player| *uuid == player.uuid)
                    .map(|player| player.rejoin_token),
                prompt: game.prompt.clone(),
                turn: game.current_turn,
                players: players.clone(),
            });
        }
        Games::Anagrams => {
            todo!()
        }
    }

    Ok(())
}
