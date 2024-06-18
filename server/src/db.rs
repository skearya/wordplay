#![allow(dead_code)]
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteQueryResult},
    Pool, Result, Sqlite, SqlitePool,
};
use std::str::FromStr;

pub async fn create_pool() -> anyhow::Result<Pool<Sqlite>> {
    let options = SqliteConnectOptions::from_str(&dotenvy::var("DATABASE_URL")?)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal);

    let pool = SqlitePool::connect_with(options).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}

#[derive(Debug)]
pub struct User {
    discord_id: String,
    username: String,
    avatar_hash: String,
}

#[derive(Debug)]
pub struct SignupSession {
    pub signup_session_id: String,
    pub expires: i64,
    pub discord_id: String,
    pub avatar_hash: String,
}

#[derive(Debug)]
pub struct Session {
    pub session_id: String,
    pub expires: i64,
    pub discord_id: String,
}

pub async fn get_user(pool: &SqlitePool, discord_id: &str) -> Result<User> {
    let query = sqlx::query_as!(
        User,
        "select * from users where discord_id is ?",
        discord_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(query)
}

pub async fn get_user_from_session(pool: &SqlitePool, session_id: &str) -> Result<User> {
    let query = sqlx::query_as!(
        User,
        "
            select
                users.*
            from
                users
                inner join sessions using (discord_id)
            where
                sessions.session_id is ? and sessions.expires > unixepoch()
        ",
        session_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(query)
}

pub async fn insert_user(
    pool: &SqlitePool,
    discord_id: &str,
    username: &str,
    avatar_hash: &str,
) -> Result<SqliteQueryResult> {
    let query = sqlx::query!(
        "insert into users (discord_id, username, avatar_hash) values (?, ?, ?)",
        discord_id,
        username,
        avatar_hash
    )
    .execute(pool)
    .await?;

    Ok(query)
}

pub async fn get_session(pool: &SqlitePool, session_id: &str) -> Result<Session> {
    let query = sqlx::query_as!(
        Session,
        "select * from sessions where session_id is ? and expires > unixepoch()",
        session_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(query)
}

pub async fn insert_session(
    pool: &SqlitePool,
    session_id: &str,
    expires: u32,
    discord_id: &str,
) -> Result<SqliteQueryResult> {
    let query = sqlx::query!(
        "insert into sessions (session_id, expires, discord_id) values (?, ?, ?)",
        session_id,
        expires,
        discord_id,
    )
    .execute(pool)
    .await?;

    Ok(query)
}

pub async fn get_signup_session(
    pool: &SqlitePool,
    signup_session_id: &str,
) -> Result<SignupSession> {
    let query = sqlx::query_as!(
        SignupSession,
        "select * from signup_sessions where signup_session_id is ?",
        signup_session_id,
    )
    .fetch_one(pool)
    .await?;

    Ok(query)
}

pub async fn insert_signup_session(
    pool: &SqlitePool,
    signup_session_id: &str,
    expires: u32,
    discord_id: &str,
    avatar_hash: &str,
) -> Result<SqliteQueryResult> {
    let query = sqlx::query!(
        "insert into signup_sessions (signup_session_id, expires, discord_id, avatar_hash) values (?, ?, ?, ?)",
        signup_session_id,
        expires,
        discord_id,
        avatar_hash
    )
    .execute(pool)
    .await?;

    Ok(query)
}

pub async fn delete_signup_session(
    pool: &SqlitePool,
    signup_session_id: &str,
) -> Result<SqliteQueryResult> {
    let query = sqlx::query!(
        "delete from signup_sessions where signup_session_id is ?",
        signup_session_id
    )
    .execute(pool)
    .await?;

    Ok(query)
}
