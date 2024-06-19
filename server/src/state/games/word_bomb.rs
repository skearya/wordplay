use crate::{
    global::GLOBAL,
    messages::{self, ServerMessage},
    state::{
        error::{GameError, Result, WordBombError},
        lobby::end_game,
        AppState, Room, SenderInfo,
    },
    utils::{ClientUtils, Sorted},
};
use rand::{thread_rng, Rng};
use serde::Serialize;
use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::AbortHandle;
use uuid::Uuid;

#[derive(Debug)]
pub struct WordBomb {
    pub started_at: Instant,
    pub timer: Timer,
    pub prompt: String,
    pub prompt_uses: u8,
    pub missed_prompts: Vec<String>,
    pub players: Vec<Player>,
    pub turn: Uuid,
}

#[derive(Debug)]
pub struct Timer {
    pub task: Arc<AbortHandle>,
    pub start: Instant,
    pub length: f32,
}

#[derive(Serialize, Debug, Clone)]
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
#[serde(tag = "type")]
pub enum GuessInfo {
    PromptNotIn,
    NotEnglish,
    AlreadyUsed,
    Valid { extra_life: bool },
}

#[derive(Serialize, Debug, Clone)]
pub struct PostGameInfo {
    winner: Uuid,
    mins_elapsed: f32,
    words_used: usize,
    letters_typed: usize,
    fastest_guesses: Vec<(Uuid, f32, String)>,
    longest_words: Vec<(Uuid, String)>,
    avg_wpms: Vec<(Uuid, f32)>,
    avg_word_lengths: Vec<(Uuid, f32)>,
}

impl WordBomb {
    pub fn check_guess(&mut self, guess: &str) -> Result<GuessInfo, GameError> {
        let guess_info = if !guess.contains(&self.prompt) {
            GuessInfo::PromptNotIn
        } else if !GLOBAL.get().unwrap().is_valid(guess) {
            GuessInfo::NotEnglish
        } else if self
            .players
            .iter()
            .any(|player| player.used_words.iter().any(|(_, word)| word == guess))
        {
            GuessInfo::AlreadyUsed
        } else {
            let current_player = self
                .players
                .iter_mut()
                .find(|player| player.uuid == self.turn)
                .ok_or(WordBombError::PlayerNotFound)?;

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

            self.timer.length = (self.timer.length
                - Instant::now()
                    .duration_since(self.timer.start)
                    .as_secs_f32())
            .max(6.0);

            self.new_prompt();
            self.update_turn()?;

            GuessInfo::Valid { extra_life }
        };

        Ok(guess_info)
    }

    pub fn player_timed_out(&mut self) -> Result<()> {
        self.timer.length = thread_rng().gen_range(10.0..=30.0);

        self.missed_prompts.push(self.prompt.clone());

        let player = self
            .players
            .iter_mut()
            .find(|player| player.uuid == self.turn)
            .ok_or(WordBombError::PlayerNotFound)?;

        player.lives -= 1;

        self.prompt_uses += 1;
        if self.prompt_uses > 1 {
            self.new_prompt();
        }

        self.update_turn()?;

        Ok(())
    }

    pub fn alive_players(&self) -> Vec<&Player> {
        self.players
            .iter()
            .filter(|player| player.lives > 0)
            .collect()
    }

    fn new_prompt(&mut self) {
        self.prompt_uses = 0;
        self.prompt = loop {
            let new_prompt = GLOBAL.get().unwrap().random_prompt();

            if new_prompt != self.prompt {
                break new_prompt.to_string();
            }
        };
    }

    fn update_turn(&mut self) -> Result<()> {
        if self.alive_players().len() <= 1 {
            return Err(WordBombError::NoPlayersAlive)?;
        }

        let index = self
            .players
            .iter()
            .position(|player| player.uuid == self.turn)
            .ok_or(WordBombError::PlayerNotFound)?;

        let next_alive = self
            .players
            .iter()
            .cycle()
            .skip(index + 1)
            .find(|player| player.lives > 0)
            .ok_or(WordBombError::PlayerNotFound)?;

        self.turn = next_alive.uuid;

        Ok(())
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
        if new_input.len() > 35 {
            return Err(WordBombError::InputTooLong)?;
        }

        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state, .. } = lock.room_mut(room)?;
        let game = state.try_word_bomb()?;

        let player = game
            .players
            .iter_mut()
            .find(|player| player.uuid == uuid)
            .ok_or(WordBombError::PlayerNotFound)?;

        player.input.clone_from(&new_input);

        clients.broadcast(ServerMessage::WordBombInput {
            uuid,
            input: new_input,
        });

        Ok(())
    }

    pub fn word_bomb_guess(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        guess: String,
    ) -> Result<()> {
        if guess.len() > 35 {
            return Err(WordBombError::GuessTooLong)?;
        }

        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state, .. } = lock.room_mut(room)?;
        let game = state.try_word_bomb()?;

        if game.turn != uuid {
            return Err(WordBombError::OutOfTurn)?;
        }

        match game.check_guess(&guess) {
            Ok(GuessInfo::Valid { extra_life }) => {
                clients.broadcast(ServerMessage::WordBombPrompt {
                    correct_guess: Some(guess),
                    life_change: extra_life.into(),
                    prompt: game.prompt.clone(),
                    turn: game.turn,
                });

                game.timer.task.abort();
                spawn_timeout_task(self.clone(), game, room.to_string());
            }
            Ok(reason) => {
                clients.broadcast(ServerMessage::WordBombInvalidGuess { uuid, reason });
            }
            Err(error) => return Err(error),
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
            game.player_timed_out()?;

            if game.alive_players().len() <= 1 {
                let game_info = messages::PostGameInfo::WordBomb(get_post_game_info(game));

                end_game(state, clients, owner, game_info);
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

fn get_post_game_info(game: &mut WordBomb) -> PostGameInfo {
    PostGameInfo {
        winner: game.alive_players().first().unwrap().uuid,
        mins_elapsed: Instant::now().duration_since(game.started_at).as_secs_f32() / 60.0,
        words_used: game
            .players
            .iter()
            .map(|player| player.used_words.len())
            .sum(),
        letters_typed: game
            .players
            .iter()
            .map(|player| {
                player
                    .used_words
                    .iter()
                    .map(|(_, word)| word.len())
                    .sum::<usize>()
            })
            .sum(),
        fastest_guesses: game
            .players
            .iter()
            .filter_map(|player| {
                player
                    .used_words
                    .iter()
                    .map(|(duration, word)| (duration.as_secs_f32(), word))
                    .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
                    .map(|(duration, word)| (player.uuid, duration, word.clone()))
            })
            .sorted_by_vec(|a, b| a.1.partial_cmp(&b.1).unwrap()),
        longest_words: game
            .players
            .iter()
            .filter_map(|player| {
                player
                    .used_words
                    .iter()
                    .max_by_key(|(_, word)| word.len())
                    .map(|(_, word)| (player.uuid, word.clone()))
            })
            .sorted_by_vec(|a, b| b.1.len().cmp(&a.1.len())),
        avg_wpms: game
            .players
            .iter()
            .filter(|player| !player.used_letters.is_empty())
            .map(|player| {
                (
                    player.uuid,
                    player
                        .used_words
                        .iter()
                        .map(|(duration, word)| {
                            (word.len() as f32 / 5.0) / (duration.as_secs_f32() / 60.0)
                        })
                        .sum::<f32>()
                        / player.used_words.len() as f32,
                )
            })
            .sorted_by_vec(|a, b| b.1.partial_cmp(&a.1).unwrap()),
        avg_word_lengths: game
            .players
            .iter()
            .filter(|player| !player.used_letters.is_empty())
            .map(|player| {
                (
                    player.uuid,
                    player
                        .used_words
                        .iter()
                        .map(|(_, word)| word.len() as f32)
                        .sum::<f32>()
                        / player.used_words.len() as f32,
                )
            })
            .sorted_by_vec(|a, b| b.1.partial_cmp(&a.1).unwrap()),
    }
}
