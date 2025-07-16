use std::collections::HashMap;

use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

use crate::{
    games::word_bomb::messages::{ServerWordBomb, WordBombPostGameInfo},
    messages::shared::RoomSettings,
};

#[derive(Serialize, TS)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
#[ts(export)]
pub enum ServerMessage {
    /// First message sent after establishing connection, sent only once.
    Info {
        /// Joined client's designated UUID.
        uuid: Uuid,
        /// Room owner's UUID.
        owner: Uuid,
        /// Room and game settings.
        settings: RoomSettings,
        /// Room clients.
        clients: HashMap<Uuid, Client>,
        /// State of the room (lobby | type of game).
        state: RoomState,
    },
    /// Can be sent from any state.
    General(ServerGeneral),
    /// All lobby messages.
    Lobby(ServerLobby),
    /// All general in-game messages.
    InGame(ServerInGame),
    /// All Word Bomb messages.
    WordBomb(ServerWordBomb),
}

#[derive(Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub struct Client {
    username: String,
    /// URL to account avatar.
    avatar_url: Option<String>,
    /// `true` if player disconnected in game only.
    disconnected: bool,
}

#[derive(Serialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
/// Room state sent to clients when they join.
pub enum RoomState {
    Lobby {
        ready: Vec<Uuid>,
        /// Unix timestamp of when the countdown timer started.
        timer_start: Option<u64>,
        /// Previous game info.
        prev_game: Option<PostGameInfo>,
    },
    Game {
        // Specific game type state (ex: word bomb).
        state: GameState,
        /// UUIDs of players requesting to end the current game.
        requesting_end: Option<Vec<Uuid>>,
    },
}

#[derive(Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum GameState {
    WordBomb(ServerWordBomb),
}

#[derive(Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ServerGeneral {
    /// Broadcasted when a client joins/rejoins.
    Join {
        uuid: Uuid,
    },
    /// Broadcasted when a client leaves.
    Leave {
        uuid: Uuid,
    },
    /// Used to broadcast a chat message.
    Chat {
        author: Uuid,
        content: String,
    },
    Error {
        message: String,
    },
}

#[derive(Serialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum ServerLobby {
    /// Sent when the room owner has updated room/game settings.
    Settings(RoomSettings),
    Ready {
        uuid: Uuid,
        timer: TimerAction,
    },
    Unready {
        uuid: Uuid,
        timer: TimerAction,
    },
    /// Set of practice prompts for the current gamemode. Sent on explicit request.
    Practice {
        prompts: Vec<String>,
    },
    /// Send result of practice guess based on current gamemode.
    PracticeResult {
        correct: bool,
    },
    /// Sent when the game (based on room settings) has started.
    GameStarted {
        /// Contains the player's rejoin token. Is `None` if client is spectating.
        rejoin_token: Option<Uuid>,
        state: GameState,
    },
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum TimerAction {
    Start,
    Stop,
    None,
}

#[derive(Serialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[ts(export)]
pub enum ServerInGame {
    /// Broadcasted when a player requests to end the game early.
    EndRequest { uuid: Uuid },
    /// Broadcasted when the current game has ended.
    Ended {
        post_game_info: PostGameInfo,
        /// Is `Some` with a random client's uuid if the previous room owner
        /// left during game and hasn't come back.
        new_owner: Option<Uuid>,
    },
}

#[derive(Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum PostGameInfo {
    WordBomb(WordBombPostGameInfo),
}
