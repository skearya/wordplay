use uuid::Uuid;

use crate::{
    game::messages::PostGameInfo,
    messages::RoomSettings,
    room::{
        messenger::{ClientMessenger, RoomMessenger},
        state::State,
    },
};

pub trait Handler {
    type ClientMessage;
    type ServerMessage;
    type RoomMessage;

    fn handle_client(
        &mut self,
        settings: &RoomSettings,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<State>>;

    fn handle_message(
        &mut self,
        settings: &RoomSettings,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<State>>;

    fn end(&mut self);
}

pub trait GameHandler {
    type ClientMessage;
    type ServerMessage;
    type RoomMessage;
    type Settings;

    fn new(settings: &Self::Settings, players: &[Uuid]) -> Self;

    fn handle_client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<PostGameInfo>>;

    fn handle_message(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<PostGameInfo>>;

    fn end(&mut self);
}
