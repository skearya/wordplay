pub mod messages {
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::game::messages::PostGameInfo;

    #[derive(Deserialize, TS)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    #[ts(export)]
    pub enum ClientLobby {
        Ready,
        StartEarly,
        Unready,
        PracticeRequest,
        PracticeSubmission { prompt: String, input: String },
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(
        tag = "kind",
        rename_all = "camelCase",
        rename_all_fields = "camelCase"
    )]
    #[ts(export)]
    pub enum ServerLobby {
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
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub enum TimerAction {
        Start,
        Stop,
        None,
    }

    pub enum LobbyMessage {
        GameStart,
    }

    #[cfg_attr(test, derive(Deserialize, Debug, PartialEq))]
    #[derive(Serialize, TS)]
    #[serde(rename_all = "camelCase")]
    #[ts(export)]
    pub struct LobbyState {
        pub ready: Vec<Uuid>,
        /// Unix timestamp of when the countdown timer started.
        pub timer_start: Option<u64>,
        // Previous game info.
        pub prev_game: Option<PostGameInfo>,
    }
}

use std::{
    mem,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use tokio::task::AbortHandle;
use uuid::Uuid;

use crate::{
    game::messages::PostGameInfo,
    lobby::messages::{ClientLobby, LobbyMessage, LobbyState, ServerLobby, TimerAction},
    messages::RoomSettings,
    room::{StateChange, handler::Handler, messenger::ClientMessenger, sender::LobbySender},
    task,
};

pub struct Lobby {
    room: LobbySender,
    ready: Vec<Uuid>,
    countdown: Option<Countdown>,
    prev_game_info: Option<PostGameInfo>,
}

struct Countdown {
    /// Start time of the timer (milliseconds since unix epoch).
    start: u64,
    timer: AbortHandle,
}

impl Lobby {
    pub fn new(room: LobbySender, prev_game_info: Option<PostGameInfo>) -> Self {
        Self {
            room,
            ready: vec![],
            countdown: None,
            prev_game_info,
        }
    }

    fn update_countdown(&mut self) -> TimerAction {
        match &mut self.countdown {
            None if self.ready.len() >= 2 => {
                let start = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("time has gone backwards")
                    .as_millis() as u64;

                let room = self.room.clone();

                let timer = task::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    room.send(LobbyMessage::GameStart);

                    Ok(())
                })
                .abort_handle();

                self.countdown = Some(Countdown { start, timer });

                TimerAction::Start
            }
            Some(countdown) if self.ready.len() < 2 => {
                countdown.timer.abort();
                self.countdown = None;

                TimerAction::Stop
            }
            Some(_) | None => TimerAction::None,
        }
    }
}

impl Handler for Lobby {
    type ClientMessage = ClientLobby;
    type ServerMessage = ServerLobby;
    type RoomMessage = LobbyMessage;
    type StateMessage = LobbyState;

    fn state(&self) -> Self::StateMessage {
        LobbyState {
            ready: self.ready.clone(),
            timer_start: self.countdown.as_ref().map(|countdown| countdown.start),
            prev_game: self.prev_game_info.clone(),
        }
    }

    fn client(
        &mut self,
        settings: &mut RoomSettings,
        clients: impl ClientMessenger<Self::ServerMessage>,
        (uuid, message): (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<StateChange> {
        match message {
            ClientLobby::Ready => {
                if self.ready.contains(&uuid) {
                    return Ok(StateChange::None);
                }

                self.ready.push(uuid);

                clients.broadcast(ServerLobby::Ready {
                    uuid,
                    timer: self.update_countdown(),
                });
            }
            ClientLobby::StartEarly => {
                if settings.owner != uuid || self.ready.len() < 2 {
                    return Ok(StateChange::None);
                }

                return Ok(StateChange::Game(mem::take(&mut self.ready)));
            }
            ClientLobby::Unready => {
                let Some(index) = self.ready.iter().position(|client| *client == uuid) else {
                    return Ok(StateChange::None);
                };

                self.ready.remove(index);

                clients.broadcast(ServerLobby::Unready {
                    uuid,
                    timer: self.update_countdown(),
                });
            }
            ClientLobby::PracticeRequest => todo!(),
            ClientLobby::PracticeSubmission { prompt, input } => todo!(),
        }

        Ok(StateChange::None)
    }

    fn room(
        &mut self,
        _settings: &mut RoomSettings,
        _clients: impl ClientMessenger<Self::ServerMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<StateChange> {
        match message {
            LobbyMessage::GameStart => Ok(StateChange::Game(mem::take(&mut self.ready))),
        }
    }

    fn abort(&mut self) {
        if let Some(countdown) = &self.countdown {
            countdown.timer.abort();
        }
    }
}
