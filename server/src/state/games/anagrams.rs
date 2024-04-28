use std::{collections::HashSet, sync::Arc};
use tokio::task::AbortHandle;
use uuid::Uuid;

use crate::{
    global::GLOBAL,
    state::{lobby::Lobby, room::State},
};

pub struct Anagrams {
    pub timeout_task: Arc<AbortHandle>,
    pub prompt: String,
    pub players: Vec<Player>,
}

pub struct Player {
    pub uuid: Uuid,
    pub rejoin_token: Uuid,
    pub used_words: HashSet<String>,
}

pub enum GuessInfo {
    PromptNotIn,
    NotEnglish,
    AlreadyUsed,
    Valid,
}

impl Anagrams {
    pub fn check_guess(&mut self, uuid: Uuid, guess: &str) -> Option<GuessInfo> {
        // TODO: i think each letter can only be used once?
        let guess_info = if !guess.chars().all(|char| self.prompt.contains(char)) {
            GuessInfo::PromptNotIn
        } else if !GLOBAL.get().unwrap().is_valid(guess) {
            GuessInfo::NotEnglish
        } else if self
            .players
            .iter()
            .any(|player| player.used_words.contains(guess))
        {
            GuessInfo::AlreadyUsed
        } else {
            self.players
                .iter_mut()
                .find(|player| uuid == player.uuid)?
                .used_words
                .insert(guess.to_string());

            GuessInfo::Valid
        };

        Some(guess_info)
    }

    pub fn end() -> State {
        State::Lobby(Lobby {
            ready: HashSet::new(),
            countdown: None,
        })
    }
}

impl Player {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            rejoin_token: Uuid::new_v4(),
            used_words: HashSet::new(),
        }
    }
}
