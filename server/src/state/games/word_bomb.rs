use crate::{
    messages::{InvalidWordReason, ServerMessage},
    state::games::GuessInfo,
    state::{
        room::{check_for_new_room_owner, Messaging},
        AppState, Room,
    },
};

use anyhow::{Context, Result};
use futures::{future::BoxFuture, FutureExt};
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

impl AppState {
    pub fn client_input_update(&self, room: &str, uuid: Uuid, new_input: String) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state, .. } = lock.room_mut(room)?;
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
        let Room { clients, state, .. } = lock.room_mut(room)?;
        let game = state.try_in_game()?;

        if game.current_turn != uuid {
            return Ok(());
        }

        match game.parse_prompt(guess) {
            GuessInfo::Valid { extra_life } => {
                game.new_prompt();
                game.update_turn();
                game.update_timer_len();

                clients.broadcast(ServerMessage::NewPrompt {
                    life_change: extra_life.into(),
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
                    GuessInfo::Valid { .. } => unreachable!(),
                };

                clients.broadcast(ServerMessage::InvalidWord { uuid, reason });
            }
        };

        Ok(())
    }

    pub fn check_for_timeout(
        &self,
        room: String,
        timer_len: u8,
        original_prompt: String,
    ) -> BoxFuture<'_, Result<()>> {
        async move {
            tokio::time::sleep(Duration::from_secs(timer_len.into())).await;

            let mut lock = self.inner.lock().unwrap();
            let Room {
                clients,
                state,
                owner,
                ..
            } = lock.room_mut(&room)?;
            let game = state.try_in_game()?;

            if original_prompt == game.prompt {
                game.player_timed_out();

                if game.alive_players().len() == 1 {
                    clients.retain(|_uuid, client| client.socket.is_some());

                    let new_room_owner = check_for_new_room_owner(clients, owner);

                    clients.broadcast(ServerMessage::GameEnded {
                        winner: game.alive_players().first().unwrap().uuid,
                        new_room_owner,
                    });

                    *state = game.end();
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
}
