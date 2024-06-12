mod global;
mod messages;
mod routes;
mod state;
mod utils;

use axum::{routing::get, Router};
use global::{GlobalData, GLOBAL};
use routes::api;
use routes::game::ws_handler;
use state::AppState;
use tower_http::cors::{self, CorsLayer};

#[tokio::main]
async fn main() {
    GLOBAL.set(GlobalData::new()).unwrap();

    let app = Router::new()
        .route("/info", get(api::info))
        .route("/rooms/*room", get(ws_handler))
        .layer(CorsLayer::new().allow_origin(cors::Any))
        .with_state(AppState::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
