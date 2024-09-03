pub mod error;
pub mod games;
pub mod lobby;
pub mod messages;
pub mod room;

use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use error::{GameError, Result};
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use messages::ClientMessage;
use room::Room;
use sqlx::SqlitePool;
use std::{num::NonZeroU32, sync::Arc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub rooms: Arc<DashMap<String, Room>>,
    pub limiter: Arc<DefaultKeyedRateLimiter<Uuid>>,
}

#[derive(Clone, Copy)]
pub struct SenderInfo<'a> {
    pub uuid: Uuid,
    pub room: &'a str,
}

impl AppState {
    pub fn new(db: SqlitePool) -> Self {
        Self {
            db,
            rooms: Arc::new(DashMap::new()),
            // maybe generous? though typing can count as a message
            limiter: Arc::new(RateLimiter::keyed(
                Quota::per_second(NonZeroU32::new(8).unwrap())
                    .allow_burst(NonZeroU32::new(24).unwrap()),
            )),
        }
    }

    pub fn room(&self, room: &str) -> Result<Ref<String, Room>> {
        self.rooms.get(room).ok_or(GameError::RoomNotFound {
            room: room.to_string(),
        })
    }

    pub fn room_mut(&self, room: &str) -> Result<RefMut<String, Room>> {
        self.rooms.get_mut(room).ok_or(GameError::RoomNotFound {
            room: room.to_string(),
        })
    }

    pub fn handle(&self, sender: SenderInfo, message: ClientMessage) {
        let result = self
            .limiter
            .check_key(&sender.uuid)
            .map_err(|_| GameError::RateLimited)
            .and_then(|()| match message {
                ClientMessage::Ping { timestamp } => self.client_ping(sender, timestamp),
                ClientMessage::Ready => self.client_ready(sender),
                ClientMessage::StartEarly => self.client_start_early(sender),
                ClientMessage::Unready => self.client_unready(sender),
                ClientMessage::PracticeRequest { game } => {
                    self.client_practice_request(sender, game)
                }
                ClientMessage::PracticeSubmission {
                    game,
                    prompt,
                    input,
                } => self.client_practice_submission(sender, game, prompt, input),
                ClientMessage::RoomSettings(settings) => {
                    self.client_room_settings(sender, settings)
                }
                ClientMessage::ChatMessage { content } => self.client_chat_message(sender, content),
                ClientMessage::WordBombInput { input } => self.word_bomb_input(sender, input),
                ClientMessage::WordBombGuess { word } => self.word_bomb_guess(sender, word),
                ClientMessage::AnagramsGuess { word } => self.anagrams_guess(sender, word),
            });

        if let Err(error) = result {
            eprintln!("error: {} caused {:#?}", sender.uuid, error);
            self.send_error_msg(sender, &error.to_string()).ok();
        }
    }
}
