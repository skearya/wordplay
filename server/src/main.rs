mod db;
mod global;
mod routes;
mod state;
mod utils;

use axum::http::HeaderValue;
use axum::{routing::get, Router};
use global::{GlobalData, GLOBAL};
use routes::{api, auth, game};
use state::AppState;
use std::path::Path;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    GLOBAL.set(GlobalData::new()).unwrap();
    dotenvy::from_path(Path::new("../.env")).ok();

    let db = db::create_pool().await?;
    let state = AppState::new(db);

    let mut app = Router::new()
        .nest("/api", api::make_router())
        .nest("/auth", auth::make_router(state.clone()))
        .route("/rooms/*room", get(game::ws_handler))
        .with_state(state);

    if cfg!(debug_assertions) {
        app = app.layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>()?)
                .allow_credentials(true),
        )
    } else {
        app = app.fallback_service(
            ServeDir::new("../client/dist")
                .not_found_service(ServeFile::new("../client/dist/index.html")),
        );
    }

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3021").await?;
    println!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
