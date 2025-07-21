use uuid::Uuid;

use crate::{
    messages::RoomSettings,
    room::messenger::{ClientMessenger, RoomMessenger},
};

pub trait Handler<T> {
    type ClientMessage;
    type ServerMessage;
    type RoomMessage;

    fn new(settings: &RoomSettings) -> Self;

    fn client(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: (Uuid, Self::ClientMessage),
    ) -> anyhow::Result<Option<T>>;

    fn room(
        &mut self,
        clients: impl ClientMessenger<Self::ServerMessage>,
        room: impl RoomMessenger<Self::RoomMessage>,
        message: Self::RoomMessage,
    ) -> anyhow::Result<Option<T>>;
}
