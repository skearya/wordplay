use super::{room::Messaging, AppState, Client, Room};
use crate::{
    messages::{CountdownState, PlayerData, ServerMessage},
    state::games::{Countdown, GameState, Lobby},
};

use anyhow::{Context, Result};
use std::{collections::HashMap, sync::Arc, time::Duration};
use uuid::Uuid;

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
            owner,
            ..
        } = lock.room_mut(room)?;
        let lobby = state.try_lobby()?;

        if uuid == *owner && lobby.ready.len() >= 2 {
            if let Some(countdown) = &lobby.countdown {
                countdown.timer_handle.abort();
            }

            start_game(self.clone(), room.to_owned(), state, clients)?;
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
            let Room { clients, state, .. } = lock.room_mut(&room)?;

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

                start_game(self.clone(), room.clone(), state, clients)?;
            } else {
                clients.broadcast(ServerMessage::StartingCountdown {
                    time_left: countdown.time_left,
                });
            }
        }

        Ok(())
    }

    pub fn client_game_settings(
        &self,
        room: &str,
        uuid: Uuid,
        visibility_update: bool,
    ) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room {
            clients,
            state,
            owner,
            public,
        } = lock.room_mut(room)?;

        if state.try_lobby().is_ok() && uuid == *owner {
            *public = visibility_update;
            clients.broadcast(ServerMessage::GameSettings { public: *public });
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
                    tokio::task::spawn(async move {
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
    state: &mut GameState,
    clients: &HashMap<Uuid, Client>,
) -> Result<()> {
    *state = state.try_lobby()?.start_game(move |prompt, timer_len| {
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
            input: player.input.clone(),
            lives: player.lives,
            disconnected: false,
        })
        .collect();

    clients.send_each(|uuid, _client| ServerMessage::GameStarted {
        rejoin_token: game
            .players
            .iter()
            .find(|player| *uuid == player.uuid)
            .map(|player| player.rejoin_token),
        prompt: game.prompt.clone(),
        turn: game.current_turn,
        players: players.clone(),
    });

    Ok(())
}
