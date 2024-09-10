use crate::{
    global::GLOBAL,
    state::{
        error::{GameError, Result, WordBombError},
        lobby::end_game,
        messages::{self, ServerMessage},
        Room, SenderInfo,
    },
    utils::{filter_string, ClientUtils, Sorted},
    AppState,
};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::AbortHandle;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WordBombSettings {
    pub min_wpm: usize,
}

#[derive(Debug)]
pub struct WordBomb {
    pub settings: WordBombSettings,
    pub started_at: Instant,
    pub timer: Timer,
    pub prompt: &'static str,
    pub prompt_uses: u8,
    pub missed_prompts: Vec<&'static str>,
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
    fastest_guesses: Vec<(Uuid, f32)>,
    longest_words: Vec<(Uuid, String)>,
    avg_wpms: Vec<(Uuid, f32)>,
    avg_word_lengths: Vec<(Uuid, f32)>,
}

impl WordBomb {
    pub fn check_guess(&mut self, guess: &str) -> Result<GuessInfo, GameError> {
        let guess_info = if !guess.contains(self.prompt) {
            GuessInfo::PromptNotIn
        } else if !GLOBAL.is_valid(guess) {
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
                .extend(guess.chars().filter(|c| c.is_alphabetic()));

            let extra_life = ('a'..='z')
                .filter(|c| !['x', 'z'].contains(c))
                .all(|c| current_player.used_letters.contains(&c));

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

        self.missed_prompts.push(self.prompt);

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

        for _ in 0..10 {
            let new_prompt = GLOBAL.prompts.random_prompt(self.settings.min_wpm);

            if new_prompt != self.prompt {
                self.prompt = new_prompt;
                break;
            }
        }
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

        let mut lock = self.room_mut(room)?;
        let Room { clients, state, .. } = lock.value_mut();
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
        mut guess: String,
    ) -> Result<()> {
        filter_string(&mut guess);

        if guess.len() > 35 {
            return Err(WordBombError::GuessTooLong)?;
        }

        let mut lock = self.room_mut(room)?;
        let Room { clients, state, .. } = lock.value_mut();
        let game = state.try_word_bomb()?;

        if game.turn != uuid {
            return Err(WordBombError::OutOfTurn)?;
        }

        match game.check_guess(&guess) {
            Ok(GuessInfo::Valid { extra_life }) => {
                clients.broadcast(ServerMessage::WordBombPrompt {
                    correct_guess: Some(guess),
                    life_change: extra_life.into(),
                    prompt: game.prompt.to_string(),
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
        original_prompt: &str,
    ) -> Result<()> {
        tokio::time::sleep(Duration::from_secs_f32(timer_len)).await;

        let mut lock = self.room_mut(&room)?;
        let Room {
            clients,
            state,
            owner,
            ..
        } = lock.value_mut();
        let game = state.try_word_bomb()?;

        if original_prompt == game.prompt {
            match game.player_timed_out() {
                Ok(()) => {
                    clients.broadcast(ServerMessage::WordBombPrompt {
                        correct_guess: None,
                        life_change: -1,
                        prompt: game.prompt.to_string(),
                        turn: game.turn,
                    });

                    spawn_timeout_task(self.clone(), game, room);
                }
                Err(GameError::WordBomb(WordBombError::NoPlayersAlive)) => {
                    let game_info = messages::PostGameInfo::WordBomb(get_post_game_info(game));
                    end_game(state, clients, owner, game_info);
                }
                Err(error) => Err(error)?,
            }
        }

        Ok(())
    }
}

fn spawn_timeout_task(app_state: AppState, game: &mut WordBomb, room: String) {
    let timer_len = game.timer.length;
    let current_prompt = game.prompt;

    game.timer.task = Arc::new(
        tokio::spawn(async move {
            app_state
                .word_bomb_timer(room, timer_len, current_prompt)
                .await
                .inspect_err(|error| eprintln!("word bomb timer error: {error:#?}"))
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
        fastest_guesses: game
            .players
            .iter()
            .filter_map(|player| {
                player
                    .used_words
                    .iter()
                    .map(|(duration, _)| duration.as_secs_f32())
                    .min_by(|a, b| a.partial_cmp(b).unwrap())
                    .map(|duration| (player.uuid, duration))
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
