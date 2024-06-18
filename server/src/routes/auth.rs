use crate::{
    db,
    state::AppState,
    utils::{random_string, AppError, UnixTime},
};
use anyhow::{anyhow, Context};
use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    middleware::{from_fn_with_state, Next},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use axum_extra::extract::CookieJar;
use cookie::{Cookie, SameSite};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use serde_json::Value;
use std::time::{Duration, SystemTime};

pub fn make_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/discord", get(discord_redirect))
        .route("/discord-callback", get(discord_callback))
        .route("/choose-username", post(choose_username))
        .layer(from_fn_with_state(state, check_if_already_authenticated))
}

async fn check_if_already_authenticated(
    jar: CookieJar,
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if let Some(session_id) = jar.get("session").map(Cookie::value) {
        if db::get_session(&state.db, session_id).await.is_ok() {
            return Err(StatusCode::FORBIDDEN);
        }
    }

    Ok(next.run(request).await)
}

async fn discord_redirect(jar: CookieJar) -> impl IntoResponse {
    let state = random_string(24);
    let redirect_url = format!(
        "https://discord.com/oauth2/authorize?client_id={}&response_type=code&redirect_uri={}/auth/discord-callback&scope=identify&state={}",
        dotenvy::var("DISCORD_CLIENT_ID").unwrap(),
        dotenvy::var("PUBLIC_SERVER").unwrap(),
        state
    );

    (
        jar.add(
            Cookie::build(("state", state))
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax)
                .max_age(cookie::time::Duration::minutes(10)),
        ),
        Redirect::to(&redirect_url),
    )
}

#[derive(Deserialize, Debug)]
pub struct DiscordCallbackParams {
    code: String,
    state: String,
}

async fn discord_callback(
    jar: CookieJar,
    State(state): State<AppState>,
    Query(params): Query<DiscordCallbackParams>,
) -> Result<Response, AppError> {
    let stored_state = jar.get("state");

    if !stored_state.is_some_and(|stored| stored.value() == params.state) {
        return Err(anyhow!("bad request"))?;
    }

    let client = reqwest::Client::new();

    let form = [
        ("grant_type", "authorization_code"),
        ("code", &params.code),
        (
            "redirect_uri",
            &format!(
                "{}/auth/discord-callback",
                dotenvy::var("PUBLIC_SERVER").unwrap()
            ),
        ),
        ("client_id", &dotenvy::var("DISCORD_CLIENT_ID").unwrap()),
        (
            "client_secret",
            &dotenvy::var("DISCORD_CLIENT_SECRET").unwrap(),
        ),
    ];

    let res: Value = client
        .post("https://discord.com/api/oauth2/token")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&form)
        .send()
        .await?
        .json()
        .await?;

    let access_token = res["access_token"]
        .as_str()
        .context("failed to authenticate")?;

    let res: Value = client
        .get("https://discord.com/api/users/@me")
        .header(AUTHORIZATION, format!("Bearer {access_token}"))
        .send()
        .await?
        .json()
        .await?;

    let discord_id = res["id"].as_str().context("something went wrong")?;

    if db::get_user(&state.db, discord_id).await.is_ok() {
        let session_id = random_string(24);
        let expires =
            (SystemTime::now() + Duration::from_secs(60 * 60 * 24 * 60)).to_unix_timestamp();

        db::insert_session(&state.db, &session_id, expires, discord_id).await?;

        Ok(jar
            .add(
                Cookie::build(("session", session_id))
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .max_age(cookie::time::Duration::days(60)),
            )
            .into_response())
    } else {
        let username = res["username"].as_str().context("something went wrong")?;
        let avatar_hash = res["avatar"].as_str().context("something went wrong")?;

        let signup_session_id = random_string(24);
        let expires = (SystemTime::now() + Duration::from_secs(60 * 60)).to_unix_timestamp();

        db::insert_signup_session(
            &state.db,
            &signup_session_id,
            expires,
            discord_id,
            avatar_hash,
        )
        .await?;

        Ok((
            jar.add(
                Cookie::build(("signup_session", signup_session_id))
                    .path("/")
                    .http_only(true)
                    .same_site(SameSite::Lax)
                    .max_age(cookie::time::Duration::hours(1)),
            ),
            Redirect::to(&format!(
                "{}/choose-username?suggested={username}",
                dotenvy::var("PUBLIC_FRONTEND").unwrap()
            )),
        )
            .into_response())
    }
}

#[derive(Deserialize, Debug)]
pub struct UsernameForm {
    username: String,
}

async fn choose_username(
    jar: CookieJar,
    State(state): State<AppState>,
    Form(input): Form<UsernameForm>,
) -> Result<impl IntoResponse, AppError> {
    let signup_session_id = jar.get("signup_session").context("bad request")?.value();
    let signup_session = db::get_signup_session(&state.db, signup_session_id).await?;

    if signup_session.expires < SystemTime::now().to_unix_timestamp().into() {
        return Err(anyhow!("session expired"))?;
    }

    // TODO: check if someone else has that username!!!
    db::insert_user(
        &state.db,
        &signup_session.discord_id,
        &input.username,
        &signup_session.avatar_hash,
    )
    .await?;

    db::delete_signup_session(&state.db, signup_session_id).await?;

    let session_id = random_string(24);
    let expires = (SystemTime::now() + Duration::from_secs(60 * 60 * 24 * 60)).to_unix_timestamp();

    db::insert_session(&state.db, &session_id, expires, &signup_session.discord_id).await?;

    Ok((
        jar.add(
            Cookie::build(("session", session_id))
                .path("/")
                .http_only(true)
                .same_site(SameSite::Lax)
                .max_age(cookie::time::Duration::days(60)),
        ),
        StatusCode::CREATED,
    ))
}
