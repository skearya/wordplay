use anyhow::{Context, Result};
use serde::Serialize;
use std::{collections::HashSet, time::Duration};
use uuid::Uuid;

use crate::{
    global::GLOBAL,
    messages::ServerMessage,
    state::{
        lobby::end_game,
        room::{ClientUtils, Room},
        AppState, SenderInfo,
    },
};

pub struct Anagrams {
    pub prompt: String,
    pub players: Vec<Player>,
}

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

impl Anagrams {
    pub fn check_guess(&mut self, uuid: Uuid, guess: &str) -> Option<GuessInfo> {
        if guess.len() < 3 {
            return Some(GuessInfo::NotLongEnough);
        }
        if guess
            .chars()
            .any(|ch| guess.matches(ch).count() > self.prompt.matches(ch).count())
        {
            return Some(GuessInfo::PromptMismatch);
        }
        if !GLOBAL.get().unwrap().is_valid(guess) {
            return Some(GuessInfo::NotEnglish);
        }
        if self
            .players
            .iter()
            .any(|player| player.used_words.contains(guess))
        {
            return Some(GuessInfo::AlreadyUsed);
        }

        self.players
            .iter_mut()
            .find(|player| uuid == player.uuid)?
            .used_words
            .insert(guess.to_string());

        Some(GuessInfo::Valid)
    }

    pub fn leaderboard(&self) -> Vec<(Uuid, u16)> {
        let mut points: Vec<(Uuid, u16)> = self
            .players
            .iter()
            .map(|player| {
                (
                    player.uuid,
                    player
                        .used_words
                        .iter()
                        .fold(0, |acc, word| acc + 50 * 2_u16.pow((word.len() - 2) as u32)),
                )
            })
            .collect();

        points.sort();
        points
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

        match game.check_guess(uuid, guess).context("Not a player")? {
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

        // TODO: send leaderboard instead of singular winner?
        let leaderboard = game.leaderboard();
        end_game(state, clients, owner, leaderboard.last().unwrap().0);

        Ok(())
    }
}
