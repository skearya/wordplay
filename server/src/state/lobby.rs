use crate::{
    global::GLOBAL,
    state::{
        error::Result,
        games::{
            anagrams::{self, Anagrams},
            word_bomb::{self, WordBomb, WordBombSettings},
        },
        messages::{CountdownState, Games, PostGameInfo, RoomStateInfo, ServerMessage},
        room::{check_for_new_room_owner, Client, RoomSettings, State},
        Room, SenderInfo,
    },
    utils::ClientUtils,
    AppState,
};
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::AbortHandle;
use uuid::Uuid;

#[derive(Debug)]
pub struct Lobby {
    pub ready: HashSet<Uuid>,
    pub countdown: Option<Countdown>,
}

#[derive(Debug)]
pub struct Countdown {
    pub time_left: u8,
    pub timer_handle: Arc<AbortHandle>,
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            ready: HashSet::new(),
            countdown: None,
        }
    }

    pub fn start_word_bomb(
        &self,
        app_state: AppState,
        room: String,
        settings: WordBombSettings,
    ) -> State {
        let timer_len = thread_rng().gen_range(10.0..=30.0);
        let prompt = GLOBAL.prompts.random_prompt(settings.min_wpm);
        let mut players: Vec<word_bomb::Player> = self
            .ready
            .iter()
            .map(|uuid| word_bomb::Player::new(*uuid))
            .collect();
        players.shuffle(&mut thread_rng());

        let task = Arc::new(
            tokio::spawn(async move {
                app_state
                    .word_bomb_timer(room, timer_len, prompt)
                    .await
                    .inspect_err(|error| eprintln!("word bomb timer error: {error:#?}"))
            })
            .abort_handle(),
        );

        State::WordBomb(WordBomb {
            settings,
            started_at: Instant::now(),
            timer: word_bomb::Timer {
                task,
                start: Instant::now(),
                length: timer_len,
            },
            prompt,
            prompt_uses: 0,
            missed_prompts: Vec::new(),
            turn: players[0].uuid,
            players,
        })
    }

    pub fn start_anagrams(&self, app_state: AppState, room: String) -> State {
        let (original, anagram) = GLOBAL.random_anagram();

        let timer = Arc::new(
            tokio::spawn(async move {
                app_state
                    .anagrams_timer(room)
                    .await
                    .inspect_err(|error| eprintln!("anagrams timer error: {error:#?}"))
            })
            .abort_handle(),
        );

        State::Anagrams(Anagrams {
            timer,
            anagram,
            original: original.to_string(),
            players: self
                .ready
                .iter()
                .map(|uuid| anagrams::Player::new(*uuid))
                .collect(),
        })
    }
}

impl AppState {
    pub fn client_ready(&self, SenderInfo { uuid, room }: SenderInfo) -> Result<()> {
        let mut lock = self.room_mut(room)?;
        let Room { clients, state, .. } = lock.value_mut();
        let lobby = state.try_lobby()?;

        if !lobby.ready.insert(uuid) {
            return Ok(());
        }

        let countdown_update = check_for_countdown_update(self.clone(), room.to_string(), lobby);

        clients.broadcast(ServerMessage::ReadyPlayers {
            ready: lobby.ready.iter().copied().collect(),
            countdown_update,
        });

        Ok(())
    }

    pub fn client_start_early(&self, SenderInfo { uuid, room }: SenderInfo) -> Result<()> {
        let mut lock = self.room_mut(room)?;
        let Room {
            clients,
            state,
            settings,
            owner,
        } = lock.value_mut();
        let lobby = state.try_lobby()?;

        if uuid == *owner && lobby.ready.len() >= 2 {
            if let Some(countdown) = lobby.countdown.as_ref() {
                countdown.timer_handle.abort();
            }

            start_game(self.clone(), room.to_owned(), state, clients, settings)?;
        }

        Ok(())
    }

    pub fn client_unready(&self, SenderInfo { uuid, room }: SenderInfo) -> Result<()> {
        let mut lock = self.room_mut(room)?;
        let Room { clients, state, .. } = lock.value_mut();
        let lobby = state.try_lobby()?;

        if !lobby.ready.remove(&uuid) {
            return Ok(());
        }

        let countdown_update = check_for_countdown_update(self.clone(), room.to_string(), lobby);

        clients.broadcast(ServerMessage::ReadyPlayers {
            ready: lobby.ready.iter().copied().collect(),
            countdown_update,
        });

        Ok(())
    }

    pub async fn start_when_ready(&self, room: String) -> Result<()> {
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let mut lock = self.room_mut(&room)?;
            let Room {
                clients,
                state,
                settings,
                ..
            } = lock.value_mut();
            let lobby = state.try_lobby()?;

            if let Some(countdown) = lobby.countdown.as_mut() {
                countdown.time_left -= 1;

                if countdown.time_left == 0 {
                    if lobby.ready.len() < 2 {
                        return Ok(());
                    }

                    start_game(self.clone(), room.clone(), state, clients, settings)?;
                } else {
                    clients.broadcast(ServerMessage::StartingCountdown {
                        time_left: countdown.time_left,
                    });
                }
            }
        }

        Ok(())
    }

    pub fn client_practice_request(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        game: Games,
    ) -> Result<()> {
        let lock = self.room(room)?;
        let Room {
            clients, settings, ..
        } = lock.value();

        let set: Vec<String> = match game {
            Games::WordBomb => (0..50)
                .map(|_| {
                    GLOBAL
                        .prompts
                        .random_prompt(settings.word_bomb.min_wpm)
                        .to_string()
                })
                .collect(),
            Games::Anagrams => (0..50).map(|_| GLOBAL.random_anagram().1).collect(),
        };

        clients[&uuid].send(ServerMessage::PracticeSet { set });

        Ok(())
    }

    pub fn client_practice_submission(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        game: Games,
        prompt: &str,
        input: &str,
    ) -> Result<()> {
        let lock = self.room(room)?;
        let Room { clients, .. } = lock.value();

        let correct = match game {
            Games::WordBomb => input.contains(prompt) && GLOBAL.is_valid(input),
            Games::Anagrams => {
                input.len() >= 2
                    && !input
                        .chars()
                        .any(|ch| input.matches(ch).count() > prompt.matches(ch).count())
                    && GLOBAL.is_valid(input)
            }
        };

        clients[&uuid].send(ServerMessage::PracticeResult { correct });

        Ok(())
    }

    pub fn client_room_settings(
        &self,
        SenderInfo { uuid, room }: SenderInfo,
        settings_update: RoomSettings,
    ) -> Result<()> {
        let mut lock = self.room_mut(room)?;
        let Room {
            clients,
            state,
            owner,
            settings,
        } = lock.value_mut();

        if state.try_lobby().is_ok() && *owner == uuid {
            settings.clone_from(&settings_update);
            clients.broadcast(ServerMessage::RoomSettings(settings_update));
        }

        Ok(())
    }
}

pub fn check_for_countdown_update(
    app_state: AppState,
    room: String,
    lobby: &mut Lobby,
) -> Option<CountdownState> {
    match &lobby.countdown {
        Some(countdown) if lobby.ready.len() < 2 => {
            countdown.timer_handle.abort();
            lobby.countdown = None;

            Some(CountdownState::Stopped)
        }
        None if lobby.ready.len() >= 2 => {
            lobby.countdown = Some(Countdown {
                timer_handle: Arc::new(
                    tokio::spawn(async move {
                        app_state
                            .start_when_ready(room)
                            .await
                            .inspect_err(|error| eprintln!("countdown error: {error:#?}"))
                    })
                    .abort_handle(),
                ),
                time_left: 10,
            });

            Some(CountdownState::InProgress { time_left: 10 })
        }
        _ => None,
    }
}

fn start_game(
    app_state: AppState,
    room: String,
    state: &mut State,
    clients: &mut HashMap<Uuid, Client>,
    settings: &RoomSettings,
) -> Result<()> {
    let lobby = state.try_lobby()?;

    for uuid in &lobby.ready {
        clients.get_mut(uuid).unwrap().rejoin_token = Some(Uuid::new_v4());
    }

    let game = match settings.game {
        Games::WordBomb => {
            *state = lobby.start_word_bomb(app_state, room, settings.word_bomb.clone());

            let game = state.try_word_bomb()?;

            RoomStateInfo::WordBomb {
                turn: game.turn,
                players: game.players.clone(),
                prompt: game.prompt.to_string(),
                used_letters: None,
            }
        }
        Games::Anagrams => {
            *state = lobby.start_anagrams(app_state, room);

            let game = state.try_anagrams()?;

            RoomStateInfo::Anagrams {
                players: game.players.clone(),
                anagram: game.anagram.clone(),
            }
        }
    };

    clients.send_each(|_uuid, client| ServerMessage::GameStarted {
        rejoin_token: client.rejoin_token,
        game: game.clone(),
    });

    Ok(())
}

pub fn end_game(
    state: &mut State,
    clients: &mut HashMap<Uuid, Client>,
    owner: &mut Uuid,
    info: PostGameInfo,
) {
    clients.retain(|_uuid, client| client.socket.is_some());

    for client in clients.values_mut() {
        client.rejoin_token = None;
    }

    let new_room_owner = check_for_new_room_owner(clients, owner);

    clients.broadcast(ServerMessage::GameEnded {
        new_room_owner,
        info,
    });

    *state = State::default();
}
