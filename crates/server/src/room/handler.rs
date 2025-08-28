use uuid::Uuid;

use crate::{
    messages::RoomSettings,
    room::{
        StateChange,
        messenger::{ClientMessenger, ClientUtils},
    },
};

pub trait Handler {
    type ClientMessage;
    type ServerMessage;
    type RoomMessage;
    type StateMessage;

    fn state(&self) -> Self::StateMessage;

    fn client(
        &mut self,
        settings: &mut RoomSettings,
        clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils,
        message: (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<StateChange>;

    fn room(
        &mut self,
        settings: &mut RoomSettings,
        clients: impl ClientMessenger<Self::ServerMessage> + ClientUtils,
        message: Self::RoomMessage,
    ) -> anyhow::Result<StateChange>;

    fn abort(&mut self);
}

pub trait GameHandler {
    type ClientMessage;
    type ServerMessage;
    type RoomMessage;
    type StateMessage;
    type PostGameMessage;

    fn state(&self) -> Self::StateMessage;

    fn client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        message: (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<Self::PostGameMessage>>;

    fn room(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<Self::PostGameMessage>>;

    fn abort(&mut self);
}
