use crate::{
    global::GLOBAL,
    messages::ServerMessage,
    state::{lobby::end_game, room::ClientUtils, AppState, Room, SenderInfo},
};

use anyhow::{Context, Result};
use rand::{thread_rng, Rng};
use serde::Serialize;
use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::AbortHandle;
use uuid::Uuid;

pub struct WordBomb {
    pub timeout_task: Arc<AbortHandle>,
    pub timer_len: u8,
    pub starting_time: Instant,
    pub prompt: String,
    pub prompt_uses: u8,
    pub used_words: HashSet<String>,
    pub players: Vec<Player>,
    pub current_turn: Uuid,
}

pub struct Player {
    pub uuid: Uuid,
    pub input: String,
    pub lives: u8,
    pub used_letters: HashSet<char>,
}

#[derive(Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GuessInfo {
    PromptNotIn,
    NotEnglish,
    AlreadyUsed,
    Valid { extra_life: bool },
}

impl WordBomb {
    pub fn check_guess(&mut self, guess: &str) -> GuessInfo {
        if !guess.contains(&self.prompt) {
            return GuessInfo::PromptNotIn;
        }
        if !GLOBAL.get().unwrap().is_valid(guess) {
            return GuessInfo::NotEnglish;
        }
        if !self.used_words.insert(guess.to_owned()) {
            return GuessInfo::AlreadyUsed;
        }

        let current_player = self
            .players
            .iter_mut()
            .find(|player| player.uuid == self.current_turn)
            .unwrap();

        current_player
            .used_letters
            .extend(guess.chars().filter(|char| char.is_alphabetic()));

        let extra_life = ('a'..='z')
            .filter(|char| !['x', 'z'].contains(char))
            .all(|char| current_player.used_letters.contains(&char));

        if extra_life {
            current_player.lives += 1;
            current_player.used_letters.clear();
        }

        GuessInfo::Valid { extra_life }
    }

    pub fn new_prompt(&mut self) {
        self.prompt = loop {
            let new_prompt = GLOBAL.get().unwrap().random_prompt();

            if new_prompt != self.prompt {
                break new_prompt.to_string();
            }
        };

        self.prompt_uses = 0;
    }

    pub fn update_turn(&mut self) {
        let index = self
            .players
            .iter()
            .position(|player| player.uuid == self.current_turn)
            .unwrap();

        let next_alive = self
            .players
            .iter()
            .cycle()
            .skip(index + 1)
            .find(|player| player.lives > 0)
            .unwrap();

        self.current_turn = next_alive.uuid;
    }

    pub fn update_timer_len(&mut self) {
        self.timer_len = self
            .timer_len
            .saturating_sub(Instant::now().duration_since(self.starting_time).as_secs() as u8)
            .max(6);
    }

    pub fn player_timed_out(&mut self) {
        self.timer_len = thread_rng().gen_range(10..=30);

        if let Some(player) = self
            .players
            .iter_mut()
            .find(|player| player.uuid == self.current_turn)
        {
            player.lives -= 1;
        }

        self.update_turn();
        self.prompt_uses += 1;

        if self.prompt_uses > 1 {
            self.new_prompt();
        }
    }

    pub fn alive_players(&self) -> Vec<&Player> {
        self.players
            .iter()
            .filter(|player| player.lives > 0)
            .collect()
    }
}

impl Player {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            input: String::new(),
            lives: 2,
            used_letters: HashSet::new(),
        }
    }
}

impl AppState {
    pub fn word_bomb_input(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        new_input: String,
    ) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state, .. } = lock.room_mut(room)?;
        let game = state.try_word_bomb()?;
        let player = game
            .players
            .iter_mut()
            .find(|player| player.uuid == uuid)
            .context("Player not found")?;

        player.input = new_input.clone();

        clients.broadcast(ServerMessage::WordBombInput {
            uuid,
            input: new_input,
        });

        Ok(())
    }

    pub fn word_bomb_guess(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        guess: &str,
    ) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state, .. } = lock.room_mut(room)?;
        let game = state.try_word_bomb()?;
        if game.current_turn != uuid {
            return Ok(());
        }

        match game.check_guess(guess) {
            GuessInfo::Valid { extra_life } => {
                game.new_prompt();
                game.update_turn();
                game.update_timer_len();

                clients.broadcast(ServerMessage::WordBombPrompt {
                    correct_guess: Some(guess.to_string()),
                    life_change: extra_life.into(),
                    prompt: game.prompt.clone(),
                    turn: game.current_turn,
                });

                game.timeout_task.abort();
                spawn_timeout_task(self.clone(), game, room.to_string());
            }
            guess_info => {
                clients.broadcast(ServerMessage::WordBombInvalidGuess {
                    uuid,
                    reason: guess_info,
                });
            }
        };

        Ok(())
    }

    pub async fn word_bomb_timer(
        &self,
        room: String,
        timer_len: u8,
        original_prompt: String,
    ) -> Result<()> {
        tokio::time::sleep(Duration::from_secs(timer_len.into())).await;

        let mut lock = self.inner.lock().unwrap();
        let Room {
            clients,
            state,
            owner,
            ..
        } = lock.room_mut(&room)?;
        let game = state.try_word_bomb()?;

        if original_prompt == game.prompt {
            game.player_timed_out();

            if game.alive_players().len() == 1 {
                let winner = game.alive_players().first().unwrap().uuid;
                end_game(state, clients, owner, winner);
            } else {
                clients.broadcast(ServerMessage::WordBombPrompt {
                    correct_guess: None,
                    life_change: -1,
                    prompt: game.prompt.clone(),
                    turn: game.current_turn,
                });

                spawn_timeout_task(self.clone(), game, room);
            }
        }

        Ok(())
    }
}

fn spawn_timeout_task(app_state: AppState, game: &mut WordBomb, room: String) {
    let timer_len = game.timer_len;
    let current_prompt = game.prompt.clone();

    game.timeout_task = Arc::new(
        tokio::spawn(async move {
            app_state
                .word_bomb_timer(room, timer_len, current_prompt)
                .await
                .ok();
        })
        .abort_handle(),
    );
}
