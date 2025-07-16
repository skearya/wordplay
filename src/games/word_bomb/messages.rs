use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WordBombSettings {}

#[derive(Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ClientWordBomb {
    Input { input: String },
    Guess { word: String },
}

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
        reason: &'static str,
    },
    /// Broadcasted previously active player failed to come up with a valid guess.
    Timeout {
        /// New prompt.
        prompt: String,
        /// Player UUID of new turn.
        turn: Uuid,
    },
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct WordBombPostGameInfo {
    winner: Uuid,
    mins_elapsed: f32,
    words_used: usize,
    fastest_guesses: Vec<(Uuid, f32)>,
    longest_words: Vec<(Uuid, String)>,
    avg_wpms: Vec<(Uuid, f32)>,
    avg_word_lengths: Vec<(Uuid, f32)>,
}
