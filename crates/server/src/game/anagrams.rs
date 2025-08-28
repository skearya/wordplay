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
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct AnagramsState {
        pub players: HashMap<Uuid, AnagramsPlayer>,
        pub anagram: String,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
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
        original_word: String,
        leaderboard: Vec<(Uuid, u32)>,
        words: Vec<(Uuid, Vec<String>)>,
        guesses: Vec<(Uuid, u32)>,
    }
}

use std::{collections::HashMap, time::Duration};

use tokio::task::AbortHandle;
use uuid::Uuid;

use crate::{
    game::anagrams::messages::{
        AnagramsMessage, AnagramsPlayer, AnagramsPostGame, AnagramsSettings, AnagramsState,
        ClientAnagrams, ServerAnagrams,
    },
    global::{is_english, random_anagram},
    room::{handler::GameHandler, messenger::ClientMessenger, sender::AnagramsSender},
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
}

impl Anagrams {
    pub fn new(room: AnagramsSender, settings: &AnagramsSettings, players: &[Uuid]) -> Self {
        let (original, anagram) = random_anagram();

        let timer = task::spawn(async move {
            tokio::time::sleep(Duration::from_secs(30)).await;
            room.send(AnagramsMessage::TimerEnd);

            Ok(())
        })
        .abort_handle();

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
            let points = Self::points(&word);

            player.valid(word);

            Ok(points)
        }
    }

    fn points(word: &str) -> u32 {
        50 * 2_u32.pow(word.len() as u32 - 2)
    }
}

impl GameHandler for Anagrams {
    type ClientMessage = ClientAnagrams;
    type ServerMessage = ServerAnagrams;
    type RoomMessage = AnagramsMessage;
    type StateMessage = AnagramsState;
    type PostGameMessage = AnagramsPostGame;

    fn state(&self) -> Self::StateMessage {
        AnagramsState {
            players: self
                .players
                .iter()
                .map(|(&uuid, player)| {
                    (
                        uuid,
                        AnagramsPlayer {
                            points: player.used.iter().map(|word| Self::points(&word)).sum(),
                        },
                    )
                })
                .collect(),
            anagram: self.anagram.clone(),
        }
    }

    fn client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        (uuid, message): (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<Self::PostGameMessage>> {
        if !self.players.contains_key(&uuid) {
            return Err(anyhow::anyhow!("you aren't a player"));
        };

        match message {
            ClientAnagrams::Guess { word } => {
                match self.submission(uuid, word) {
                    Ok(points) => {
                        clients.broadcast(ServerAnagrams::Valid { uuid, points });
                    }
                    Err(reason) => {
                        clients.send(
                            uuid,
                            ServerAnagrams::Invalid {
                                reason: reason.to_owned(),
                            },
                        );
                    }
                };
            }
        }

        todo!()
    }

    fn room(
        &mut self,
        _clients: impl ClientMessenger<Self::ServerMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<Self::PostGameMessage>> {
        match message {
            AnagramsMessage::TimerEnd => todo!(),
        }
    }

    fn abort(&mut self) {
        self.timer.abort();
    }
}
