pub mod messages {
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::game::messages::GameState;

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
        /// Sent when the game (based on room settings) has started.
        GameStarted {
            /// Contains the player's rejoin token. Is `None` if client is spectating.
            rejoin_token: Option<Uuid>,
            state: GameState,
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
        // TODO: Show previous game info.
        // prev_game: Option<PostGameInfo>,
    }
}

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::task::AbortHandle;
use uuid::Uuid;

use crate::{
    game::Game,
    lobby::messages::{ClientLobby, LobbyMessage, LobbyState, ServerLobby, TimerAction},
    messages::RoomSettings,
    room::{
        handler::Handler,
        messenger::{ClientMessenger, ClientUtils, RoomMessenger},
        state::State,
    },
    task,
};

#[derive(Default)]
pub struct Lobby {
    ready: Vec<Uuid>,
    countdown: Option<Countdown>,
}

struct Countdown {
    /// Start time of the timer (milliseconds since unix epoch).
    start: u64,
    timer: AbortHandle,
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            ready: vec![],
            countdown: None,
        }
    }
}

impl Handler for Lobby {
    type ClientMessage = ClientLobby;
    type ServerMessage = ServerLobby;
    type RoomMessage = LobbyMessage;
    type StateMessage = LobbyState;

    fn handle_client(
        &mut self,
        settings: &mut RoomSettings,
        clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils,
        room: impl RoomMessenger<Self::RoomMessage>,
        (uuid, message): (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<State>> {
        match message {
            ClientLobby::Ready => {
                if self.ready.contains(&uuid) {
                    return Ok(None);
                }

                self.ready.push(uuid);

                clients.broadcast(ServerLobby::Ready {
                    uuid,
                    timer: self.update_countdown(room),
                });
            }
            ClientLobby::StartEarly => {
                if settings.owner != uuid || self.ready.len() < 2 {
                    return Ok(None);
                }

                return Ok(Some(State::InGame(Game::new(settings, &self.ready))));
            }
            ClientLobby::Unready => {
                let Some(index) = self.ready.iter().position(|client| *client == uuid) else {
                    return Ok(None);
                };

                self.ready.remove(index);

                clients.broadcast(ServerLobby::Unready {
                    uuid,
                    timer: self.update_countdown(room),
                });
            }
            ClientLobby::PracticeRequest => todo!(),
            ClientLobby::PracticeSubmission { prompt, input } => todo!(),
        }

        Ok(None)
    }

    fn handle_message(
        &mut self,
        settings: &mut RoomSettings,
        _clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils,
        _room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<State>> {
        match message {
            LobbyMessage::GameStart => Ok(Some(State::InGame(Game::new(settings, &self.ready)))),
        }
    }

    fn state(&self) -> Self::StateMessage {
        LobbyState {
            ready: self.ready.clone(),
            timer_start: self.countdown.as_ref().map(|countdown| countdown.start),
        }
    }

    fn end(&mut self) {
        if let Some(countdown) = &self.countdown {
            countdown.timer.abort();
        }
    }
}

impl Lobby {
    fn update_countdown(&mut self, room: impl RoomMessenger<LobbyMessage>) -> TimerAction {
        match &mut self.countdown {
            None if self.ready.len() >= 2 => {
                let start = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("time has gone backwards")
                    .as_micros() as u64;

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

#[cfg(test)]
mod tests {
    use super::*;

    struct Dummy;

    impl RoomMessenger<LobbyMessage> for Dummy {
        fn send(&self, _message: LobbyMessage) {}
    }

    #[test]
    fn countdown_none() {
        let mut lobby = Lobby::new();

        assert_eq!(lobby.update_countdown(Dummy), TimerAction::None);
        assert!(lobby.countdown.is_none())
    }
}
