mod game;
mod general;
mod global;
mod info;
mod lobby;
mod messages;
mod room;
mod socket;
mod state;

use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tower_http::cors::{self, CorsLayer};

use crate::{global::init_globals, state::AppState};

#[tokio::main]
async fn main() {
    init_globals();

    console_subscriber::init();

    let state = AppState::new();

    let app = Router::new()
        .route("/info", get(info::rooms))
        .route("/info/{room}", get(info::room))
        .route("/connect/{room}", get(socket::handler))
        .with_state(state)
        .layer(CorsLayer::new().allow_origin(cors::Any));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {:#?}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
