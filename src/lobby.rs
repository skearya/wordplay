pub mod messages {
    use serde::{Deserialize, Serialize};
    use ts_rs::TS;
    use uuid::Uuid;

    use crate::{general::messages::ServerGameState, messages::RoomSettings};

    #[derive(Deserialize)]
    #[serde(tag = "kind", rename_all = "camelCase")]
    pub enum ClientLobby {
        RoomSettings(RoomSettings),
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
}

use std::time::{Duration, Instant};

use tokio::{sync::mpsc, task::AbortHandle};
use uuid::Uuid;

use crate::{
    lobby::messages::{ClientLobby, ServerLobby, TimerAction},
    messages::RoomMessage,
    room::clients::Messenger,
    task,
};

pub struct Lobby {
    ready: Vec<Uuid>,
    countdown: Option<Countdown>,
}

struct Countdown {
    start: Instant,
    timer: AbortHandle,
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            ready: vec![],
            countdown: None,
        }
    }

    // pub fn handle_room(
    //     &mut self,
    //     room: mpsc::UnboundedSender<RoomMessage>,
    //     clients: impl Messenger<ServerLobby>,
    //     message: LobbyMessage,
    // ) -> anyhow::Result<()> {
    // }

    pub fn handle_client(
        &mut self,
        room: mpsc::UnboundedSender<RoomMessage>,
        clients: impl Messenger<ServerLobby>,
        (uuid, message): (Uuid, ClientLobby),
    ) -> anyhow::Result<()> {
        match message {
            ClientLobby::RoomSettings(room_settings) => todo!(),
            ClientLobby::Ready => {
                if self.ready.contains(&uuid) {
                    return Ok(());
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
                    return Ok(());
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

        Ok(())
    }

    fn update_countdown(&mut self, room: mpsc::UnboundedSender<RoomMessage>) -> TimerAction {
        match &mut self.countdown {
            None if self.ready.len() >= 2 => {
                let start = Instant::now();

                let timer = task::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    room.send(RoomMessage::CloseCheck)?;

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
