mod db;
mod global;
mod messages;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    GLOBAL.set(GlobalData::new()).unwrap();
    dotenvy::from_path(Path::new("../.env")).ok();

    let db = db::create_pool().await?;
    let state = AppState::new(db);

    let app = Router::new()
        .nest("/api", api::make_router())
        .nest("/auth", auth::make_router(state.clone()))
        .route("/rooms/*room", get(game::ws_handler))
        .layer(
            CorsLayer::new().allow_origin(dotenvy::var("PUBLIC_FRONTEND")?.parse::<HeaderValue>()?),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}
