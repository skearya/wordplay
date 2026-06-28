use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
};
use serde::Serialize;
use ts_rs::TS;

use crate::{
    room::messages::{DetailedRoomInfo, RoomInfo},
    state::AppState,
};

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoomsInfoResponse {
    rooms: HashMap<String, RoomInfo>,
}

pub async fn rooms(State(state): State<AppState>) -> Json<RoomsInfoResponse> {
    let mut rooms_info = state.get_rooms_info().await;

    rooms_info.retain(|_name, room| room.settings.public);

    Json(RoomsInfoResponse { rooms: rooms_info })
}

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RoomInfoResponse {
    room: Option<DetailedRoomInfo>,
}

pub async fn room(
    State(state): State<AppState>,
    Path(room): Path<String>,
) -> Json<RoomInfoResponse> {
    let room_info = state.get_room_info(&room).await;

    Json(RoomInfoResponse { room: room_info })
}
