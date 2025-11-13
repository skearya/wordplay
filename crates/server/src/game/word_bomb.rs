pub mod messages {
    use std::collections::HashMap;

    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    #[cfg_attr(test, derive(Debug, PartialEq))]
    #[derive(Serialize, Deserialize, TS, Clone, Copy)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct WordBombSettings {
        pub min_wpm: usize,
    }

    impl Default for WordBombSettings {
        fn default() -> Self {
            Self { min_wpm: 500 }
        }
    }

    #[derive(Deserialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ClientWordBomb {
        Input { input: String },
        Guess { word: String },
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ServerWordBomb {
        /// Broadcasted when the currently active player is typing.
        Input { input: String },
        Valid {
            /// Current player's guess that was valid.
            guess: String,
            /// True if the current player has used up all letters.
            life: bool,
            /// New prompt.
            prompt: String,
            /// Player UUID of new turn.
            turn: Uuid,
        },
        Invalid {
            /// Reason for invalid guess (ex: "guess doesn't include prompt")
            reason: String,
        },
        /// Broadcasted previously active player failed to come up with a valid guess.
        Exploded {
            /// New prompt.
            prompt: String,
            /// Player UUID of new turn.
            turn: Uuid,
        },
    }

    pub enum WordBombMessage {
        Exploded,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS, Clone)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct WordBombState {
        pub players: HashMap<Uuid, WordBombPlayer>,
        pub turn: Uuid,
        pub prompt: String,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS, Clone)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct WordBombPlayer {
        pub input: String,
        pub lives: u8,
        pub letters: Vec<char>,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS, Clone)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct WordBombPostGame {
        pub winner: Uuid,
        // pub mins_elapsed: f32,
        // pub words_used: usize,
        // pub fastest_guesses: Vec<(Uuid, f32)>,
        // pub longest_words: Vec<(Uuid, String)>,
        // pub avg_wpms: Vec<(Uuid, f32)>,
        // pub avg_word_lengths: Vec<(Uuid, f32)>,
    }
}

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use rand::seq::SliceRandom;
use tokio::task::AbortHandle;
use uuid::Uuid;

use crate::{
    game::{
        GameContext, GameHandler,
        messages::{GameVariantState, PostGameInfo},
        word_bomb::messages::{
            ClientWordBomb, ServerWordBomb, WordBombMessage, WordBombPlayer, WordBombPostGame,
            WordBombSettings, WordBombState,
        },
    },
    global::{is_english, random_prompt},
    room::sender::RoomSender,
    task,
};

pub struct WordBomb {
    room: RoomSender,
    settings: WordBombSettings,

    players: HashMap<Uuid, Player>,
    /// Order of players.
    order: Vec<Uuid>,
    /// Index into `self.order`.
    turn: usize,
    /// Current prompt, prompt uses.
    prompt: Prompt,
    /// Handle to task that sends `WordBombMessage::Exploded`, time spawned.
    timer: Timer,

    /// Record of when the game started. Used to calculate game length.
    start: Instant,
    /// Player, used word.
    used: Vec<(Uuid, String)>,
    /// Player, prompt that they exploded to.
    exploded: Vec<(Uuid, &'static str)>,
}

struct Player {
    lives: u8,
    input: String,
    // 4 bytes = ceil(26 letters / 8 bits)
    letters: u32,
    /// Amount of times player guessed incorrectly.
    incorrect: u32,
}

struct Prompt {
    text: &'static str,
    uses: u8,
}

struct Timer {
    task: AbortHandle,
    start: Instant,
    length: f64,
}

impl Player {
    fn new() -> Self {
        Self {
            lives: 2,
            input: String::with_capacity(8),
            letters: 0,
            incorrect: 0,
        }
    }

    /// Returns `true` if player gained a life. `word` must be ascii and lowercase.
    fn valid(&mut self, word: &str) -> bool {
        for c in word.chars() {
            let index = c as u8 - b'a';

            self.letters |= 1 << index;
        }

        if self.letters == 2u32.pow(26) - 1 {
            self.lives += 1;
            self.letters = 0;

            true
        } else {
            false
        }
    }

    fn incorrect(&mut self) {
        self.incorrect += 1;
    }

    fn exploded(&mut self) {
        self.lives -= 1;
    }

    fn alive(&self) -> bool {
        self.lives != 0
    }
}

impl WordBomb {
    pub fn new(ctx: GameContext, players: &[Uuid]) -> Self {
        let mut order = players.to_owned();
        order.shuffle(&mut rand::rng());

        let start = Instant::now();
        let length = rand::random_range(10.0..=30.0);

        let task = {
            let room = ctx.room.clone();

            task::spawn(async move {
                tokio::time::sleep_until((start + Duration::from_secs_f64(length)).into()).await;
                room.send(WordBombMessage::Exploded);

                Ok(())
            })
            .abort_handle()
        };

        Self {
            room: ctx.room.clone(),
            settings: ctx.settings.word_bomb,
            players: players
                .iter()
                .copied()
                .map(|uuid| (uuid, Player::new()))
                .collect(),
            order,
            turn: 0,
            prompt: Prompt {
                text: random_prompt(ctx.settings.word_bomb.min_wpm),
                uses: 0,
            },
            timer: Timer {
                task,
                start,
                length,
            },
            start,
            used: vec![],
            exploded: vec![],
        }
    }

    /// Returns `Ok(life)` or `Err(reason)`.
    fn submission(&mut self, mut word: String) -> Result<bool, &'static str> {
        word.retain(|c| c.is_ascii_alphabetic());
        word.make_ascii_lowercase();

        let error = if !word.contains(self.prompt.text) {
            Some("word doesn't contain prompt")
        } else if self.used.iter().any(|(_, used)| *used == word) {
            Some("word has already been used")
        } else if !is_english(&word) {
            Some("word is not english")
        } else {
            None
        };

        if let Some(error) = error {
            self.active().incorrect();

            Err(error)
        } else {
            let life = self.active().valid(&word);

            self.used.push((self.order[self.turn], word));
            self.timer.task.abort();

            // We should always be able to advance after a correct submission.
            assert!(self.advance(false));

            Ok(life)
        }
    }

    /// Returns `true` if advanced, `false` if game ended.
    fn explosion(&mut self) -> bool {
        self.active().exploded();

        self.exploded
            .push((self.order[self.turn], self.prompt.text));

        self.advance(true)
    }

    /// Returns `true` if advanced, `false` if game ended.
    fn advance(&mut self, exploded: bool) -> bool {
        let alive = self
            .players
            .values()
            .filter(|player| player.alive())
            .count();

        if alive == 1 {
            return false;
        }

        loop {
            self.turn = (self.turn + 1) % self.order.len();

            if self.active().alive() {
                break;
            }
        }

        self.prompt = if exploded && let Prompt { text, uses: 0 } = self.prompt {
            Prompt { text, uses: 1 }
        } else {
            Prompt {
                text: random_prompt(self.settings.min_wpm),
                uses: 0,
            }
        };

        self.spawn_timer(exploded);

        true
    }

    fn spawn_timer(&mut self, exploded: bool) {
        let start = Instant::now();

        let length = if exploded {
            rand::random_range(10.0..=30.0)
        } else {
            (self.timer.length - self.timer.start.duration_since(start).as_secs_f64()).max(5.0)
        };

        let room = self.room.clone();

        let task = task::spawn(async move {
            tokio::time::sleep_until((start + Duration::from_secs_f64(length)).into()).await;
            room.send(WordBombMessage::Exploded);

            Ok(())
        })
        .abort_handle();

        self.timer = Timer {
            task,
            start,
            length,
        };
    }

    fn info(&self) -> WordBombPostGame {
        WordBombPostGame {
            winner: *self
                .players
                .iter()
                .find(|player| player.1.alive())
                .expect("one player should be alive")
                .0,
        }
    }

    fn active(&mut self) -> &mut Player {
        self.players
            .get_mut(&self.order[self.turn])
            .expect("players[order[?]] shouldn't fail")
    }
}

impl GameHandler for WordBomb {
    type ClientMessage = ClientWordBomb;
    type SelfMessage = WordBombMessage;
    type Outcome = WordBombPostGame;
    type Snapshot = WordBombState;

    fn on_client_message(
        &mut self,
        ctx: super::GameContext,
        (uuid, message): (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<Self::Outcome>> {
        let Some(player) = self.players.get_mut(&uuid) else {
            return Err(anyhow::anyhow!("you aren't a player"));
        };

        if uuid != self.order[self.turn] {
            return Err(anyhow::anyhow!("it's not your turn"));
        }

        match message {
            ClientWordBomb::Input { input } => {
                player.input = input;

                ctx.clients.broadcast(ServerWordBomb::Input {
                    input: player.input.clone(),
                });
            }
            ClientWordBomb::Guess { word } => match self.submission(word.clone()) {
                Ok(life) => {
                    ctx.clients.broadcast(ServerWordBomb::Valid {
                        guess: word,
                        life,
                        prompt: self.prompt.text.to_owned(),
                        turn: self.order[self.turn],
                    });
                }
                Err(reason) => {
                    ctx.clients.broadcast(ServerWordBomb::Invalid {
                        reason: reason.to_owned(),
                    });
                }
            },
        }

        Ok(None)
    }

    fn on_self_message(
        &mut self,
        ctx: super::GameContext,
        message: Self::SelfMessage,
    ) -> anyhow::Result<Option<Self::Outcome>> {
        let info = match message {
            WordBombMessage::Exploded => {
                if self.explosion() {
                    ctx.clients.broadcast(ServerWordBomb::Exploded {
                        prompt: self.prompt.text.to_owned(),
                        turn: self.order[self.turn],
                    });

                    None
                } else {
                    Some(self.info())
                }
            }
        };

        Ok(info)
    }

    fn on_abort(&mut self) {
        self.timer.task.abort();
    }

    fn snapshot(&self) -> Self::Snapshot {
        WordBombState {
            players: self
                .players
                .iter()
                .map(|(&uuid, player)| {
                    (
                        uuid,
                        WordBombPlayer {
                            input: player.input.clone(),
                            lives: player.lives,
                            letters: (0..26)
                                .map(|i| (player.letters >> i & 1) as u8)
                                .map(|value| (value + b'a') as char)
                                .collect(),
                        },
                    )
                })
                .collect(),
            turn: self.order[self.turn],
            prompt: self.prompt.text.to_owned(),
        }
    }
}

impl From<WordBombState> for GameVariantState {
    fn from(value: WordBombState) -> Self {
        Self::WordBomb(value)
    }
}

impl From<WordBombPostGame> for PostGameInfo {
    fn from(value: WordBombPostGame) -> Self {
        Self::WordBomb(value)
    }
}
