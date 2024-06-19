mod db;
mod game;
mod global;
mod routes;
mod utils;

use axum::http::HeaderValue;
use axum::{routing::get, Router};
use game::GameState;
use global::{GlobalData, GLOBAL};
use routes::{api, auth, game::ws_handler};
use sqlx::SqlitePool;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: SqlitePool,
    game: Arc<Mutex<GameState>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    GLOBAL.set(GlobalData::new()).unwrap();
    dotenvy::from_path(Path::new("../.env")).ok();

    let db = db::create_pool().await?;
    let state = AppState::new(db);

    let app = Router::new()
        .nest("/api", api::make_router())
        .nest("/auth", auth::make_router(state.clone()))
        .route("/rooms/*room", get(ws_handler))
        .layer(
            CorsLayer::new().allow_origin(dotenvy::var("PUBLIC_FRONTEND")?.parse::<HeaderValue>()?),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
