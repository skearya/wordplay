use crate::{state::messages::Games, AppState};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;

pub fn make_router() -> Router<AppState> {
    Router::new()
        .route("/info", get(info))
        .route("/room-available/:room_name", get(room_available))
}

async fn info(State(state): State<AppState>) -> Json<ServerInfo> {
    Json(state.info())
}

async fn room_available(State(state): State<AppState>, Path(room_name): Path<String>) -> String {
    state.room_available(&room_name).to_string()
}

#[derive(Serialize, Debug)]
pub struct ServerInfo {
    pub clients_connected: usize,
    pub public_rooms: Vec<RoomData>,
}

#[derive(Serialize, Debug)]
pub struct RoomData {
    pub name: String,
    pub players: Vec<String>,
    pub game: Games,
}

impl AppState {
    pub fn info(&self) -> ServerInfo {
        let lock = self.game.lock().unwrap();

        ServerInfo {
            clients_connected: lock.rooms.values().map(|room| room.clients.len()).sum(),
            public_rooms: lock
                .rooms
                .iter()
                .filter(|(_, room)| room.settings.public)
                .map(|(name, data)| RoomData {
                    name: name.clone(),
                    players: data
                        .clients
                        .values()
                        .map(|client| client.username.clone())
                        .collect(),
                    game: data.settings.game,
                })
                .collect(),
        }
    }

    pub fn room_available(&self, name: &str) -> bool {
        let lock = self.game.lock().unwrap();

        !lock.rooms.contains_key(name)
    }
}
