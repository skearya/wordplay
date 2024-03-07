use crate::global::GLOBAL;

use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{collections::HashSet, sync::Arc, time::Instant};
use tokio::task::AbortHandle;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Lobby {
    pub ready: HashSet<Uuid>,
    pub start_handle: Option<Arc<AbortHandle>>,
}

#[derive(Debug, Clone)]
pub struct InGame {
    pub timeout_task: Arc<AbortHandle>,
    pub timer_len: u8,
    pub starting_time: Instant,
    pub prompt: String,
    pub prompt_uses: u8,
    pub players: Vec<Player>,
    pub current_turn: Uuid,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub uuid: Uuid,
    pub rejoin_token: Uuid,
    pub input: String,
    pub lives: u8,
    pub used_letters: HashSet<char>,
}

#[derive(Debug, Clone)]
pub enum GameState {
    Lobby(Lobby),
    InGame(InGame),
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Lobby(Lobby {
            start_handle: None,
            ready: HashSet::new(),
        })
    }
}

impl Lobby {
    pub fn start_game<F>(&self, timeout_task_handle: F) -> GameState
    where
        F: FnOnce(String, u8) -> Arc<AbortHandle>,
    {
        let timer_len = thread_rng().gen_range(10..=30);
        let prompt = GLOBAL.get().unwrap().random_prompt();
        let mut players: Vec<Player> = self.ready.iter().map(|uuid| Player::new(*uuid)).collect();

        players.shuffle(&mut thread_rng());

        GameState::InGame(InGame {
            timeout_task: timeout_task_handle(prompt.clone(), timer_len),
            timer_len,
            starting_time: Instant::now(),
            prompt,
            prompt_uses: 0,
            current_turn: players[0].uuid,
            players,
        })
    }
}

impl InGame {
    pub fn check_for_extra_life(&mut self, guess: &str) -> bool {
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

        extra_life
    }

    pub fn new_prompt(&mut self) {
        self.prompt = loop {
            let new_prompt = GLOBAL.get().unwrap().random_prompt();

            if new_prompt != self.prompt {
                break new_prompt;
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
            .saturating_sub(Instant::now().duration_since(self.starting_time).as_secs() as u8);

        self.timer_len = self.timer_len.max(6);
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

    pub fn end(&self) -> GameState {
        GameState::Lobby(Lobby {
            start_handle: None,
            ready: HashSet::new(),
        })
    }
}

impl Player {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            rejoin_token: Uuid::new_v4(),
            input: String::new(),
            lives: 2,
            used_letters: HashSet::new(),
        }
    }
}
