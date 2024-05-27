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
    pub started_at: Instant,
    pub timer: Timer,
    pub prompt: String,
    pub prompt_uses: u8,
    pub missed_prompts: Vec<String>,
    pub players: Vec<Player>,
    pub turn: Uuid,
}

pub struct Timer {
    pub task: Arc<AbortHandle>,
    pub start: Instant,
    pub length: f32,
}

#[derive(Debug, Serialize, Clone)]
pub struct Player {
    pub uuid: Uuid,
    pub input: String,
    pub lives: u8,
    #[serde(skip_serializing)]
    pub used_words: Vec<(Duration, String)>,
    #[serde(skip_serializing)]
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
        if self
            .players
            .iter()
            .any(|player| player.used_words.iter().any(|(_, word)| word == guess))
        {
            return GuessInfo::AlreadyUsed;
        }

        let current_player = self
            .players
            .iter_mut()
            .find(|player| player.uuid == self.turn)
            .unwrap();

        current_player.used_words.push((
            Instant::now().duration_since(self.timer.start),
            guess.to_string(),
        ));

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

        self.new_prompt();
        self.update_turn();
        self.update_timer_len();

        GuessInfo::Valid { extra_life }
    }

    pub fn player_timed_out(&mut self) {
        self.timer.length = thread_rng().gen_range(10.0..=30.0);

        if let Some(player) = self
            .players
            .iter_mut()
            .find(|player| player.uuid == self.turn)
        {
            player.lives -= 1;
        }

        self.missed_prompts.push(self.prompt.clone());

        self.prompt_uses += 1;
        if self.prompt_uses > 1 {
            self.new_prompt();
        }

        self.update_turn();
    }

    pub fn alive_players(&self) -> Vec<&Player> {
        self.players
            .iter()
            .filter(|player| player.lives > 0)
            .collect()
    }

    fn new_prompt(&mut self) {
        self.prompt = loop {
            let new_prompt = GLOBAL.get().unwrap().random_prompt();

            if new_prompt != self.prompt {
                break new_prompt.to_string();
            }
        };

        self.prompt_uses = 0;
    }

    fn update_turn(&mut self) {
        let index = self
            .players
            .iter()
            .position(|player| player.uuid == self.turn)
            .unwrap();

        let next_alive = self
            .players
            .iter()
            .cycle()
            .skip(index + 1)
            .find(|player| player.lives > 0)
            .unwrap();

        self.turn = next_alive.uuid;
    }

    fn update_timer_len(&mut self) {
        self.timer.length = (self.timer.length
            - Instant::now()
                .duration_since(self.timer.start)
                .as_secs_f32())
        .max(6.0);
    }
}

impl Player {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            input: String::new(),
            lives: 2,
            used_letters: HashSet::new(),
            used_words: Vec::new(),
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
        if game.turn != uuid {
            return Ok(());
        }

        match game.check_guess(guess) {
            GuessInfo::Valid { extra_life } => {
                clients.broadcast(ServerMessage::WordBombPrompt {
                    correct_guess: Some(guess.to_string()),
                    life_change: extra_life.into(),
                    prompt: game.prompt.clone(),
                    turn: game.turn,
                });

                game.timer.task.abort();
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
        timer_len: f32,
        original_prompt: String,
    ) -> Result<()> {
        tokio::time::sleep(Duration::from_secs_f32(timer_len)).await;

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
                    turn: game.turn,
                });

                spawn_timeout_task(self.clone(), game, room);
            }
        }

        Ok(())
    }
}

fn spawn_timeout_task(app_state: AppState, game: &mut WordBomb, room: String) {
    let timer_len = game.timer.length;
    let current_prompt = game.prompt.clone();

    game.timer.task = Arc::new(
        tokio::spawn(async move {
            app_state
                .word_bomb_timer(room, timer_len, current_prompt)
                .await
                .ok();
        })
        .abort_handle(),
    );
}
