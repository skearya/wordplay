use crate::{
    global::GLOBAL,
    messages::{self, ServerMessage},
    state::{
        error::{AnagramsError, GameError},
        lobby::end_game,
        room::Room,
        AppState, SenderInfo,
    },
    utils::{ClientUtils, Sorted},
};
use serde::Serialize;
use std::{collections::HashSet, sync::Arc, time::Duration};
use tokio::task::AbortHandle;
use uuid::Uuid;

#[derive(Debug)]
pub struct Anagrams {
    pub timer: Arc<AbortHandle>,
    pub anagram: String,
    pub original: String,
    pub players: Vec<Player>,
}

#[derive(Serialize, Debug, Clone)]
pub struct Player {
    pub uuid: Uuid,
    pub used_words: HashSet<String>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum GuessInfo {
    NotLongEnough,
    PromptMismatch,
    NotEnglish,
    AlreadyUsed,
    Valid,
}

#[derive(Serialize, Clone)]
pub struct PostGameInfo {
    original_word: String,
    leaderboard: Vec<(Uuid, u32)>,
}

impl Anagrams {
    pub fn check_guess(&mut self, uuid: Uuid, guess: &str) -> Result<GuessInfo, GameError> {
        let guess_info = if guess.len() < 3 {
            GuessInfo::NotLongEnough
        } else if guess
            .chars()
            .any(|ch| guess.matches(ch).count() > self.anagram.matches(ch).count())
        {
            GuessInfo::PromptMismatch
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
                .find(|player| uuid == player.uuid)
                .ok_or(AnagramsError::PlayerNotFound)?
                .used_words
                .insert(guess.to_string());

            GuessInfo::Valid
        };

        Ok(guess_info)
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
    pub fn anagrams_guess(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        guess: String,
    ) -> Result<(), GameError> {
        if guess.len() > 6 {
            return Err(AnagramsError::GuessTooLong)?;
        }

        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state, .. } = lock.room_mut(room)?;
        let game = state.try_anagrams()?;

        match game.check_guess(uuid, &guess) {
            Ok(GuessInfo::Valid) => {
                clients.broadcast(ServerMessage::AnagramsCorrectGuess { uuid, guess });
            }
            Ok(reason) => clients[&uuid].send(ServerMessage::AnagramsInvalidGuess { reason }),
            Err(error) => return Err(error),
        }

        Ok(())
    }

    pub async fn anagrams_timer(&self, room: String) -> Result<(), GameError> {
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
