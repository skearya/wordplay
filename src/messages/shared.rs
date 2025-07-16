use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::games::word_bomb::messages::WordBombSettings;

#[derive(Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoomSettings {
    word_bomb: WordBombSettings,
}
