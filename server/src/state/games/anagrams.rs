use anyhow::{anyhow, Result};
use serde::Serialize;
use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::task::AbortHandle;
use uuid::Uuid;

use crate::{
    global::GLOBAL,
    messages::{self, ServerMessage},
    state::{
        lobby::end_game,
        room::{ClientUtils, Room},
        AppState, SenderInfo,
    },
    utils::Sorted,
};

pub struct Anagrams {
    pub timer: Arc<AbortHandle>,
    pub anagram: String,
    pub original: String,
    pub players: Vec<Player>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub uuid: Uuid,
    pub used_words: HashSet<String>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum GuessInfo {
    NotLongEnough,
    PromptMismatch,
    NotEnglish,
    AlreadyUsed,
    Valid,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PostGameInfo {
    original_word: String,
    leaderboard: Vec<(Uuid, u32)>,
}

impl Anagrams {
    pub fn check_guess(&mut self, uuid: Uuid, guess: &str) -> GuessInfo {
        if guess.len() < 3 {
            return GuessInfo::NotLongEnough;
        }
        if guess
            .chars()
            .any(|ch| guess.matches(ch).count() > self.anagram.matches(ch).count())
        {
            return GuessInfo::PromptMismatch;
        }
        if !GLOBAL.get().unwrap().is_valid(guess) {
            return GuessInfo::NotEnglish;
        }
        if self
            .players
            .iter()
            .any(|player| player.used_words.contains(guess))
        {
            return GuessInfo::AlreadyUsed;
        }

        self.players
            .iter_mut()
            .find(|player| uuid == player.uuid)
            .unwrap()
            .used_words
            .insert(guess.to_string());

        GuessInfo::Valid
    }

    pub fn leaderboard(&self) -> Vec<(Uuid, u32)> {
        self.players
            .iter()
            .map(|player| {
                (
                    player.uuid,
                    player
                        .used_words
                        .iter()
                        .fold(0, |acc, word| acc + 50 * 2_u32.pow((word.len() - 2) as u32)),
                )
            })
            .sorted_by_vec(|a, b| b.1.cmp(&a.1))
    }
}

impl Player {
    pub fn new(uuid: Uuid) -> Self {
        Self {
            uuid,
            used_words: HashSet::new(),
        }
    }
}

impl AppState {
    pub fn anagrams_guess(&self, SenderInfo { uuid, room }: SenderInfo, guess: &str) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state, .. } = lock.room_mut(room)?;
        let game = state.try_anagrams()?;

        if game
            .players
            .iter_mut()
            .find(|player| uuid == player.uuid)
            .is_none()
        {
            return Err(anyhow!("Not a player"));
        }

        match game.check_guess(uuid, guess) {
            GuessInfo::Valid => {
                clients.broadcast(ServerMessage::AnagramsCorrectGuess {
                    uuid,
                    guess: guess.to_string(),
                });
            }
            guess_info => {
                clients[&uuid]
                    .tx
                    .send(ServerMessage::AnagramsInvalidGuess { reason: guess_info }.into())
                    .ok();
            }
        }

        Ok(())
    }

    pub async fn anagrams_timer(&self, room: String) -> Result<()> {
        tokio::time::sleep(Duration::from_secs(30)).await;

        let mut lock = self.inner.lock().unwrap();
        let Room {
            clients,
            state,
            owner,
            ..
        } = lock.room_mut(&room)?;
        let game = state.try_anagrams()?;

        let game_info = messages::PostGameInfo::Anagrams(PostGameInfo {
            original_word: game.original.clone(),
            leaderboard: game.leaderboard(),
        });

        end_game(state, clients, owner, game_info);

        Ok(())
    }
}
