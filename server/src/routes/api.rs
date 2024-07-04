use crate::{state::ServerInfo, AppState};
use axum::{extract::State, routing::get, Json, Router};

pub fn make_router() -> Router<AppState> {
    Router::new().route("/info", get(info))
}

async fn info(State(state): State<AppState>) -> Json<ServerInfo> {
    Json(state.info())
}
