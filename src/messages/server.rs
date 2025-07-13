use serde::Serialize;
use ts_rs::TS;
use uuid::Uuid;

#[derive(Serialize, TS)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
#[ts(export)]
pub enum ServerMessage {
    /// Can be sent from any state.
    General(ServerGeneral),
    /// Sent in lobby (exception `GameEnded`).
    Lobby(ServerLobby),
    /// Sent in game.
    Game(ServerGame),
}

#[derive(Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ServerGeneral {
    /// First message sent after establishing connection.
    Info {
        /// Joined client's designated UUID.
        uuid: Uuid,
        state: State,
    },
    /// Sent when a client joins/rejoins.
    Join {
        uuid: Uuid,
    },
    /// Sent when a client leaves.
    Leave {
        uuid: Uuid,
    },
    /// Used to broadcast a chat message.
    Chat {
        author: Uuid,
        content: String,
    },
    Error(String),
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
    Settings(Settings),
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
        state: State,
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
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ServerGame {
    /// Sent when the current game has ended.
    GameEnded {
        info: GameInfo,
        /// Is `Some` with a random client's uuid if the previous room owner
        /// left during game and hasn't come back.
        new_owner: Option<Uuid>,
    },
    WordBomb(ServerWordBomb),
}

#[derive(Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ServerWordBomb {
    /// Broadcasted when the currently active player is typing.
    Input { uuid: Uuid, input: String },
}

#[derive(Serialize, TS)]
#[ts(export)]
pub struct State;

#[derive(Serialize, TS)]
#[ts(export)]
pub struct Settings;

#[derive(Serialize, TS)]
#[ts(export)]
pub struct GameInfo;
