use uuid::Uuid;

use crate::{
    messages::RoomSettings,
    room::{
        State,
        messenger::{ClientMessenger, ClientUtils, RoomMessenger},
    },
};

pub trait Handler {
    type ClientMessage;
    type ServerMessage;
    type RoomMessage;
    type StateMessage;

    fn state(&self) -> Self::StateMessage;

    fn handle_client(
        &mut self,
        settings: &mut RoomSettings,
        clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<State>>;

    fn handle_message(
        &mut self,
        settings: &mut RoomSettings,
        clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<State>>;

    fn end(&mut self);
}

pub trait GameHandler {
    type GameSettings;
    type ClientMessage;
    type ServerMessage;
    type RoomMessage;
    type StateMessage;
    type PostGameMessage;

    fn new(settings: &Self::GameSettings, players: &[Uuid]) -> Self;

    fn state(&self) -> Self::StateMessage;

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

    fn end(&mut self) -> Self::PostGameMessage;
}
