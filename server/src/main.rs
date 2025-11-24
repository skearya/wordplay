mod game;
mod general;
mod global;
mod lobby;
mod messages;
mod room;
mod socket;
mod state;

use axum::{Router, routing::get};
use tokio::net::TcpListener;

use crate::{global::init_globals, socket::handler, state::AppState};

#[tokio::main]
async fn main() {
    init_globals();

    console_subscriber::init();

    let state = AppState::new();

    let app = Router::new()
        .route("/{room}", get(handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {:#?}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
