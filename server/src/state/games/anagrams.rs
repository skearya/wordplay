use anyhow::{Context, Result};
use std::{collections::HashSet, time::Duration};
use uuid::Uuid;

use crate::{
    global::GLOBAL,
    messages::ServerMessage,
    state::{
        lobby::Lobby,
        room::{check_for_new_room_owner, ClientUtils, Room, State},
        AppState, SenderInfo,
    },
};

pub struct Anagrams {
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

    pub fn winner(&self) -> Uuid {
        // TODO
        self.players.first().unwrap().uuid
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

impl AppState {
    pub fn anagrams_guess(&self, SenderInfo { uuid, room }: SenderInfo, guess: &str) -> Result<()> {
        let mut lock = self.inner.lock().unwrap();
        let Room { clients, state, .. } = lock.room_mut(&room)?;
        let game = state.try_anagrams()?;

        match game.check_guess(uuid, guess).context("Not a player")? {
            GuessInfo::Valid => clients.broadcast(ServerMessage::AnagramsCorrectGuess {
                uuid,
                guess: guess.to_string(),
            }),
            _ => {}
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

        clients.retain(|_uuid, client| client.socket.is_some());

        let new_room_owner = check_for_new_room_owner(clients, owner);

        clients.broadcast(ServerMessage::GameEnded {
            winner: game.winner(),
            new_room_owner,
        });

        *state = Anagrams::end();

        Ok(())
    }
}