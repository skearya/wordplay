pub mod messages {
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::room::general::messages::ServerGameState;

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
            state: ServerGameState,
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

    pub enum LobbyMessage {
        GameStart,
    }
}

use std::time::{Duration, Instant};

use tokio::task::AbortHandle;
use uuid::Uuid;

use crate::{
    lobby::messages::{ClientLobby, LobbyMessage, ServerLobby, TimerAction},
    messages::RoomSettings,
    room::{
        handler::Handler,
        messenger::{ClientMessenger, RoomMessenger},
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
    start: Instant,
    timer: AbortHandle,
}

impl Handler<State> for Lobby {
    type ClientMessage = ClientLobby;
    type ServerMessage = ServerLobby;
    type RoomMessage = LobbyMessage;

    fn new(settings: &RoomSettings) -> Self {
        Self {
            ready: vec![],
            countdown: None,
        }
    }

    fn handle_client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
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
            ClientLobby::StartEarly => todo!(),
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
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<State>> {
        match message {
            LobbyMessage::GameStart => Ok(Some(State::InGame(todo!()))),
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
                let start = Instant::now();

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

                TimerAction::Stop
            }
            Some(_) | None => TimerAction::None,
        }
    }
}
