pub mod messages {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    #[cfg_attr(test, derive(Debug, PartialEq))]
    #[derive(Serialize, Deserialize, TS, Clone, Copy)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AnagramsSettings {}

    impl Default for AnagramsSettings {
        fn default() -> Self {
            Self {}
        }
    }

    #[derive(Deserialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ClientAnagrams {
        Guess { word: String },
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ServerAnagrams {
        Valid {
            uuid: Uuid,
            points: u32,
        },
        Invalid {
            /// Reason for invalid guess (ex: "guess doesn't include prompt")
            reason: String,
        },
    }

    pub enum AnagramsMessage {
        TimerEnd,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS, Clone)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AnagramsState {
        pub players: HashMap<Uuid, AnagramsPlayer>,
        pub anagram: String,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS, Clone)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AnagramsPlayer {
        pub points: u32,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS, Clone)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AnagramsPostGame {
        pub original: String,
        pub leaderboard: Vec<(Uuid, u32)>,
        // pub words: Vec<(Uuid, Vec<String>)>,
    }
}

use std::{collections::HashMap, time::Duration};

use tokio::task::AbortHandle;
use uuid::Uuid;

use crate::{
    game::{
        GameContext, GameHandler,
        anagrams::messages::{
            AnagramsMessage, AnagramsPlayer, AnagramsPostGame, AnagramsState, ClientAnagrams,
            ServerAnagrams,
        },
        messages::{GameVariantState, PostGameInfo},
    },
    global::{is_english, random_anagram},
    task,
};

pub struct Anagrams {
    /// Players.
    players: HashMap<Uuid, Player>,
    /// Original word that the anagram was scrambled on.
    original: &'static str,
    /// 6 letter anagram.
    anagram: String,
    /// Abort handle to timer task.
    timer: AbortHandle,
}

struct Player {
    /// Player's correctly guessed words.
    used: Vec<String>,
    /// Amount of times player guessed incorrectly.
    incorrect: u32,
}

impl Player {
    fn new() -> Self {
        Self {
            used: vec![],
            incorrect: 0,
        }
    }

    fn valid(&mut self, word: String) {
        self.used.push(word);
    }

    fn incorrect(&mut self) {
        self.incorrect += 1;
    }

    fn points(&self) -> u32 {
        self.used.iter().map(|word| points(word)).sum()
    }
}

fn points(word: &str) -> u32 {
    50 * 2_u32.pow(word.len() as u32 - 2)
}

impl Anagrams {
    pub fn new(ctx: GameContext, players: &[Uuid]) -> Self {
        let room = ctx.room.clone();

        let timer = task::spawn(async move {
            tokio::time::sleep(Duration::from_secs(30)).await;
            room.send(AnagramsMessage::TimerEnd);

            Ok(())
        })
        .abort_handle();

        let (original, anagram) = random_anagram();

        Self {
            players: players.iter().map(|&uuid| (uuid, Player::new())).collect(),
            original,
            anagram,
            timer,
        }
    }

    /// Returns `Ok(points)` or `Err(reason)`.
    fn submission(&mut self, uuid: Uuid, word: String) -> Result<u32, &'static str> {
        let error = if word.len() < 2 {
            Some("word isn't long enough")
        } else if word
            .chars()
            .any(|c| word.matches(c).count() > self.anagram.matches(c).count())
        {
            Some("word doesn't match anagram")
        } else if self.players[&uuid].used.contains(&word) {
            Some("word has already been used")
        } else if !is_english(&word) {
            Some("word is not english")
        } else {
            None
        };

        let player = self
            .players
            .get_mut(&uuid)
            .expect("uuid should've been validated");

        if let Some(error) = error {
            player.incorrect();

            Err(error)
        } else {
            let points = points(&word);

            player.valid(word);

            Ok(points)
        }
    }

    fn info(&self) -> AnagramsPostGame {
        AnagramsPostGame {
            original: self.original.to_owned(),
            leaderboard: self
                .players
                .iter()
                .map(|(&uuid, player)| (uuid, player.points()))
                .collect(),
        }
    }
}

impl GameHandler for Anagrams {
    type ClientMessage = ClientAnagrams;
    type SelfMessage = AnagramsMessage;
    type Outcome = AnagramsPostGame;
    type Snapshot = AnagramsState;

    fn on_client_message(
        &mut self,
        ctx: GameContext,
        (uuid, message): (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<Self::Outcome>> {
        if !self.players.contains_key(&uuid) {
            return Err(anyhow::anyhow!("you aren't a player"));
        }

        match message {
            ClientAnagrams::Guess { word } => match self.submission(uuid, word) {
                Ok(points) => {
                    ctx.clients
                        .broadcast(ServerAnagrams::Valid { uuid, points });
                }
                Err(reason) => {
                    ctx.clients.send(
                        uuid,
                        ServerAnagrams::Invalid {
                            reason: reason.to_owned(),
                        },
                    );
                }
            },
        }

        todo!()
    }

    fn on_self_message(
        &mut self,
        _ctx: GameContext,
        message: Self::SelfMessage,
    ) -> anyhow::Result<Option<Self::Outcome>> {
        match message {
            AnagramsMessage::TimerEnd => Ok(Some(self.info())),
        }
    }

    fn on_abort(&mut self) {
        self.timer.abort();
    }

    fn snapshot(&self) -> Self::Snapshot {
        AnagramsState {
            players: self
                .players
                .iter()
                .map(|(&uuid, player)| {
                    (
                        uuid,
                        AnagramsPlayer {
                            points: player.points(),
                        },
                    )
                })
                .collect(),
            anagram: self.anagram.clone(),
        }
    }
}

impl From<AnagramsState> for GameVariantState {
    fn from(value: AnagramsState) -> Self {
        Self::Anagrams(value)
    }
}

impl From<AnagramsPostGame> for PostGameInfo {
    fn from(value: AnagramsPostGame) -> Self {
        Self::Anagrams(value)
    }
}
