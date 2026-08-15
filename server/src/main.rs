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
use tower_http::{
    cors::{self, CorsLayer},
    services::{ServeDir, ServeFile},
};

use crate::{global::init_globals, state::AppState};

#[tokio::main]
async fn main() {
    init_globals();

    tracing_subscriber::fmt::init();

    let state = AppState::new();

    let app = Router::new()
        .route("/info", get(info::rooms))
        .route("/info/{room}", get(info::room))
        .route("/connect/{room}", get(socket::handler))
        .fallback_service(
            ServeDir::new("assets").not_found_service(ServeFile::new("assets/200.html")),
        )
        .layer(CorsLayer::new().allow_origin(cors::Any))
        .with_state(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {:#?}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
