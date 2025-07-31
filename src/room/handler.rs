use uuid::Uuid;

use crate::{
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
    type StateMessage;

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

    fn state(&self) -> Self::StateMessage;

    fn end(&mut self);
}

pub trait GameHandler {
    type Settings;
    type ClientMessage;
    type ServerMessage;
    type RoomMessage;
    type StateMessage;
    type PostGameMessage;

    fn new(settings: &Self::Settings, players: &[Uuid]) -> Self;

    fn handle_client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<Self::PostGameMessage>>;

    fn handle_message(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<Self::PostGameMessage>>;

    fn state(&self) -> Self::StateMessage;

    fn end(&mut self);
}
