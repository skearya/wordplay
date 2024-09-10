mod db;
mod global;
mod routes;
mod state;
mod utils;

use axum::http::HeaderValue;
use axum::{routing::get, Router};
use global::GLOBAL;
use routes::{auth, game, info};
use state::AppState;
use std::path::Path;
use std::sync::LazyLock;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    LazyLock::force(&GLOBAL);
    dotenvy::from_path(Path::new("../.env")).ok();

    let db = db::create_pool().await?;
    let state = AppState::new(db);

    let mut app = Router::new().nest(
        "/api",
        Router::new()
            .nest("/info", info::make_router())
            .nest("/auth", auth::make_router(state.clone()))
            .route("/room/*room", get(game::ws_handler))
            .with_state(state),
    );

    if cfg!(debug_assertions) {
        app = app.layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>()?)
                .allow_credentials(true),
        );
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3021").await?;
    println!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
