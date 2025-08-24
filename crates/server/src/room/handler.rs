use uuid::Uuid;

use crate::{
    messages::RoomSettings,
    room::{
        StateChange,
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
    ) -> anyhow::Result<StateChange>;

    fn handle_message(
        &mut self,
        settings: &mut RoomSettings,
        clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<StateChange>;

    fn abort(&mut self);
}

pub trait GameHandler {
    type GameSettings;
    type ClientMessage;
    type ServerMessage;
    type RoomMessage;
    type StateMessage;
    type PostGameMessage;

    fn new(
        settings: &Self::GameSettings,
        players: &[Uuid],
        room: impl RoomMessenger<Self::RoomMessage>,
    ) -> Self;

    fn state(&self) -> Self::StateMessage;

    fn handle_client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        message: (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<Self::PostGameMessage>>;

    fn handle_message(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<Self::PostGameMessage>>;

    fn abort(&mut self);
}
