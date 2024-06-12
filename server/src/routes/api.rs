use crate::state::{AppState, ServerInfo};
use axum::{extract::State, Json};

pub async fn info(State(state): State<AppState>) -> Json<ServerInfo> {
    Json(state.info())
}
