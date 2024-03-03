use crate::global::GLOBAL;

use rand::{seq::SliceRandom, thread_rng, Rng};
use std::{collections::HashSet, sync::Arc, time::Instant};
use tokio::task::AbortHandle;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Lobby {
    pub ready: HashSet<(Uuid, String)>,
}

#[derive(Debug, Clone)]
pub struct InGame {
    pub timeout_task: Option<Arc<AbortHandle>>,
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
            ready: HashSet::new(),
        })
    }
}

impl Lobby {
    pub fn start_game(&self) -> GameState {
        let mut players: Vec<Player> = self
            .ready
            .iter()
            .map(|player| Player::new(player.0))
            .collect();

        players.shuffle(&mut thread_rng());

        GameState::InGame(InGame {
            timeout_task: None,
            timer_len: thread_rng().gen_range(10..=30),
            starting_time: Instant::now(),
            prompt: GLOBAL.get().unwrap().random_prompt(),
            prompt_uses: 0,
            current_turn: players[0].uuid,
            players,
        })
    }
}

impl InGame {
    pub fn new_prompt(&mut self) {
        let mut prompt = GLOBAL.get().unwrap().random_prompt();

        while self.prompt == prompt {
            prompt = GLOBAL.get().unwrap().random_prompt();
        }

        self.prompt = prompt;
        self.prompt_uses = 0;
    }

    pub fn update_turn(&mut self) {
        let mut index = self
            .players
            .iter()
            .position(|player| player.uuid == self.current_turn)
            .unwrap();

        let mut next_turn = self
            .players
            .get(index + 1)
            .unwrap_or_else(|| &self.players[0]);

        while next_turn.lives == 0 {
            index += 1;

            next_turn = self
                .players
                .get(index + 1)
                .unwrap_or_else(|| &self.players[0]);
        }

        self.current_turn = next_turn.uuid;
    }

    pub fn update_timer_len(&mut self) {
        self.timer_len = self
            .timer_len
            .saturating_sub(Instant::now().duration_since(self.starting_time).as_secs() as u8);

        if self.timer_len < 6 {
            self.timer_len = 6;
        }
    }

    pub fn progress(&mut self) {
        self.new_prompt();
        self.update_turn();
        self.update_timer_len();
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
        self.update_timer_len();
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
            ready: HashSet::new(),
        })
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
