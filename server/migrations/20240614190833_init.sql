-- Add migration script here
create table users(
    discord_id text primary key not null,
    username text unique not null,
    avatar_hash text not null
);

create table sessions(
    session_id text primary key not null,
    expires integer not null,
    discord_id text not null,
    foreign key (discord_id) references users(discord_id)
);

create table signup_sessions(
    signup_session_id text primary key not null,
    expires integer not null,
    discord_id text not null,
    avatar_hash text not null
);