mod game;
mod general;
mod lobby;
mod messages;
mod room;
mod socket;
mod state;
mod task;

use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{socket::handler, state::AppState};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer().with_file(true).with_line_number(true))
        .with(EnvFilter::from_default_env())
        .init();

    let state = AppState::new();

    let app = Router::new()
        .route("/{room}", get(handler))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {:#?}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
